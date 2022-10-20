use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::home_page::TableData;

/// Renders the Summary UI page
pub fn summary_ui<B: Backend>(
    f: &mut Frame<B>,
    table_data: &mut TableData,
    text_data: &Vec<(f64, String)>,
) {
    let size = f.size();

    let normal_style = Style::default().bg(Color::LightBlue);
    let selected_style = Style::default().bg(Color::Rgb(255, 245, 238));

    let header_cells = ["Tag", "Total Income", "Total Expense"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Rgb(255, 255, 255))));

    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(0);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Length(7), Constraint::Min(5)].as_ref())
        .split(size);

    let block = Block::default().style(
        Style::default()
            .bg(Color::Rgb(255, 255, 255))
            .fg(Color::Rgb(50, 205, 50)),
    );

    f.render_widget(block, size);

    // * contains the text for the upper side of the Summary UI
    let text = vec![
        Spans::from(Span::styled(
            format!("{} {:.2}", text_data[0].1, text_data[0].0),
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Blue),
        )),
        Spans::from(Span::styled(
            format!("{} {:.2}", text_data[1].1, text_data[1].0),
            Style::default().add_modifier(Modifier::BOLD).fg(Color::Red),
        )),
        Spans::from(vec![
            Span::styled(
                format!("Largest Income: {:.2}, ", text_data[2].0),
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Blue),
            ),
            Span::styled(
                format!("Method: {}", text_data[2].1),
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Rgb(205, 133, 63)),
            ),
        ]),
        Spans::from(vec![
            Span::styled(
                format!("Largest Expense: {:.2}, ", text_data[3].0),
                Style::default().add_modifier(Modifier::BOLD).fg(Color::Red),
            ),
            Span::styled(
                format!("Method: {}", text_data[3].1),
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Rgb(205, 133, 63)),
            ),
        ]),
        Spans::from(vec![
            Span::styled(
                format!("Most Earning Month: {}, ", text_data[4].1),
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Rgb(100, 149, 237)),
            ),
            Span::styled(
                format!("Income: {:.2}", text_data[4].0),
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Blue),
            ),
        ]),
        Spans::from(vec![
            Span::styled(
                format!("Most Expensive Month: {}, ", text_data[5].1),
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Rgb(205, 92, 92)),
            ),
            Span::styled(
                format!("Expense: {:.2}", text_data[5].0),
                Style::default().add_modifier(Modifier::BOLD).fg(Color::Red),
            ),
        ]),
    ];

    // * Goes through all tags provided and creates row for the table
    let rows = table_data.items.iter().map(|item| {
        let height = 1;
        let cells = item.iter().map(|c| Cell::from(c.to_string()));
        Row::new(cells).height(height as u16).bottom_margin(0)
    });

    let table_area = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Tags"))
        .widths(&[
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ])
        .highlight_symbol(">> ")
        .highlight_style(selected_style);

    let paragraph = Paragraph::new(text).style(
        Style::default()
            .bg(Color::Rgb(255, 255, 255))
            .fg(Color::Rgb(50, 205, 50)),
    );

    f.render_widget(paragraph, chunks[0]);
    f.render_stateful_widget(table_area, chunks[1], &mut table_data.state)
}
