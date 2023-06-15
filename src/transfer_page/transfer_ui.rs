use crate::page_handler::{TxTab, BACKGROUND, BLUE, GRAY, RED, TEXT};
use crate::tx_handler::TxData;
use crate::utility::{create_bolded_text, main_block, styled_block};
use ratatui::backend::Backend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

/// The function draws the Transfer page of the interface.
#[cfg(not(tarpaulin_include))]
pub fn transfer_ui<B: Backend>(
    f: &mut Frame<B>,
    transfer_data: &TxData,
    currently_selected: &TxTab,
) {
    let input_data = transfer_data.get_all_texts();
    let status_data = transfer_data.get_tx_status();
    let current_index = transfer_data.get_current_index();
    let size = f.size();

    // divide the terminal into various chunks to draw the interface. This is a vertical chunk
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(13),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Percentage(25),
            ]
            .as_ref(),
        )
        .split(size);

    // We will now cut down a single vertical chunk into multiple horizontal chunk.
    let first_chunk = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
            ]
            .as_ref(),
        )
        .split(chunks[1]);

    f.render_widget(main_block(), size);

    // This is the details of the Help widget
    let unmodified_help_text = "Press the respective keys to edit fields.
1: Date         Example: 2022-05-12, YYYY-MM-DD
2: TX details   Example: For Grocery, Salary
3: From Method  Example: Cash, Bank, Card
4: To Method    Example: Cash, Bank, Card
5: Amount       Example: 1000, 100+50
6: TX Tags      Example: Empty, Food, Car. Add a Comma for a new tag
S: Save the inputted data as a Transaction
H: Shows further detailed help info
Enter: Submit field and continue
Esc: Stop editing field
";

    let help_text = create_bolded_text(unmodified_help_text);

    let mut status_text = vec![];

    // * iter through the data in reverse mode because we want the latest status text
    // * to be at the top which is the final value of the vector.
    for i in status_data.iter().rev() {
        let (initial, rest) = i.split_once(':').unwrap();
        if !i.contains("Accepted") && !i.contains("Nothing") {
            status_text.push(Line::from(vec![
                Span::styled(
                    initial,
                    Style::default().fg(RED).add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!(":{rest}"), Style::default().fg(RED)),
            ]));
        } else {
            status_text.push(Line::from(vec![
                Span::styled(
                    initial,
                    Style::default().fg(BLUE).add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!(":{rest}"), Style::default().fg(BLUE)),
            ]));
        }
    }
    // We already fetched the data for each of these. Assign them now and then use them to load the widget
    let date_text = Line::from(format!("{} ", input_data[0]));

    let details_text = Line::from(format!("{} ", input_data[1]));

    let mut from_text = Line::from(format!("{} ", input_data[2]));

    let mut to_text = Line::from(format!("{} ", input_data[3]));

    let amount_text = Line::from(format!("{} ", input_data[4]));

    // * 5th index is the tx type which is not necessary for the transfer ui
    let mut tags_text = Line::from(format!("{} ", input_data[6]));

    match currently_selected {
        TxTab::FromMethod => {
            from_text = Line::from(vec![
                Span::from(format!("{} ", input_data[2])),
                Span::styled(input_data[7], Style::default().fg(GRAY)),
            ]);
        }
        TxTab::ToMethod => {
            to_text = Line::from(vec![
                Span::from(format!("{} ", input_data[3])),
                Span::styled(input_data[7], Style::default().fg(GRAY)),
            ]);
        }
        TxTab::Tags => {
            tags_text = Line::from(vec![
                Span::from(format!("{} ", input_data[6])),
                Span::styled(input_data[7], Style::default().fg(GRAY)),
            ]);
        }
        _ => {}
    }

    // creates the widgets to ready it for rendering
    let help_sec = Paragraph::new(help_text)
        .style(Style::default().bg(BACKGROUND).fg(TEXT))
        .block(styled_block("Help"))
        .alignment(Alignment::Left);

    let status_sec = Paragraph::new(status_text)
        .style(Style::default().bg(BACKGROUND).fg(TEXT))
        .block(styled_block("Status"))
        .alignment(Alignment::Left);

    let date_sec = Paragraph::new(date_text)
        .style(Style::default().bg(BACKGROUND).fg(TEXT))
        .block(styled_block("Date"))
        .alignment(Alignment::Left);

    let from_sec = Paragraph::new(from_text)
        .style(Style::default().bg(BACKGROUND).fg(TEXT))
        .block(styled_block("From Method"))
        .alignment(Alignment::Left);

    let to_sec = Paragraph::new(to_text)
        .style(Style::default().bg(BACKGROUND).fg(TEXT))
        .block(styled_block("To Method"))
        .alignment(Alignment::Left);

    let amount_sec = Paragraph::new(amount_text)
        .style(Style::default().bg(BACKGROUND).fg(TEXT))
        .block(styled_block("Amount"))
        .alignment(Alignment::Left);

    let details_sec = Paragraph::new(details_text)
        .style(Style::default().bg(BACKGROUND).fg(TEXT))
        .block(styled_block("Details"))
        .alignment(Alignment::Left);

    let tags_sec = Paragraph::new(tags_text)
        .style(Style::default().bg(BACKGROUND).fg(TEXT))
        .block(styled_block("Tags"))
        .alignment(Alignment::Left);

    // We will be adding a cursor/box based on which tab is selected.
    // This was created utilizing the tui-rs example named user_input.rs
    match currently_selected {
        TxTab::Date => f.set_cursor(
            first_chunk[0].x + current_index as u16 + 1,
            first_chunk[0].y + 1,
        ),
        TxTab::Details => f.set_cursor(chunks[2].x + current_index as u16 + 1, chunks[2].y + 1),
        TxTab::FromMethod => f.set_cursor(
            first_chunk[1].x + current_index as u16 + 1,
            first_chunk[1].y + 1,
        ),
        TxTab::ToMethod => f.set_cursor(
            first_chunk[2].x + current_index as u16 + 1,
            first_chunk[2].y + 1,
        ),
        // The text of this goes into the middle so couldn't find a better place to insert the input box
        TxTab::Amount => f.set_cursor(
            first_chunk[3].x + current_index as u16 + 1,
            first_chunk[3].y + 1,
        ),
        TxTab::Tags => f.set_cursor(
            first_chunk[4].x + current_index as u16 + 1,
            first_chunk[4].y + 1,
        ),
        _ => {}
    }

    // render the previously generated data into an interface
    f.render_widget(date_sec, first_chunk[0]);
    f.render_widget(details_sec, chunks[2]);
    f.render_widget(tags_sec, first_chunk[4]);

    f.render_widget(help_sec, chunks[0]);
    f.render_widget(status_sec, chunks[3]);

    f.render_widget(from_sec, first_chunk[1]);
    f.render_widget(to_sec, first_chunk[2]);

    f.render_widget(amount_sec, first_chunk[3]);
}
