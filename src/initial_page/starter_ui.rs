use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Alignment},
    style::{Color, Modifier, Style},
    text::{Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn starter_ui<B: Backend>(
    f: &mut Frame<B>,
    index: usize,
) {
    let size = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(8),
                Constraint::Min(5),
            ]
            .as_ref(),
        )
        .split(size);

    let block = Block::default().style(Style::default().bg(Color::White).fg(Color::Green));
    let create_block = |title| {
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::White).fg(Color::Green))
            .title(Span::styled(
                title,
                Style::default().add_modifier(Modifier::BOLD),
            ))
    };
    f.render_widget(block, size);
    let text = r#"  ____    _____  __  __
    |  _ \  | ____| \ \/ /
    | |_) | |  _|    \  / 
    |  _ <  | |___   /  \ 
    |_| \_\ |_____| /_/\_\
                            "#.to_string();
    let state = text.split("\n");
    let splitted = state.collect::<Vec<&str>>();
    let mut new_text = String::new();
    for line in splitted {
        let mut cu_index = 0;
        let mut total_to_add = 10;
        let mut target_index = index;
        for char in line.chars() {
            if cu_index == target_index && total_to_add != 0 {
                new_text.push(char);
                total_to_add -= 1;
                cu_index += 1;
                target_index += 1
            }
            else {
                new_text.push_str(" ");
                cu_index += 1;
            }
        }
        new_text.push_str("\n");
    }

    new_text.push_str("\n    Press Any Key To Continue");
    let second_text = "'Arrow Key' : Navigate
'A' : Add Transaction Page
'H' : Home Page
'D' : Delete Selected Transaction (Home Page)
'S': Save the inputted data as a Transaction (Add Transaction Page)
'Q' : Quit

Add Transaction Page:
'1': Edit Date          '4': Edit Amount
'2': Edit TX details    '3': Edit TX Method
'5': Edit TX Type
'Enter' or 'Esc': Submit/Stop Editing Field
";

    let paragraph = Paragraph::new(new_text)
        .style(Style::default().bg(Color::White).fg(Color::Green))
        .alignment(Alignment::Center);

    let paragraph_2 = Paragraph::new(second_text)
    .style(Style::default().bg(Color::White).fg(Color::Green))
    .block(create_block("Help"));

    f.render_widget(paragraph, chunks[0]);
    f.render_widget(paragraph_2, chunks[1]);
    
}