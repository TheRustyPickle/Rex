use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Position};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Cell, Paragraph, Row, Table};
use rex_app::conn::RecurrenceFrequency;
use thousands::Separable;

use crate::outputs::TxType;
use crate::page_handler::{LogType, TableData, TxTab};
use crate::theme::Theme;
use crate::tx_handler::TxData;
use crate::utility::{LerpState, main_block, styled_block};

pub const RECURRING_TABLE_ID: &str = "recurring_table_row";

pub fn recurring_ui(
    f: &mut Frame,
    recurring_data: &TxData,
    recurring_tab: &TxTab,
    recurring_table: &mut TableData,
    lerp_state: &mut LerpState,
    theme: &Theme,
) {
    let status_data = recurring_data.get_tx_status();
    // date, details, from method, to method, amount, tx type, tags, autofill
    let input_data = recurring_data.get_all_texts();
    // frequency, recur interval, recur value, recur month, end date
    let recur_data = recurring_data.get_recurring_texts();
    // The index of the cursor position
    let current_index = recurring_data.get_current_index();

    let tx_type = recurring_data.get_tx_type();
    let frequency = recurring_data.get_frequency();

    let size = f.area();

    let from_method_name = match tx_type {
        TxType::IncomeExpense => "TX Method",
        TxType::Transfer => "From Method",
    };

    let tx_count = recurring_table.items.len();
    let lerp_id = "recurring_count";
    let lerp_tx_count = lerp_state.lerp(lerp_id, tx_count as f64, None) as i64;

    let lerp_row = lerp_state.lerp(RECURRING_TABLE_ID, tx_count as f64, Some(0.50)) as usize;

    let table_name = format!("Recurring Transactions: {lerp_tx_count}");

    let header_cells = [
        "Next Date",
        "Frequency",
        "Details",
        "Method",
        "Amount",
        "Type",
        "Tags",
        "Paused",
    ]
    .iter()
    .map(|h| Cell::from(*h).style(Style::default().fg(theme.background())));

    let header = Row::new(header_cells)
        .style(Style::default().bg(theme.header()))
        .height(1)
        .bottom_margin(0);

    let rows = recurring_table.items.iter().take(lerp_row).map(|item| {
        let height = 1;
        let cells = item.iter().enumerate().map(|(index, c)| {
            // Index 4 is the Amount column - the only one that should get thousands
            // separators
            if index == 4 {
                Cell::from(c.separate_with_commas())
            } else {
                Cell::from(c.clone())
            }
        });
        Row::new(cells)
            .height(height as u16)
            .bottom_margin(0)
            .style(Style::default().bg(theme.background()).fg(theme.text()))
    });

    let mut table_area = Table::new(
        rows,
        [
            Constraint::Percentage(12),
            Constraint::Percentage(13),
            Constraint::Percentage(25),
            Constraint::Percentage(13),
            Constraint::Percentage(11),
            Constraint::Percentage(8),
            Constraint::Percentage(11),
            Constraint::Percentage(7),
        ],
    )
    .header(header)
    .block(styled_block(&table_name, theme));

    // Divide the terminal into 5 parts vertically
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            // core tx fields
            Constraint::Length(3),
            // recurrence schedule fields
            Constraint::Length(3),
            // details input chunk
            Constraint::Length(3),
            // status chunk
            Constraint::Length(10),
            // recurring rules list chunk
            Constraint::Min(0),
        ])
        .split(size);

    // Same field set/order as the Add Transaction page, plus Tags
    let core_chunk = match tx_type {
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
    };

    // Number of recurrence boxes to render: frequency and interval are always
    // shown, then the frequency decides whether a value, a month and the end date
    // follow
    let recur_box_count = match frequency {
        RecurrenceFrequency::Daily => 3,
        RecurrenceFrequency::Weekly | RecurrenceFrequency::Monthly => 4,
        RecurrenceFrequency::Yearly => 5,
    };

    let recur_percent = 100 / recur_box_count;
    let recur_chunk = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Percentage(recur_percent);
            recur_box_count as usize
        ])
        .split(chunks[1]);

    f.render_widget(main_block(theme), size);

    let mut status_text = vec![];

    for i in status_data.iter().rev() {
        let (initial, rest) = i.text.split_once(':').unwrap();

        match i.log_type {
            LogType::Info => {
                status_text.push(Line::from(vec![
                    Span::styled(
                        initial,
                        Style::default()
                            .fg(theme.positive())
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(format!(":{rest}"), Style::default().fg(theme.positive())),
                ]));
            }
            LogType::Error => {
                status_text.push(Line::from(vec![
                    Span::styled(
                        initial,
                        Style::default()
                            .fg(theme.negative())
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(format!(":{rest}"), Style::default().fg(theme.negative())),
                ]));
            }
        }
    }

    let date_text = Line::from(format!("{} ", input_data[0]));
    let mut details_text = Line::from(format!("{} ", input_data[1]));
    let mut from_method_text = Line::from(format!("{} ", input_data[2]));
    let mut to_method_text = Line::from(format!("{} ", input_data[3]));
    let amount_text = Line::from(format!("{} ", input_data[4]));
    let mut tx_type_text = Line::from(format!("{} ", input_data[5]));
    let mut tags_text = Line::from(format!("{} ", input_data[6]));

    let mut frequency_text = Line::from(format!("{} ", recur_data[0]));
    let recur_interval_text = Line::from(format!("{} ", recur_data[1]));
    let mut recur_value_text = Line::from(format!("{} ", recur_data[2]));
    let mut recur_month_text = Line::from(format!("{} ", recur_data[3]));
    let end_date_text = Line::from(format!("{} ", recur_data[4]));

    match recurring_tab {
        TxTab::Details => {
            details_text = Line::from(vec![
                Span::from(format!("{} ", input_data[1])),
                Span::styled(input_data[7], Style::default().fg(theme.autocomplete())),
            ]);
        }
        TxTab::FromMethod => {
            from_method_text = Line::from(vec![
                Span::from(format!("{} ", input_data[2])),
                Span::styled(input_data[7], Style::default().fg(theme.autocomplete())),
            ]);
        }
        TxTab::ToMethod => {
            to_method_text = Line::from(vec![
                Span::from(format!("{} ", input_data[3])),
                Span::styled(input_data[7], Style::default().fg(theme.autocomplete())),
            ]);
        }
        TxTab::Tags => {
            tags_text = Line::from(vec![
                Span::from(format!("{} ", input_data[6])),
                Span::styled(input_data[7], Style::default().fg(theme.autocomplete())),
            ]);
        }
        TxTab::Frequency => {
            frequency_text = Line::from(vec![
                Span::from(format!("{} ", recur_data[0])),
                Span::styled(input_data[7], Style::default().fg(theme.autocomplete())),
            ]);
        }
        TxTab::RecurValue => {
            recur_value_text = Line::from(vec![
                Span::from(format!("{} ", recur_data[2])),
                Span::styled(input_data[7], Style::default().fg(theme.autocomplete())),
            ]);
        }
        TxTab::RecurMonth => {
            recur_month_text = Line::from(vec![
                Span::from(format!("{} ", recur_data[3])),
                Span::styled(input_data[7], Style::default().fg(theme.autocomplete())),
            ]);
        }
        TxTab::TxType => {
            tx_type_text = Line::from(vec![
                Span::from(format!("{} ", input_data[5])),
                Span::styled(input_data[7], Style::default().fg(theme.autocomplete())),
            ]);
        }
        _ => {}
    }

    let status_sec = Paragraph::new(status_text)
        .style(Style::default().bg(theme.background()).fg(theme.text()))
        .block(styled_block("Status", theme))
        .alignment(Alignment::Left);

    let date_sec = Paragraph::new(date_text)
        .style(Style::default().bg(theme.background()).fg(theme.text()))
        .block(styled_block("Start Date", theme))
        .alignment(Alignment::Left);

    let from_method_sec = Paragraph::new(from_method_text)
        .style(Style::default().bg(theme.background()).fg(theme.text()))
        .block(styled_block(from_method_name, theme))
        .alignment(Alignment::Left);

    let to_method_sec = Paragraph::new(to_method_text)
        .style(Style::default().bg(theme.background()).fg(theme.text()))
        .block(styled_block("To Method", theme))
        .alignment(Alignment::Left);

    let amount_sec = Paragraph::new(amount_text)
        .style(Style::default().bg(theme.background()).fg(theme.text()))
        .block(styled_block("Amount", theme))
        .alignment(Alignment::Left);

    let tx_type_sec = Paragraph::new(tx_type_text)
        .style(Style::default().bg(theme.background()).fg(theme.text()))
        .block(styled_block("TX Type", theme))
        .alignment(Alignment::Left);

    let details_sec = Paragraph::new(details_text)
        .style(Style::default().bg(theme.background()).fg(theme.text()))
        .block(styled_block("Details", theme))
        .alignment(Alignment::Left);

    let tags_sec = Paragraph::new(tags_text)
        .style(Style::default().bg(theme.background()).fg(theme.text()))
        .block(styled_block("Tags", theme))
        .alignment(Alignment::Left);

    let frequency_sec = Paragraph::new(frequency_text)
        .style(Style::default().bg(theme.background()).fg(theme.text()))
        .block(styled_block("Frequency", theme))
        .alignment(Alignment::Left);

    let recur_interval_title = match frequency {
        RecurrenceFrequency::Daily => "Every N Days",
        RecurrenceFrequency::Weekly => "Every N Weeks",
        RecurrenceFrequency::Monthly => "Every N Months",
        RecurrenceFrequency::Yearly => "Every N Years",
    };

    let recur_interval_sec = Paragraph::new(recur_interval_text)
        .style(Style::default().bg(theme.background()).fg(theme.text()))
        .block(styled_block(recur_interval_title, theme))
        .alignment(Alignment::Left);

    let recur_value_title = match frequency {
        RecurrenceFrequency::Weekly => "Day of Week",
        _ => "Day of Month",
    };

    let recur_value_sec = Paragraph::new(recur_value_text)
        .style(Style::default().bg(theme.background()).fg(theme.text()))
        .block(styled_block(recur_value_title, theme))
        .alignment(Alignment::Left);

    let recur_month_sec = Paragraph::new(recur_month_text)
        .style(Style::default().bg(theme.background()).fg(theme.text()))
        .block(styled_block("Month", theme))
        .alignment(Alignment::Left);

    let end_date_sec = Paragraph::new(end_date_text)
        .style(Style::default().bg(theme.background()).fg(theme.text()))
        .block(styled_block("End Date (Optional)", theme))
        .alignment(Alignment::Left);

    match recurring_tab {
        TxTab::Date => f.set_cursor_position(Position {
            x: core_chunk[0].x + current_index as u16 + 1,
            y: core_chunk[0].y + 1,
        }),
        TxTab::Details => f.set_cursor_position(Position {
            x: chunks[2].x + current_index as u16 + 1,
            y: chunks[2].y + 1,
        }),
        TxTab::TxType => f.set_cursor_position(Position {
            x: core_chunk[1].x + current_index as u16 + 1,
            y: core_chunk[1].y + 1,
        }),
        TxTab::FromMethod => f.set_cursor_position(Position {
            x: core_chunk[2].x + current_index as u16 + 1,
            y: core_chunk[2].y + 1,
        }),
        _ => {}
    }

    match tx_type {
        TxType::IncomeExpense => match recurring_tab {
            TxTab::Amount => f.set_cursor_position(Position {
                x: core_chunk[3].x + current_index as u16 + 1,
                y: core_chunk[3].y + 1,
            }),
            TxTab::Tags => f.set_cursor_position(Position {
                x: core_chunk[4].x + current_index as u16 + 1,
                y: core_chunk[4].y + 1,
            }),
            _ => {}
        },
        TxType::Transfer => match recurring_tab {
            TxTab::ToMethod => f.set_cursor_position(Position {
                x: core_chunk[3].x + current_index as u16 + 1,
                y: core_chunk[3].y + 1,
            }),
            TxTab::Amount => f.set_cursor_position(Position {
                x: core_chunk[4].x + current_index as u16 + 1,
                y: core_chunk[4].y + 1,
            }),
            TxTab::Tags => f.set_cursor_position(Position {
                x: core_chunk[5].x + current_index as u16 + 1,
                y: core_chunk[5].y + 1,
            }),
            _ => {}
        },
    }

    match recurring_tab {
        TxTab::Frequency => f.set_cursor_position(Position {
            x: recur_chunk[0].x + current_index as u16 + 1,
            y: recur_chunk[0].y + 1,
        }),
        TxTab::RecurInterval => f.set_cursor_position(Position {
            x: recur_chunk[1].x + current_index as u16 + 1,
            y: recur_chunk[1].y + 1,
        }),
        TxTab::RecurValue => f.set_cursor_position(Position {
            x: recur_chunk[2].x + current_index as u16 + 1,
            y: recur_chunk[2].y + 1,
        }),
        TxTab::RecurMonth if recur_box_count == 5 => f.set_cursor_position(Position {
            x: recur_chunk[3].x + current_index as u16 + 1,
            y: recur_chunk[3].y + 1,
        }),
        TxTab::EndDate => {
            let end_date_index = (recur_box_count - 1) as usize;
            f.set_cursor_position(Position {
                x: recur_chunk[end_date_index].x + current_index as u16 + 1,
                y: recur_chunk[end_date_index].y + 1,
            });
        }
        _ => {}
    }

    f.render_widget(details_sec, chunks[2]);
    f.render_widget(status_sec, chunks[3]);
    f.render_widget(date_sec, core_chunk[0]);
    f.render_widget(tx_type_sec, core_chunk[1]);
    f.render_widget(from_method_sec, core_chunk[2]);

    match tx_type {
        TxType::IncomeExpense => {
            f.render_widget(amount_sec, core_chunk[3]);
            f.render_widget(tags_sec, core_chunk[4]);
        }
        TxType::Transfer => {
            f.render_widget(to_method_sec, core_chunk[3]);
            f.render_widget(amount_sec, core_chunk[4]);
            f.render_widget(tags_sec, core_chunk[5]);
        }
    }

    f.render_widget(frequency_sec, recur_chunk[0]);
    f.render_widget(recur_interval_sec, recur_chunk[1]);

    match frequency {
        RecurrenceFrequency::Daily => {
            f.render_widget(end_date_sec, recur_chunk[2]);
        }
        RecurrenceFrequency::Weekly | RecurrenceFrequency::Monthly => {
            f.render_widget(recur_value_sec, recur_chunk[2]);
            f.render_widget(end_date_sec, recur_chunk[3]);
        }
        RecurrenceFrequency::Yearly => {
            f.render_widget(recur_value_sec, recur_chunk[2]);
            f.render_widget(recur_month_sec, recur_chunk[3]);
            f.render_widget(end_date_sec, recur_chunk[4]);
        }
    }

    if recurring_table.state.selected().is_some() {
        table_area = table_area.highlight_symbol(">> ");

        let add_modifier = theme.add_reverse_modifier();

        let mut style = Style::default();

        if add_modifier {
            style = style.fg(theme.selected()).add_modifier(Modifier::REVERSED);
        } else {
            style = style.bg(theme.selected());
        }

        table_area = table_area.row_highlight_style(style);
    }

    if let Some(index) = recurring_table.state.selected()
        && index > 10
    {
        *recurring_table.state.offset_mut() = index - 10;
    }

    f.render_stateful_widget(table_area, chunks[4], &mut recurring_table.state);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn render(data: &TxData, tab: &TxTab) -> String {
        let (width, height) = (140, 40);
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        let theme = Theme::new_index(0);
        let mut table = TableData::new(Vec::new());
        let mut lerp = LerpState::new(1.0);

        terminal
            .draw(|f| recurring_ui(f, data, tab, &mut table, &mut lerp, &theme))
            .unwrap();

        let buffer = terminal.backend().buffer().clone();
        (0..height)
            .map(|y| {
                (0..width)
                    .map(|x| buffer[(x, y)].symbol().to_string())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn suggestion_shows_next_to_the_selected_recurrence_field_only() {
        let mut data = TxData::new_empty();
        // Yearly is the only frequency that shows all three boxes at once
        data.frequency = "y".to_string();
        data.recur_value = "mon".to_string();
        data.recur_month = "sep".to_string();
        data.autofill = "SUGGESTION".to_string();

        for tab in [TxTab::Frequency, TxTab::RecurValue, TxTab::RecurMonth] {
            let screen = render(&data, &tab);
            assert_eq!(
                screen.matches("SUGGESTION").count(),
                1,
                "exactly one box should show the suggestion for {tab:?}"
            );
        }

        let screen = render(&data, &TxTab::Nothing);
        assert!(!screen.contains("SUGGESTION"));
    }
}
