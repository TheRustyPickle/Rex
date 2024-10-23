use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Cell, Paragraph, Row, Table};
use ratatui::Frame;
use thousands::Separable;

use crate::outputs::TxType;
use crate::page_handler::{
    DateType, TableData, TxTab, BACKGROUND, BLUE, GRAY, HEADER, RED, SELECTED, TEXT,
};
use crate::tx_handler::TxData;
use crate::utility::{main_block, styled_block};

#[cfg(not(tarpaulin_include))]
pub fn search_ui(
    f: &mut Frame,
    search_data: &TxData,
    search_tab: &TxTab,
    search_table: &mut TableData,
    date_type: &DateType,
) {
    // get the data to insert into the Status widget of this page

    use ratatui::layout::Position;
    let status_data = search_data.get_tx_status();
    // Contains date, details, from method, to method, amount, tx type, tags.
    // Except to method, rest will be used for the widgets
    let input_data = search_data.get_all_texts();
    // The index of the cursor position
    let current_index = search_data.get_current_index();

    let selected_style_income = Style::default().fg(BLUE).add_modifier(Modifier::REVERSED);
    let selected_style_expense = Style::default().fg(RED).add_modifier(Modifier::REVERSED);

    let size = f.area();

    let tx_type = search_data.get_tx_type();

    let rows = search_table.items.iter().map(|item| {
        let height = 1;
        let cells = item.iter().map(|c| Cell::from(c.separate_with_commas()));
        Row::new(cells)
            .height(height as u16)
            .bottom_margin(0)
            .style(Style::default().bg(BACKGROUND).fg(TEXT))
    });

    let from_method_name = match tx_type {
        TxType::IncomeExpense => "TX Method",
        TxType::Transfer => "From Method",
    };

    let mut table_name = "Transactions".to_string();

    let date_name = match date_type {
        DateType::Exact => "Search by Exact Date",
        DateType::Monthly => "Search by Month",
        DateType::Yearly => "Search by Year",
    };

    if !search_table.items.is_empty() {
        table_name = format!("Transactions: {}", search_table.items.len());
    }

    let header_cells = ["Date", "Details", "TX Method", "Amount", "Type", "Tags"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(BACKGROUND)));

    let header = Row::new(header_cells)
        .style(Style::default().bg(HEADER))
        .height(1)
        .bottom_margin(0);

    // divide the terminal into 4 parts vertically
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            // input chunk
            Constraint::Length(3),
            // details input chunk
            Constraint::Length(3),
            // status chunk
            Constraint::Length(10),
            // Transaction list chunk,
            Constraint::Min(0),
        ])
        .split(size);

    // based on the tx type divide the first chunk into 5 or 6 parts horizontally
    // this chunk contains the input boxes take takes input
    let input_chunk = {
        match tx_type {
            TxType::IncomeExpense => Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                ])
                .split(chunks[0]),
            TxType::Transfer => Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(16),
                    Constraint::Percentage(16),
                    Constraint::Percentage(16),
                    Constraint::Percentage(16),
                    Constraint::Percentage(16),
                    Constraint::Percentage(20),
                ])
                .split(chunks[0]),
        }
    };

    // creates border around the entire terminal
    f.render_widget(main_block(), size);

    let mut table_area = Table::new(
        rows,
        [
            Constraint::Percentage(10),
            Constraint::Percentage(37),
            Constraint::Percentage(13),
            Constraint::Percentage(13),
            Constraint::Percentage(8),
            Constraint::Percentage(18),
        ],
    )
    .header(header)
    .block(styled_block(&table_name));

    let mut status_text = vec![];

    // iter through the data in reverse mode because we want the latest status text
    // to be at the top which is the final value of the vector.
    for i in status_data.iter().rev() {
        let (initial, rest) = i.split_once(':').unwrap();
        if !i.contains("Accepted") && !i.contains("Nothing") && !i.contains("Search: Found") {
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

    let mut details_text = Line::from(format!("{} ", input_data[1]));

    let mut from_method_text = Line::from(format!("{} ", input_data[2]));

    let mut to_method_text = Line::from(format!("{} ", input_data[3]));

    let amount_text = Line::from(format!("{} ", input_data[4]));

    let tx_type_text = Line::from(format!("{} ", input_data[5]));

    let mut tags_text = Line::from(format!("{} ", input_data[6]));

    match search_tab {
        TxTab::Details => {
            details_text = Line::from(vec![
                Span::from(format!("{} ", input_data[1])),
                Span::styled(input_data[7], Style::default().fg(GRAY)),
            ]);
        }
        TxTab::FromMethod => {
            from_method_text = Line::from(vec![
                Span::from(format!("{} ", input_data[2])),
                Span::styled(input_data[7], Style::default().fg(GRAY)),
            ]);
        }
        TxTab::ToMethod => {
            to_method_text = Line::from(vec![
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
    let status_sec = Paragraph::new(status_text)
        .style(Style::default().bg(BACKGROUND).fg(TEXT))
        .block(styled_block("Status"))
        .alignment(Alignment::Left);

    let date_sec = Paragraph::new(date_text)
        .style(Style::default().bg(BACKGROUND).fg(TEXT))
        .block(styled_block(date_name))
        .alignment(Alignment::Left);

    let from_method_sec = Paragraph::new(from_method_text)
        .style(Style::default().bg(BACKGROUND).fg(TEXT))
        .block(styled_block(from_method_name))
        .alignment(Alignment::Left);

    let to_method_sec = Paragraph::new(to_method_text)
        .style(Style::default().bg(BACKGROUND).fg(TEXT))
        .block(styled_block("To Method"))
        .alignment(Alignment::Left);

    let amount_sec = Paragraph::new(amount_text)
        .style(Style::default().bg(BACKGROUND).fg(TEXT))
        .block(styled_block("Amount"))
        .alignment(Alignment::Left);

    let tx_type_sec = Paragraph::new(tx_type_text)
        .style(Style::default().bg(BACKGROUND).fg(TEXT))
        .block(styled_block("TX Type"))
        .alignment(Alignment::Left);

    let details_sec = Paragraph::new(details_text)
        .style(Style::default().bg(BACKGROUND).fg(TEXT))
        .block(styled_block("Details"))
        .alignment(Alignment::Left);

    let tags_sec = Paragraph::new(tags_text)
        .style(Style::default().bg(BACKGROUND).fg(TEXT))
        .block(styled_block("Tags"))
        .alignment(Alignment::Left);

    // We will be adding a cursor based on which tab is selected + the selected index.
    // This was created utilizing the tui-rs example named user_input.rs
    match search_tab {
        TxTab::Date => f.set_cursor_position(Position {
            x: input_chunk[0].x + current_index as u16 + 1,
            y: input_chunk[0].y + 1,
        }),
        TxTab::Details => f.set_cursor_position(Position {
            x: chunks[1].x + current_index as u16 + 1,
            y: chunks[1].y + 1,
        }),
        TxTab::TxType => f.set_cursor_position(Position {
            x: input_chunk[1].x + current_index as u16 + 1,
            y: input_chunk[1].y + 1,
        }),
        TxTab::FromMethod => f.set_cursor_position(Position {
            x: input_chunk[2].x + current_index as u16 + 1,
            y: input_chunk[2].y + 1,
        }),
        _ => {}
    }

    match tx_type {
        TxType::IncomeExpense => match search_tab {
            TxTab::Amount => f.set_cursor_position(Position {
                x: input_chunk[3].x + current_index as u16 + 1,
                y: input_chunk[3].y + 1,
            }),

            TxTab::Tags => f.set_cursor_position(Position {
                x: input_chunk[4].x + current_index as u16 + 1,
                y: input_chunk[4].y + 1,
            }),
            _ => {}
        },
        TxType::Transfer => match search_tab {
            TxTab::ToMethod => f.set_cursor_position(Position {
                x: input_chunk[3].x + current_index as u16 + 1,
                y: input_chunk[3].y + 1,
            }),
            TxTab::Amount => f.set_cursor_position(Position {
                x: input_chunk[4].x + current_index as u16 + 1,
                y: input_chunk[4].y + 1,
            }),
            TxTab::Tags => f.set_cursor_position(Position {
                x: input_chunk[5].x + current_index as u16 + 1,
                y: input_chunk[5].y + 1,
            }),
            _ => {}
        },
    }

    if let Some(a) = search_table.state.selected() {
        table_area = table_area.highlight_symbol(">> ");
        if search_table.items[a][4] == "Expense" {
            table_area = table_area.row_highlight_style(selected_style_expense);
        } else if search_table.items[a][4] == "Income" {
            table_area = table_area.row_highlight_style(selected_style_income);
        } else if search_table.items[a][4] == "Transfer" {
            table_area = table_area.row_highlight_style(Style::default().bg(SELECTED));
        }
    }

    if let Some(index) = search_table.state.selected() {
        if index > 10 {
            *search_table.state.offset_mut() = index - 10;
        }
    }

    // render the previously generated data into an interface
    f.render_widget(details_sec, chunks[1]);
    f.render_widget(status_sec, chunks[2]);
    f.render_widget(date_sec, input_chunk[0]);
    f.render_widget(tx_type_sec, input_chunk[1]);
    f.render_widget(from_method_sec, input_chunk[2]);

    match tx_type {
        TxType::IncomeExpense => {
            f.render_widget(amount_sec, input_chunk[3]);
            f.render_widget(tags_sec, input_chunk[4]);
        }
        TxType::Transfer => {
            f.render_widget(to_method_sec, input_chunk[3]);
            f.render_widget(amount_sec, input_chunk[4]);
            f.render_widget(tags_sec, input_chunk[5]);
        }
    }

    f.render_stateful_widget(table_area, chunks[3], &mut search_table.state);
}
