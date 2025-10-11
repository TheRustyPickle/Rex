use anyhow::Result;
use app::conn::{get_conn, get_conn_old};
use app::migration::start_migration;
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Tabs};
use std::collections::HashMap;
use std::io::{Stdout, stdout};
use std::path::{Path, PathBuf};

use crate::outputs::ComparisonType;
use crate::page_handler::{BACKGROUND, BOX, HIGHLIGHTED, IndexedData, RED, SortingType, TEXT};

pub const RESTRICTED: [&str; 8] = [
    "Total",
    "Balance",
    "Changes",
    "Income",
    "Expense",
    "Cancel",
    "Daily Income",
    "Daily Expense",
];

/// Enters raw mode so the TUI can render properly
pub fn enter_tui_interface() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// Exits raw mode so the terminal starts working normally
pub fn exit_tui_interface() -> Result<()> {
    let stdout = stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    disable_raw_mode()?;
    Ok(())
}

/// Returns a styled block for UI to use
#[must_use]
pub fn styled_block(title: &str) -> Block<'_> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().bg(BACKGROUND).fg(BOX))
        .title(Span::styled(
            title,
            Style::default().add_modifier(Modifier::BOLD),
        ))
}

#[must_use]
pub fn styled_block_no_top(title: &str) -> Block<'_> {
    Block::default()
        .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
        .border_type(BorderType::Rounded)
        .style(Style::default().bg(BACKGROUND).fg(BOX))
        .title(Span::styled(
            title,
            Style::default().add_modifier(Modifier::BOLD),
        ))
}

#[must_use]
pub fn styled_block_no_bottom(title: &str) -> Block<'_> {
    Block::default()
        .borders(Borders::LEFT | Borders::RIGHT | Borders::TOP)
        .border_type(BorderType::Rounded)
        .style(Style::default().bg(BACKGROUND).fg(BOX))
        .title(Span::styled(
            title,
            Style::default().add_modifier(Modifier::BOLD),
        ))
}

#[must_use]
pub fn main_block<'a>() -> Block<'a> {
    Block::default().style(Style::default().bg(BACKGROUND).fg(BOX))
}

/// Takes a string and makes any word before the first occurrence of : to Bold
/// Used for rendering
#[must_use]
pub fn create_bolded_text(text: &str) -> Vec<Line<'_>> {
    let mut text_data = Vec::new();

    for line in text.split('\n') {
        let split_text = line.split_once(':');
        if let Some((first_part, rest)) = split_text {
            let first_data =
                Span::styled(first_part, Style::default().add_modifier(Modifier::BOLD));
            let rest_data = Span::from(format!(":{rest}"));
            text_data.push(Line::from(vec![first_data, rest_data]));
        } else {
            text_data.push(Line::from(vec![Span::from(line)]));
        }
    }

    text_data
}

/// Tabs from some given data for the UI
#[must_use]
pub fn create_tab<'a>(data: &'a IndexedData, name: &'a str) -> Tabs<'a> {
    let titles: Vec<Line> = data
        .titles
        .iter()
        .map(|t| Line::from(vec![Span::styled(t, Style::default().fg(TEXT))]))
        .collect();

    Tabs::new(titles)
        .block(styled_block(name))
        .select(data.index)
        .style(Style::default().fg(BOX))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(HIGHLIGHTED),
        )
}

/// Create a tab with some values where each value's color will depend on the provided `HashMap` bool value
#[must_use]
pub fn create_tab_activation<'a>(
    data: &'a IndexedData,
    name: &'a str,
    activation: &HashMap<String, bool>,
) -> Tabs<'a> {
    let titles: Vec<Line> = data
        .titles
        .iter()
        .map(|t| {
            if activation[t] {
                Line::from(vec![Span::styled(t, Style::default().fg(TEXT))])
            } else {
                Line::from(vec![Span::styled(t, Style::default().fg(RED))])
            }
        })
        .collect();

    Tabs::new(titles)
        .block(styled_block(name))
        .select(data.index)
        .style(Style::default().fg(BOX))
        .highlight_style(Style::default())
}

/// Parse GitHub release information for popup menu
#[must_use]
pub fn parse_github_body(body: &str) -> String {
    let body = body.replace("## Updates", "");
    let body = body.replace('*', "•");
    let body = body.replace('\r', "");
    let end_point = body.find("## Changes").unwrap();
    format!("\n{}\n", &body[..end_point].trim())
}

/// Used for sorting summary table data
#[must_use]
pub fn sort_table_data(mut data: Vec<Vec<String>>, sort_type: &SortingType) -> Vec<Vec<String>> {
    match sort_type {
        SortingType::Tags => data.sort(),
        SortingType::Income => data.sort_by(|a, b| {
            let val_a: f64 = a[1].parse().unwrap();
            let val_b: f64 = b[1].parse().unwrap();
            val_b.partial_cmp(&val_a).unwrap()
        }),
        SortingType::Expense => {
            data.sort_by(|a, b| {
                let val_a: f64 = a[2].parse().unwrap();
                let val_b: f64 = b[2].parse().unwrap();
                val_b.partial_cmp(&val_a).unwrap()
            });
        }
    }

    data
}

/// Adds a char to the given index on the given string
pub fn add_char_to(to_add: Option<char>, current_index: &mut usize, current_data: &mut String) {
    if *current_index > current_data.len() {
        *current_index = current_data.len();
    } else {
        match to_add {
            Some(ch) => {
                current_data.insert(*current_index, ch);
                *current_index += 1;
            }
            None => {
                if !current_data.is_empty() && *current_index != 0 {
                    current_data.remove(*current_index - 1);
                    *current_index -= 1;
                }
            }
        }
    }
}

/// Checks if the string contains any symbol indicating comparison
#[must_use]
pub fn check_comparison(input: &str) -> ComparisonType {
    // Need to handle 2 letter ones first otherwise in case of >=
    // it will match with >
    if input.starts_with("<=") {
        ComparisonType::EqualOrSmaller
    } else if input.starts_with(">=") {
        ComparisonType::EqualOrBigger
    } else if input.starts_with('<') {
        ComparisonType::SmallerThan
    } else if input.starts_with('>') {
        ComparisonType::BiggerThan
    } else {
        ComparisonType::Equal
    }
}

pub fn migrate_to_new_schema(old_conn_path: &Path, new_conn: &str) -> Result<bool> {
    if !old_conn_path.exists() {
        return Ok(false);
    }

    let old_conn = get_conn_old(old_conn_path.to_string_lossy().as_ref());

    if PathBuf::from(new_conn).exists() {
        return Ok(false);
    }

    let mut new_conn = get_conn(new_conn);

    start_migration(old_conn, &mut new_conn)?;
    Ok(true)
}

pub fn centered_rect_exact(width: u16, height: u16, r: Rect) -> Rect {
    let w = width.min(r.width);
    let h = height.min(r.height);

    let horizontal_space = r.width.saturating_sub(w);
    let vertical_space = r.height.saturating_sub(h);

    let left = horizontal_space / 2;
    let top = vertical_space / 2;

    let right = horizontal_space - left;
    let bottom = vertical_space - top;

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(top),
            Constraint::Length(h),
            Constraint::Length(bottom),
        ])
        .split(r);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(left),
            Constraint::Length(w),
            Constraint::Length(right),
        ])
        .split(vertical[1]);

    horizontal[1]
}
