use crate::home_page::TransferTab;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// The UI functions that draws the Add Transaction page of the interface.
/// Takes arguments for user inputted data, status page data to process the details and turns them into
/// the the interface.
///
/// - input_data : Contains all the data for all field that has been inserted by the user so far for the transaction
///
/// Example input_data : `["2020-10-10", "", "", "", "Expense"]`
/// - cu_selected : For verifying the current selected widget to add a block box
/// - status_data : Contains all the String to push into the Status widget

pub fn transfer_ui_func<B: Backend>(
    f: &mut Frame<B>,
    input_data: Vec<&str>,
    cu_selected: &TransferTab,
    status_data: &Vec<String>,
) {
    let size = f.size();

    // divide the terminal into various chunks to draw the interface.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(5),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Percentage(25),
            ]
            .as_ref(),
        )
        .split(size);

    // This is a vertical chunk. We will basically be using this to divide the chunk[1]
    // into another 4 chunks or 4 widgets
    let first_chunk = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(15), Constraint::Percentage(50)].as_ref())
        .split(chunks[1]);

    let second_chunk = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(40),
                Constraint::Percentage(20),
                Constraint::Percentage(40),
            ]
            .as_ref(),
        )
        .split(chunks[2]);

    let third_chunk = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(30),
                Constraint::Percentage(40),
                Constraint::Percentage(30),
            ]
            .as_ref(),
        )
        .split(chunks[3]);

    let block = Block::default().style(
        Style::default()
            .bg(Color::Rgb(255, 255, 255))
            .fg(Color::Rgb(50, 205, 50)),
    );
    f.render_widget(block, size);

    // This is the details of the Help widget
    let help_text = vec![Spans::from("To Be Added")];

    let mut status_text = vec![];

    // iter through the data in reverse mode because we want the latest status text
    // to be at the top which is the final value of the vector.
    for i in status_data.iter().rev() {
        if i.contains("Accepted") == false && i.contains("Nothing") == false {
            status_text.push(Spans::from(Span::styled(
                i,
                Style::default().fg(Color::Red),
            )));
        } else {
            status_text.push(Spans::from(Span::styled(
                i,
                Style::default().fg(Color::Blue),
            )));
        }
    }

    // We got all these data from the run_app function already so just assign them
    let date_text = vec![Spans::from(input_data[0])];

    let details_text = vec![Spans::from(input_data[1])];

    let from_text = vec![Spans::from(input_data[2])];

    let to_text = vec![Spans::from(input_data[3])];

    let amount_text = vec![Spans::from(input_data[4])];

    let arrow_text = vec![Spans::from(""), Spans::from("➞ ➞ ➞")];

    let create_block = |title| {
        Block::default()
            .borders(Borders::ALL)
            .style(
                Style::default()
                    .bg(Color::Rgb(255, 255, 255))
                    .fg(Color::Rgb(50, 205, 50)),
            )
            .title(Span::styled(
                title,
                Style::default().add_modifier(Modifier::BOLD),
            ))
    };

    // creates the widgets to ready it for rendering
    let help_sec = Paragraph::new(help_text)
        .style(
            Style::default()
                .bg(Color::Rgb(255, 255, 255))
                .fg(Color::Rgb(50, 205, 50)),
        )
        .block(create_block("Help"))
        .alignment(Alignment::Left);

    let status_sec = Paragraph::new(status_text)
        .style(
            Style::default()
                .bg(Color::Rgb(255, 255, 255))
                .fg(Color::Rgb(50, 205, 50)),
        )
        .block(create_block("Status"))
        .alignment(Alignment::Left);

    let date_sec = Paragraph::new(date_text)
        .style(
            Style::default()
                .bg(Color::Rgb(255, 255, 255))
                .fg(Color::Rgb(50, 205, 50)),
        )
        .block(create_block("Date"))
        .alignment(Alignment::Left);

    let from_sec = Paragraph::new(from_text)
        .style(
            Style::default()
                .bg(Color::Rgb(255, 255, 255))
                .fg(Color::Rgb(50, 205, 50)),
        )
        .block(create_block("From"))
        .alignment(Alignment::Left);

    let arrow_sec = Paragraph::new(arrow_text)
        .style(
            Style::default()
                .bg(Color::Rgb(255, 255, 255))
                .fg(Color::Rgb(50, 205, 50)),
        )
        .alignment(Alignment::Center);

    let to_sec = Paragraph::new(to_text)
        .style(
            Style::default()
                .bg(Color::Rgb(255, 255, 255))
                .fg(Color::Rgb(50, 205, 50)),
        )
        .block(create_block("To"))
        .alignment(Alignment::Left);

    let amount_sec = Paragraph::new(amount_text)
        .style(
            Style::default()
                .bg(Color::Rgb(255, 255, 255))
                .fg(Color::Rgb(50, 205, 50)),
        )
        .block(create_block("Amount"))
        .alignment(Alignment::Center);

    let details_sec = Paragraph::new(details_text)
        .style(
            Style::default()
                .bg(Color::Rgb(255, 255, 255))
                .fg(Color::Rgb(50, 205, 50)),
        )
        .block(create_block("Details"))
        .alignment(Alignment::Left);

    // We will be adding a cursor/box based on which tab is selected.
    // This was created utilizing the tui-rs example named user_input.rs
    match cu_selected {
        TransferTab::Date => f.set_cursor(
            first_chunk[0].x + input_data[0].len() as u16 + 1,
            first_chunk[0].y + 1,
        ),
        TransferTab::Details => f.set_cursor(
            first_chunk[1].x + input_data[1].len() as u16 + 1,
            first_chunk[1].y + 1,
        ),
        TransferTab::From => f.set_cursor(
            second_chunk[0].x + input_data[2].len() as u16 + 1,
            second_chunk[0].y + 1,
        ),
        TransferTab::To => f.set_cursor(
            second_chunk[2].x + input_data[3].len() as u16 + 1,
            second_chunk[2].y + 1,
        ),
        TransferTab::Amount => f.set_cursor(third_chunk[1].x + 1, third_chunk[1].y + 1),
        TransferTab::Nothing => {}
    }

    // render the previously generated data into an interface
    f.render_widget(details_sec, first_chunk[1]);
    f.render_widget(status_sec, chunks[4]);
    f.render_widget(help_sec, chunks[0]);
    f.render_widget(date_sec, first_chunk[0]);
    f.render_widget(from_sec, second_chunk[0]);
    f.render_widget(to_sec, second_chunk[2]);
    f.render_widget(arrow_sec, second_chunk[1]);
    f.render_widget(amount_sec, third_chunk[1]);
}
