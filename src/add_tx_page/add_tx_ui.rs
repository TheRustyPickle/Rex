use crate::page_handler::{TxTab, BACKGROUND, BLUE, RED, TEXT};
use crate::tx_handler::TxData;
use crate::utility::{create_bolded_text, main_block, styled_block};
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::Paragraph;
use tui::Frame;

/// The function draws the Add Transaction page of the interface.
#[cfg(not(tarpaulin_include))]
pub fn add_tx_ui<B: Backend>(f: &mut Frame<B>, add_tx_data: &TxData, currently_selected: &TxTab) {
    // get the data to insert into the Status widget of this page
    let status_data = add_tx_data.get_tx_status();
    // * Contains date, details, from method, to method, amount, tx type, tags.
    // Except to method, rest will be used for the widgets
    let input_data = add_tx_data.get_all_texts();
    // The index of the cursor position
    let current_index = add_tx_data.get_current_index();

    let size = f.size();

    // divide the terminal into 4 parts vertically
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                // helper chunk
                Constraint::Length(13),
                // input chunk
                Constraint::Length(3),
                // details input chunk
                Constraint::Length(3),
                // status chunk
                Constraint::Percentage(25),
            ]
            .as_ref(),
        )
        .split(size);

    // divide the second chunk into 5 parts horizontally
    // this chunk contains the input boxes take takes input
    let input_chunk = Layout::default()
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

    // creates border around the entire terminal
    f.render_widget(main_block(), size);

    // This is the details of the Help widget
    let unmodified_help_text = "Press the respective keys to edit fields.
1: Date         Example: 2022-05-12, YYYY-MM-DD
2: TX details   Example: For Grocery, Salary
3: TX Method    Example: Cash, Bank, Card
4: Amount       Example: 1000, 100+50, b - 100
5: TX Type      Example: Income/Expense/I/E
6: TX Tags      Example: Empty, Food, Car. Add a Comma for a new tag
S: Save the inputted data as a Transaction
H: Shows further detailed help info
Enter: Submit field and continue
Esc: Stop editing filed";

    // highlight texts before the first color to Bold
    let help_text = create_bolded_text(unmodified_help_text);

    let mut status_text = vec![];

    // * iter through the data in reverse mode because we want the latest status text
    // * to be at the top which is the final value of the vector.
    for i in status_data.iter().rev() {
        let (initial, rest) = i.split_once(':').unwrap();
        if !i.contains("Accepted") && !i.contains("Nothing") {
            status_text.push(Spans::from(vec![
                Span::styled(
                    initial,
                    Style::default().fg(RED).add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!(":{rest}"), Style::default().fg(RED)),
            ]));
        } else {
            status_text.push(Spans::from(vec![
                Span::styled(
                    initial,
                    Style::default().fg(BLUE).add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!(":{rest}"), Style::default().fg(BLUE)),
            ]));
        }
    }

    // We already fetched the data for each of these. Assign them now and then use them to load the widget
    let date_text = vec![Spans::from(input_data[0])];

    let details_text = vec![Spans::from(input_data[1])];

    let tx_method_text = vec![Spans::from(input_data[2])];

    let amount_text = vec![Spans::from(input_data[4])];

    let tx_type_text = vec![Spans::from(input_data[5])];

    let tags_text = vec![Spans::from(input_data[6])];

    // creates the widgets to ready it for rendering
    let help_sec = Paragraph::new(help_text.clone())
        .style(Style::default().bg(BACKGROUND).fg(TEXT))
        .block(styled_block("Help"))
        .alignment(Alignment::Left);

    let status_sec = Paragraph::new(status_text.clone())
        .style(Style::default().bg(BACKGROUND).fg(TEXT))
        .block(styled_block("Status"))
        .alignment(Alignment::Left);

    let date_sec = Paragraph::new(date_text.clone())
        .style(Style::default().bg(BACKGROUND).fg(TEXT))
        .block(styled_block("Date"))
        .alignment(Alignment::Left);

    let tx_method_sec = Paragraph::new(tx_method_text.clone())
        .style(Style::default().bg(BACKGROUND).fg(TEXT))
        .block(styled_block("TX Method"))
        .alignment(Alignment::Left);

    let amount_sec = Paragraph::new(amount_text.clone())
        .style(Style::default().bg(BACKGROUND).fg(TEXT))
        .block(styled_block("Amount"))
        .alignment(Alignment::Left);

    let tx_type_sec = Paragraph::new(tx_type_text.clone())
        .style(Style::default().bg(BACKGROUND).fg(TEXT))
        .block(styled_block("TX Type"))
        .alignment(Alignment::Left);

    let details_sec = Paragraph::new(details_text.clone())
        .style(Style::default().bg(BACKGROUND).fg(TEXT))
        .block(styled_block("Details"))
        .alignment(Alignment::Left);

    let tags_sec = Paragraph::new(tags_text.clone())
        .style(Style::default().bg(BACKGROUND).fg(TEXT))
        .block(styled_block("Tags"))
        .alignment(Alignment::Left);

    // We will be adding a cursor based on which tab is selected + the selected index.
    // This was created utilizing the tui-rs example named user_input.rs
    match currently_selected {
        TxTab::Date => f.set_cursor(
            input_chunk[0].x + current_index as u16 + 1,
            input_chunk[0].y + 1,
        ),
        TxTab::Details => f.set_cursor(chunks[2].x + current_index as u16 + 1, chunks[2].y + 1),
        TxTab::FromMethod => f.set_cursor(
            input_chunk[1].x + current_index as u16 + 1,
            input_chunk[1].y + 1,
        ),
        TxTab::Amount => f.set_cursor(
            input_chunk[2].x + current_index as u16 + 1,
            input_chunk[2].y + 1,
        ),
        TxTab::TxType => f.set_cursor(
            input_chunk[3].x + current_index as u16 + 1,
            input_chunk[3].y + 1,
        ),
        TxTab::Tags => f.set_cursor(
            input_chunk[4].x + current_index as u16 + 1,
            input_chunk[4].y + 1,
        ),
        _ => {}
    }

    // render the previously generated data into an interface
    f.render_widget(help_sec, chunks[0]);
    f.render_widget(details_sec, chunks[2]);
    f.render_widget(status_sec, chunks[3]);
    f.render_widget(date_sec, input_chunk[0]);
    f.render_widget(tx_method_sec, input_chunk[1]);
    f.render_widget(amount_sec, input_chunk[2]);
    f.render_widget(tx_type_sec, input_chunk[3]);
    f.render_widget(tags_sec, input_chunk[4]);
}
