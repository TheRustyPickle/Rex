use tui::{
    backend::{Backend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::ui_data_state::TxTab;

pub fn tx_ui<B: Backend>(f: &mut Frame<B>, input_data: Vec<&str>, cu_selected: &TxTab, status_data: &Vec<String>) {
    let size = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(5)
        .constraints([Constraint::Length(11), Constraint::Length(3), Constraint::Length(3), Constraint::Percentage(25)].as_ref())
        .split(size);

    let another_chunk = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([Constraint::Percentage(25), Constraint::Percentage(25),
                Constraint::Percentage(25), Constraint::Percentage(25)].as_ref())
    .split(chunks[1]);
    
    let block = Block::default().style(Style::default().bg(Color::White).fg(Color::Green));
    f.render_widget(block, size);

    let help_text = vec![
        Spans::from("Press the respective keys to edit fields."),
        Spans::from("'1': Date         Example: 2022-05-12, YYYY-MM-DD"),
        Spans::from("'2': TX details   Example: For Grocery, Salary"),
        Spans::from("'3': TX Method    Example: Cash, Bank, Card"),
        Spans::from("'4': Amount       Example: 1000, 500"),
        Spans::from("'5': TX Type      Example: Income/Expense/I/E"),
        Spans::from("'Enter': Submit/Stop Editing Field"),
        Spans::from("'Esc': Submit/Stop Editing Field"),
        Spans::from("'S': Save the inputted data as a Transaction"),
    ];

    let mut status_text = vec![];

    for i in status_data.iter().rev() {
        if i.contains("Accepted") == false && i.contains("Nothing") == false {
            status_text.push(Spans::from(Span::styled(i,Style::default().fg(Color::Red),)));
        }
        else {
            status_text.push(Spans::from(Span::styled(i,Style::default().fg(Color::Blue),)));
        }
    }
    
    let date_text = vec![
        Spans::from(input_data[0])
    ];

    let details_text = vec![
        Spans::from(input_data[1])
    ];

    let tx_method_text = vec![
        Spans::from(input_data[2])
    ];

    let amount_text = vec![
        Spans::from(input_data[3])
    ];

    let tx_type_text = vec![
        Spans::from(input_data[4])
    ];

    let create_block = |title| {
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::White).fg(Color::Green))
            .title(Span::styled(
                title,
                Style::default().add_modifier(Modifier::BOLD),
            ))
        };
    let help_sec = Paragraph::new(help_text.clone())
    .style(Style::default().bg(Color::White).fg(Color::Green))
    .block(create_block("Help"))
    .alignment(Alignment::Left);
    
    let status_sec = Paragraph::new(status_text.clone())
    .style(Style::default().bg(Color::White).fg(Color::Green))
    .block(create_block("Status"))
    .alignment(Alignment::Left);

    let date_sec = Paragraph::new(date_text.clone())
    .style(Style::default().bg(Color::White).fg(Color::Green))
    .block(create_block("Date"))
    .alignment(Alignment::Left);

    let tx_method_sec = Paragraph::new(tx_method_text.clone())
    .style(Style::default().bg(Color::White).fg(Color::Green))
    .block(create_block("TX Method"))
    .alignment(Alignment::Left);

    let amount_sec = Paragraph::new(amount_text.clone())
    .style(Style::default().bg(Color::White).fg(Color::Green))
    .block(create_block("Amount"))
    .alignment(Alignment::Left);

    let tx_type_sec = Paragraph::new(tx_type_text.clone())
    .style(Style::default().bg(Color::White).fg(Color::Green))
    .block(create_block("TX Type"))
    .alignment(Alignment::Left);

    let details_sec = Paragraph::new(details_text.clone())
    .style(Style::default().bg(Color::White).fg(Color::Green))
    .block(create_block("Details"))
    .alignment(Alignment::Left);
    
    match cu_selected {
        TxTab::Date => {
            f.set_cursor(another_chunk[0].x + input_data[0].len() as u16 + 1, 
            another_chunk[0].y + 1,)
        }
        TxTab::Details => {
            f.set_cursor(chunks[2].x + input_data[1].len() as u16 + 1, 
            chunks[2].y + 1,)
        }
        TxTab::TxMethod => {
            f.set_cursor(another_chunk[1].x + input_data[2].len() as u16 + 1, 
            another_chunk[1].y + 1,)
        }
        TxTab::Amount => {
            f.set_cursor(another_chunk[2].x + input_data[3].len() as u16 + 1, 
            another_chunk[2].y + 1,)
        }
        TxTab::TxType => {
            f.set_cursor(another_chunk[3].x + input_data[4].len() as u16 + 1, 
            another_chunk[3].y + 1,)
        }
        TxTab::Nothing => {}
    }
    
    f.render_widget(details_sec, chunks[2]);
    f.render_widget(status_sec, chunks[3]);
    f.render_widget(help_sec, chunks[0]);
    f.render_widget(date_sec, another_chunk[0]);
    f.render_widget(tx_method_sec, another_chunk[1]);
    f.render_widget(amount_sec, another_chunk[2]);
    f.render_widget(tx_type_sec, another_chunk[3]);
    
}