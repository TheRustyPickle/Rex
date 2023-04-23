use crate::page_handler::{BACKGROUND, BOX, RED, TEXT};
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Modifier, Style};
use tui::text::{Span, Spans, Text};
use tui::widgets::{Block, Borders, Clear, Paragraph};
use tui::Frame;

/// Creates a popup on top of a window with the given size, title and text attributes
pub fn create_popup<B: Backend>(f: &mut Frame<B>, popup_data: &[String]) {
    let size = f.size();
    
    let title = Span::styled(&popup_data[0], Style::default().add_modifier(Modifier::BOLD));
    let text = &popup_data[1];

    let mut text_data = Vec::new();

    for line in text.split("\n") {
        let splitted = line.split_once(":");
        if let Some((first_part, rest)) = splitted {
            let first_data =
                Span::styled(first_part, Style::default().add_modifier(Modifier::BOLD));
            let rest_data = Span::from(format!(":{rest}"));
            text_data.push(Spans::from(vec![first_data, rest_data]));
        } else {
            text_data.push(Spans::from(vec![Span::from(line)]))
        }
    }

    // determines the size of the popup window
    let x_value = popup_data[2].parse::<u16>().unwrap();
    let y_value = popup_data[3].parse::<u16>().unwrap();

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().bg(BACKGROUND).fg(BOX));

    // returns an area where we can add anything like a normal window.
    let area = centered_rect(x_value, y_value, size);

    let new_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Min(1), Constraint::Length(1)].as_ref())
        .split(area);

    f.render_widget(Clear, area);
    f.render_widget(block, area);

    //let help_sec = Paragraph::new(text)
    //    .style(Style::default().bg(BACKGROUND).fg(TEXT))
    //    .alignment(Alignment::Left);

    let help_sec =
        Paragraph::new(Text::from(text_data)).style(Style::default().bg(BACKGROUND).fg(TEXT));

    let dismiss_sec = Paragraph::new("Press Any Key To Dismiss")
        .style(
            Style::default()
                .bg(BACKGROUND)
                .fg(RED)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);

    f.render_widget(help_sec, new_chunks[0]);
    f.render_widget(dismiss_sec, new_chunks[1]);
}

/// The function takes certain parameters to create an empty space in the layout
/// and returns an area where we can place various widgets. Taken from tui-rs examples.
/// This is used as a popup for helpful information.
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
