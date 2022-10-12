use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, Row, Table, Tabs, Paragraph},
    Frame,
};

use crate::home_page::{SelectedTab, TableData, TimeData};

// TODO show expense, income summary based on tags
// TODO show Biggest income and expense transaction
// TODO the month with the most income and expense

pub fn summary_ui<B: Backend>(f: &mut Frame<B>,) {
    let size = f.size();

    let test_table_data = vec![vec!["Food".to_string(), "99999".to_string(), "00000".to_string()], vec!["Car".to_string(), "0".to_string(), "110000".to_string()]];
    let mut table_data = TableData::new(test_table_data);
    // TODO change to a different color
    let normal_style = Style::default().bg(Color::LightBlue);

    let header_cells = ["Tag", "Total Income", "Total Expense"]
    .iter()
    .map(|h| Cell::from(*h).style(Style::default().fg(Color::Rgb(255, 255, 255))));

    let header = Row::new(header_cells)
    .style(normal_style)
    .height(1)
    .bottom_margin(0);

    // TODO chunk length
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Length(8), Constraint::Min(5)].as_ref())
        .split(size);

    let block = Block::default().style(
        Style::default()
            .bg(Color::Rgb(255, 255, 255))
            .fg(Color::Rgb(50, 205, 50)),
    );

    f.render_widget(block, size);

    let text = String::from("The biggest expense transaction happened on 01-01-2022 using Bank with the amount: 99999

The biggest income transaction happened on 01-01-2022 using Bank with the amount: 99999
    
The month with the highest expense was January of 2022

The month with the highest income was January of 2022");

    let rows = table_data.items.iter().map(|item| {
        let height = 1;
        let cells = item.iter().map(|c| Cell::from(c.to_string()));
        Row::new(cells).height(height as u16).bottom_margin(0)
    });

    let table_area = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Transactions"))
        .widths(&[
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ]).highlight_symbol(">> ");

    let paragraph = Paragraph::new(text)
    .style(
        Style::default()
            .bg(Color::Rgb(255, 255, 255))
            .fg(Color::Rgb(50, 205, 50)),
    );

    f.render_widget(paragraph, chunks[0]);
    f.render_stateful_widget(table_area, chunks[1], &mut table_data.state)

    
}