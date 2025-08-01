use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Span, Text};
use ratatui::widgets::{
    Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap,
};

use crate::page_handler::{BACKGROUND, BLUE, BOX, DeletionStatus, HIGHLIGHTED, RED, TEXT};
use crate::utility::create_bolded_text;

/// Creates a popup on top of a window with the given size, title, and text attributes
#[cfg(not(tarpaulin_include))]
pub fn create_popup(f: &mut Frame, title: &str, text: &str, position: usize) {
    let size = f.area();
    let x_value = 60;
    let y_value = 60;

    let title = Span::styled(title, Style::default().add_modifier(Modifier::BOLD));

    let text_len = text.split('\n').count() + 5;

    let text = create_bolded_text(text);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().bg(BACKGROUND).fg(BOX));

    // Returns an area where we can add anything like a normal window.
    let area = centered_rect(x_value, y_value, size);

    let new_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    f.render_widget(Clear, area);
    f.render_widget(block, area);

    let help_sec = Paragraph::new(Text::from(text))
        .style(Style::default().bg(BACKGROUND).fg(TEXT))
        .wrap(Wrap::default())
        .scroll((position as u16, 0));

    let dismiss_sec = Paragraph::new("Use Arrow Keys To Scroll. Press Any Other Key To Dismiss")
        .style(
            Style::default()
                .bg(BACKGROUND)
                .fg(RED)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
    let mut scrollbar_state = ScrollbarState::new(text_len)
        .position(position)
        .content_length(text_len - 5);

    f.render_widget(help_sec, new_chunks[0]);
    f.render_widget(dismiss_sec, new_chunks[1]);
    f.render_stateful_widget(scrollbar, new_chunks[0], &mut scrollbar_state);
}

#[cfg(not(tarpaulin_include))]
pub fn create_deletion_popup(f: &mut Frame, deletion_status: &DeletionStatus) {
    let text = "Are you sure you want to delete this transaction?";
    let title = "TX Deletion";
    let size = f.area();

    let title = Span::styled(title, Style::default().add_modifier(Modifier::BOLD));
    let text = create_bolded_text(text);

    // Determines the size of the popup window
    let x_value = 40;
    let y_value = 25;

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().bg(BACKGROUND).fg(BOX));

    // Returns an area where we can add anything like a normal window.
    let area = centered_rect(x_value, y_value, size);

    let new_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Min(1), Constraint::Length(5)])
        .split(area);

    let selection_chunk = Layout::default()
        .direction(Direction::Horizontal)
        .margin(2)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(new_chunks[1]);

    f.render_widget(Clear, area);
    f.render_widget(block, area);

    let deletion_text = Paragraph::new(Text::from(text))
        .style(Style::default().bg(BACKGROUND).fg(TEXT))
        .alignment(Alignment::Center);

    let yes_text = match deletion_status {
        DeletionStatus::Yes => Span::styled(
            " Yes ",
            Style::default()
                .fg(RED)
                .add_modifier(Modifier::BOLD)
                .bg(HIGHLIGHTED),
        ),
        DeletionStatus::No => Span::styled(
            " Yes ",
            Style::default().fg(RED).add_modifier(Modifier::BOLD),
        ),
    };

    let no_text = match deletion_status {
        DeletionStatus::No => Span::styled(
            " No ",
            Style::default()
                .fg(BLUE)
                .add_modifier(Modifier::BOLD)
                .bg(HIGHLIGHTED),
        ),
        DeletionStatus::Yes => Span::styled(
            " No ",
            Style::default().fg(BLUE).add_modifier(Modifier::BOLD),
        ),
    };

    let yes_sec = Paragraph::new(yes_text).alignment(Alignment::Center);

    let no_sec = Paragraph::new(no_text).alignment(Alignment::Center);

    f.render_widget(deletion_text, new_chunks[0]);
    f.render_widget(yes_sec, selection_chunk[0]);
    f.render_widget(no_sec, selection_chunk[1]);
}

/// The function takes certain parameters to create an empty space in the layout
/// and returns an area where we can place various widgets. Taken from tui-rs examples.
/// This is used as a popup for helpful information.
#[cfg(not(tarpaulin_include))]
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
