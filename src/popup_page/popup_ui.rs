use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Creates a popup on top of a window with the given size, title and text attributes
pub fn create_popup<B: Backend>(f: &mut Frame<B>, popup_data: &[String]) {
    let size = f.size();
    let title = popup_data[0].to_string();
    let text = popup_data[1].to_string();

    // determines the size of the popup window
    let x_value = popup_data[2].parse::<u16>().unwrap();
    let y_value = popup_data[3].parse::<u16>().unwrap();

    let block = Block::default().title(title).borders(Borders::ALL).style(
        Style::default()
            .bg(Color::Rgb(255, 255, 255))
            .fg(Color::Rgb(50, 205, 50)),
    );

    // returns an area where we can add anything like a normal window.
    let area = centered_rect(x_value, y_value, size);

    let new_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(area);

    f.render_widget(Clear, area);
    f.render_widget(block, area);

    let help_sec = Paragraph::new(text)
        .style(
            Style::default()
                .bg(Color::Rgb(255, 255, 255))
                .fg(Color::Rgb(50, 205, 50)),
        )
        .alignment(Alignment::Left);
    f.render_widget(help_sec, new_chunks[0]);
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
