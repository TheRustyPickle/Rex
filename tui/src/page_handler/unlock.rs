use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Position};
use ratatui::style::{Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{BorderType, Borders, Paragraph};

use crate::theme::Theme;
use crate::utility::{
    centered_rect_exact, enter_tui_interface, exit_tui_interface, main_block, styled_block,
};

/// Asks for the database password on a screen of its own, before the app has a connection.
/// Returns `None` if the user quits instead of answering.
pub fn ask_password(theme: &Theme, error: Option<&str>) -> Result<Option<String>> {
    let mut terminal = enter_tui_interface()?;
    let mut password = String::new();

    let answer = loop {
        terminal.draw(|f| show_unlock_ui(f, theme, password.chars().count(), error))?;

        let Event::Key(key) = event::read()? else {
            continue;
        };

        if key.kind != KeyEventKind::Press {
            continue;
        }

        match key.code {
            KeyCode::Enter if !password.is_empty() => break Some(password),
            KeyCode::Esc => break None,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break None,
            KeyCode::Char(c) => password.push(c),
            KeyCode::Backspace => {
                password.pop();
            }
            _ => {}
        }
    };

    exit_tui_interface()?;

    Ok(answer)
}

/// Only the length of the password is ever given here, so nothing typed can end up on screen
fn show_unlock_ui(f: &mut Frame, theme: &Theme, password_len: usize, error: Option<&str>) {
    let area = centered_rect_exact(50, 10, f.area());

    f.render_widget(
        main_block(theme)
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .title(Span::styled(
                "Rex - Locked",
                Style::default().add_modifier(Modifier::BOLD),
            )),
        area,
    );

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(area);

    let masked = "*".repeat(password_len);

    f.render_widget(
        Paragraph::new(masked)
            .style(Style::default().bg(theme.background()).fg(theme.text()))
            .block(styled_block("Password", theme)),
        chunks[0],
    );

    f.set_cursor_position(Position {
        x: chunks[0].x + password_len as u16 + 1,
        y: chunks[0].y + 1,
    });

    let (status, color) = match error {
        Some(error) => (error.to_string(), theme.negative()),
        None => (String::new(), theme.text()),
    };

    f.render_widget(
        Paragraph::new(status)
            .style(
                Style::default()
                    .bg(theme.background())
                    .fg(color)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center),
        chunks[1],
    );

    f.render_widget(
        Paragraph::new("Enter to unlock. Esc to quit")
            .style(Style::default().bg(theme.background()).fg(theme.text()))
            .alignment(Alignment::Center),
        chunks[3],
    );
}
