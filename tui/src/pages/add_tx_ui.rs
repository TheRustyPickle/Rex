use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Position};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Cell, Paragraph, Row, Table};
use rusqlite::Connection;
use thousands::Separable;

use crate::outputs::TxType;
use crate::page_handler::{BACKGROUND, BLUE, BOX, GRAY, HomeRow, LogType, RED, TEXT, TxTab};
use crate::pages::BALANCE_BOLD;
use crate::tx_handler::TxData;
use crate::utility::{LerpState, get_all_tx_methods, main_block, styled_block};

/// The function draws the Add Transaction page of the interface.
pub fn add_tx_ui(
    f: &mut Frame,
    balance: &mut [Vec<String>],
    add_tx_data: &TxData,
    add_tx_tab: &TxTab,
    width_data: &mut [Constraint],
    lerp_state: &mut LerpState,
    conn: &Connection,
) {
    let all_methods = get_all_tx_methods(conn);
    // Get the data to insert into the Status widget of this page

    let status_data = add_tx_data.get_tx_status();
    // Contains date, details, from method, to method, amount, tx type, tags.
    // Except to method, rest will be used for the widgets
    let input_data = add_tx_data.get_all_texts();
    // The index of the cursor position
    let current_index = add_tx_data.get_current_index();

    let size = f.area();

    let tx_type = add_tx_data.get_tx_type();

    let from_method_name = match tx_type {
        TxType::IncomeExpense => "TX Method",
        TxType::Transfer => "From Method",
    };

    // Divide the terminal into 3 parts vertically
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            // Balance change section
            Constraint::Length(5),
            // input chunk
            Constraint::Length(3),
            // details input chunk
            Constraint::Length(3),
            // status chunk
            Constraint::Percentage(100),
        ])
        .split(size);

    // Based on the tx type divide the first chunk into 5 or 6 parts horizontally
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
                .split(chunks[1]),
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
                .split(chunks[1]),
        }
    };

    // Creates border around the entire terminal
    f.render_widget(main_block(), size);

    let bal_data = balance.iter().map(|item| {
        let height = 1;

        let row_type = HomeRow::get_row(item);
        let mut index = 0;

        let cells = item.iter().map(|c| {
            let c = if row_type != HomeRow::TopRow && !["Balance", "Changes"].contains(&c.as_str())
            {
                // Changes row can contain arrow symbols
                let symbol = if c.contains('↑') || c.contains('↓') {
                    c.chars().next()
                } else {
                    None
                };

                // If loading was complete then this value is to be shown
                let actual_data: f64 = if row_type != HomeRow::Changes {
                    c.parse().unwrap()
                } else if let Some(sym) = symbol {
                    let without_symbol = c.replace(sym, "");
                    without_symbol.parse().unwrap()
                } else {
                    c.parse().unwrap()
                };

                index += 1;

                let lerp_id = format!("{row_type}:{index}");
                let to_show = lerp_state.lerp(&lerp_id, actual_data);

                // re-add the previously removed symbol if is the Changes row
                // Otherwise separate the number with commas
                if let Some(sym) = symbol {
                    format!("{sym}{to_show:.2}",).separate_with_commas()
                } else {
                    format!("{to_show:.2}").separate_with_commas()
                }
            } else {
                c.separate_with_commas()
            };

            if c.contains('↑') {
                Cell::from(c).style(Style::default().fg(BLUE))
            } else if c.contains('↓') {
                Cell::from(c).style(Style::default().fg(RED))
            } else if all_methods.contains(&c) || BALANCE_BOLD.contains(&c.as_str()) {
                Cell::from(c).style(Style::default().add_modifier(Modifier::BOLD))
            } else {
                Cell::from(c)
            }
        });
        Row::new(cells)
            .height(height as u16)
            .bottom_margin(0)
            .style(Style::default().fg(TEXT))
    });

    let balance_area = Table::new(bal_data, width_data.to_owned())
        .block(styled_block("Balance Change"))
        .style(Style::default().fg(BOX));

    let mut status_text = vec![];

    // Iter through the data in reverse mode because we want the latest status text
    // to be at the top which is the final value of the vector.
    for i in status_data.iter().rev() {
        let (initial, rest) = i.text.split_once(':').unwrap();

        match i.log_type {
            LogType::Info => {
                status_text.push(Line::from(vec![
                    Span::styled(
                        initial,
                        Style::default().fg(BLUE).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(format!(":{rest}"), Style::default().fg(BLUE)),
                ]));
            }
            LogType::Error => {
                status_text.push(Line::from(vec![
                    Span::styled(
                        initial,
                        Style::default().fg(RED).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(format!(":{rest}"), Style::default().fg(RED)),
                ]));
            }
        }
    }

    // We already fetched the data for each of these. Assign them now and then use them to load the widget
    let date_text = Line::from(format!("{} ", input_data[0]));

    let mut details_text = Line::from(format!("{} ", input_data[1]));

    let mut from_method_text = Line::from(format!("{} ", input_data[2]));

    let mut to_method_text = Line::from(format!("{} ", input_data[3]));

    let amount_text = Line::from(format!("{} ", input_data[4]));

    let mut tx_type_text = Line::from(format!("{} ", input_data[5]));

    let mut tags_text = Line::from(format!("{} ", input_data[6]));

    match add_tx_tab {
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
                Span::from(format!("{} ", input_data[5])),
                Span::styled(input_data[7], Style::default().fg(GRAY)),
            ]);
        }
        TxTab::TxType => {
            tx_type_text = Line::from(vec![
                Span::from(format!("{} ", input_data[5])),
                Span::styled(input_data[7], Style::default().fg(GRAY)),
            ]);
        }
        _ => {}
    }

    let status_sec = Paragraph::new(status_text)
        .style(Style::default().bg(BACKGROUND).fg(TEXT))
        .block(styled_block("Status"))
        .alignment(Alignment::Left);

    let date_sec = Paragraph::new(date_text)
        .style(Style::default().bg(BACKGROUND).fg(TEXT))
        .block(styled_block("Date"))
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
    match add_tx_tab {
        TxTab::Date => f.set_cursor_position(Position {
            x: input_chunk[0].x + current_index as u16 + 1,
            y: input_chunk[0].y + 1,
        }),
        TxTab::Details => f.set_cursor_position(Position {
            x: chunks[2].x + current_index as u16 + 1,
            y: chunks[2].y + 1,
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
        TxType::IncomeExpense => match add_tx_tab {
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
        TxType::Transfer => match add_tx_tab {
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

    f.render_widget(balance_area, chunks[0]);
    f.render_widget(details_sec, chunks[2]);
    f.render_widget(status_sec, chunks[3]);
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
}
