use tui::{
    backend::{Backend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn ui<B: Backend>(f: &mut Frame<B>) {
    let size = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(5)
        .constraints([Constraint::Length(8), Constraint::Length(3), Constraint::Length(3), Constraint::Percentage(20)].as_ref())
        .split(size);
    
    let block = Block::default().style(Style::default().bg(Color::White).fg(Color::Green));
    f.render_widget(block, size);

    let text = vec![
        Spans::from("Press the respective keys to edit fields. Press 'q' to cancel."),
        Spans::from("'1': Date         Example: 05-12-2022, DD-MM-YYYY"),
        Spans::from("'2': TX details   Example: For Grocery, Salary"),
        Spans::from("'3': TX Method    Example: Cash, Bank, Card"),
        Spans::from("'4': Amount       Example: 1000, 500"),
        Spans::from("'5': TX Type      Example: Income/Expense/I/E"),
    ];

    let text_2 = vec![
        Spans::from("Check status here")
    ];
    
    let text_3 = vec![
        Spans::from("Edit data here")
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
    let paragraph = Paragraph::new(text.clone())
    .style(Style::default().bg(Color::White).fg(Color::Green))
    .block(create_block("Edit"))
    .alignment(Alignment::Left);
    f.render_widget(paragraph, chunks[0]);

    let paragraph_2 = Paragraph::new(text_2.clone())
    .style(Style::default().bg(Color::White).fg(Color::Green))
    .block(create_block("Status"))
    .alignment(Alignment::Left);
    f.render_widget(paragraph_2, chunks[3]);

    let paragraph_4 = Paragraph::new(text_3.clone())
    .style(Style::default().bg(Color::White).fg(Color::Green))
    .block(create_block("Date"))
    .alignment(Alignment::Left);

    let paragraph_5 = Paragraph::new(text_3.clone())
    .style(Style::default().bg(Color::White).fg(Color::Green))
    .block(create_block("TX Method"))
    .alignment(Alignment::Left);

    let paragraph_6 = Paragraph::new(text_3.clone())
    .style(Style::default().bg(Color::White).fg(Color::Green))
    .block(create_block("Amount"))
    .alignment(Alignment::Left);

    let paragraph_7 = Paragraph::new(text_3.clone())
    .style(Style::default().bg(Color::White).fg(Color::Green))
    .block(create_block("TX Type"))
    .alignment(Alignment::Left);

    let paragraph_8 = Paragraph::new(text_3.clone())
    .style(Style::default().bg(Color::White).fg(Color::Green))
    .block(create_block("Details"))
    .alignment(Alignment::Left);
    f.render_widget(paragraph_8, chunks[2]);

    

    let another_chunk = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([Constraint::Length(20), Constraint::Length(20),
                Constraint::Length(20), Constraint::Length(20), 
                Constraint::Length(20)].as_ref())
    .split(chunks[1]);

    f.render_widget(paragraph_4, another_chunk[0]);
    f.render_widget(paragraph_5, another_chunk[1]);
    f.render_widget(paragraph_6, another_chunk[2]);
    f.render_widget(paragraph_7, another_chunk[3]);
    
}