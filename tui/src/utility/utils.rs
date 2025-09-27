use crossterm::execute;
use crossterm::terminal::{
    Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Tabs};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::{Read, Result as ioResult, Stdout, Write, stdout};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use crate::outputs::ComparisonType;
use crate::page_handler::{BACKGROUND, BOX, HIGHLIGHTED, IndexedData, RED, SortingType, TEXT};

const RESTRICTED: [&str; 6] = ["Total", "Balance", "Changes", "Income", "Expense", "Cancel"];

/// Enters raw mode so the TUI can render properly
pub fn enter_tui_interface() -> Result<Terminal<CrosstermBackend<Stdout>>, Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// Exits raw mode so the terminal starts working normally
pub fn exit_tui_interface() -> Result<(), Box<dyn Error>> {
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

/// Does the 5 second timer after input taking ends
pub fn start_timer<T: std::fmt::Display>(input: T) {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    for i in (1..6).rev() {
        write!(handle, "\r{input} Restarting in {i} seconds").unwrap();
        handle.flush().unwrap();
        thread::sleep(Duration::from_millis(1000));
    }
    println!("\n");
}

/// Takes a user input and returns the trimmed input as String
#[must_use]
pub fn take_input() -> String {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

/// Clears the terminal of all text
pub fn clear_terminal(stdout: &mut Stdout) {
    execute!(stdout, Clear(ClearType::FromCursorUp)).unwrap();
}

/// Flushes output to the terminal
pub fn flush_output(stdout: &Stdout) {
    let mut handle = stdout.lock();
    handle.flush().unwrap();
}

/// Checks if the input is a restricted word or inside a given vector
#[must_use]
pub fn check_restricted(item: &str, restricted: Option<&Vec<String>>) -> bool {
    if let Some(restricted_words) = restricted {
        for restricted_item in restricted_words {
            if restricted_item.to_lowercase() == item.to_lowercase() {
                return true;
            }
        }
    } else {
        for &restricted_item in &RESTRICTED {
            if restricted_item.to_lowercase() == item.to_lowercase() {
                return true;
            }
        }
    }

    false
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

#[derive(Serialize, Deserialize)]
struct LocationInfo {
    location: String,
}

#[derive(Serialize, Deserialize)]
struct BackupPaths {
    locations: Vec<String>,
}

/// Checks if location.json exists and returns a path if it exists
#[must_use]
pub fn is_location_changed(working_dir: &PathBuf) -> Option<PathBuf> {
    let mut json_path = working_dir.to_owned();
    json_path.pop();
    json_path.push("location.json");

    if json_path.exists() {
        let mut file = File::open(json_path).unwrap();
        let mut file_content = String::new();
        file.read_to_string(&mut file_content).unwrap();
        let location_info: LocationInfo = serde_json::from_str(&file_content).unwrap();
        Some(PathBuf::from(location_info.location))
    } else {
        None
    }
}

/// Creates a location.json file to store the new app data location
pub fn create_change_location_file(working_dir: &PathBuf, new_path: &Path) {
    let mut target_dir = working_dir.to_owned();
    target_dir.pop();

    let location = LocationInfo {
        location: new_path.to_str().unwrap().to_string(),
    };

    target_dir.push("location.json");

    let mut file = File::create(target_dir).unwrap();

    serde_json::to_writer(&mut file, &location).unwrap();
}

/// Create a `backup_paths.json` file to store the location of where backup db will be located
pub fn create_backup_location_file(original_db_path: &PathBuf, backup_paths: Vec<PathBuf>) {
    let mut target_dir = original_db_path.to_owned();
    target_dir.pop();

    let backup = BackupPaths {
        locations: backup_paths
            .into_iter()
            .map(|path| path.to_str().unwrap().to_owned())
            .collect(),
    };

    target_dir.push("backup_paths.json");
    let mut file = File::create(target_dir).unwrap();
    serde_json::to_writer(&mut file, &backup).unwrap();
}

/// Copies the latest DB to the backup location specified in `backend_paths.json`
pub fn save_backup_db(db_path: &PathBuf, original_db_path: &PathBuf) {
    let mut json_path = original_db_path.to_owned();
    json_path.pop();

    json_path.push("backup_paths.json");

    if !json_path.exists() {
        return;
    }

    let mut file = File::open(json_path).unwrap();
    let mut file_content = String::new();
    file.read_to_string(&mut file_content).unwrap();
    let location_info: BackupPaths = serde_json::from_str(&file_content).unwrap();

    for path in location_info.locations {
        let mut target_path = PathBuf::from(path);

        if !target_path.exists() {
            println!("Failed to find path {}", target_path.to_string_lossy());
            continue;
        }
        target_path.push("data.sqlite");
        if let Err(e) = fs::copy(db_path, &target_path) {
            println!(
                "Failed to copy DB to backup path {}. Error: {e:?}",
                target_path.to_string_lossy()
            );
            continue;
        }
    }
}

/// Deletes `backup_paths.json` which contains all locations where backup DB is located.
pub fn delete_backup_db(original_db_path: &PathBuf) -> ioResult<()> {
    let mut json_path = original_db_path.to_owned();
    json_path.pop();

    json_path.push("backup_paths.json");

    if !json_path.exists() {
        return Ok(());
    }

    fs::remove_file(json_path)
}

/// Deletes `locations.json` file which stores alternative location information of the DB.
pub fn delete_location_change(original_db_path: &PathBuf) -> ioResult<()> {
    let mut json_path = original_db_path.to_owned();
    json_path.pop();

    json_path.push("location.json");

    if !json_path.exists() {
        return Ok(());
    }

    fs::remove_file(json_path)
}
