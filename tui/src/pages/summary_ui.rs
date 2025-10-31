use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Cell, Row, Table};
use rex_app::conn::DbConn;
use rex_app::views::FullSummary;
use thousands::Separable;

use crate::page_handler::{IndexedData, SortingType, SummaryTab, TableData};
use crate::theme::Theme;
use crate::utility::{
    LerpState, create_tab, main_block, styled_block, styled_block_no_bottom, styled_block_no_top,
};

/// The function draws the Summary page of the interface.
pub fn summary_ui(
    f: &mut Frame,
    months: &IndexedData,
    years: &IndexedData,
    mode_selection: &IndexedData,
    table_data: &mut TableData,
    current_page: &SummaryTab,
    summary_hidden_mode: bool,
    summary_sort: &SortingType,
    lerp_state: &mut LerpState,
    full_summary: &FullSummary,
    theme: &Theme,
    conn: &mut DbConn,
) {
    let size = f.area();

    let tag_header = if let SortingType::Tags = summary_sort {
        "Tags ↑"
    } else {
        "Tags"
    };

    let total_income_header = if let SortingType::Income = summary_sort {
        "Total Income ↑"
    } else {
        "Total Income"
    };

    let total_expense_header = if let SortingType::Expense = summary_sort {
        "Total Expense ↑"
    } else {
        "Total Expense"
    };

    let mut table_headers = vec![
        tag_header,
        total_income_header,
        total_expense_header,
        "Income %",
        "Expense %",
    ];

    if mode_selection.index == 0 {
        table_headers.push("MoM Income %");
        table_headers.push("MoM Expense %");
    } else if mode_selection.index == 1 {
        table_headers.push("YoY Income %");
        table_headers.push("YoY Expense %");
    }

    table_headers.push("Borrow");
    table_headers.push("Lend");

    let header_cells = table_headers
        .into_iter()
        .map(|h| Cell::from(h).style(Style::default().fg(theme.background())));

    let mut method_headers = vec!["Method", "Total Income", "Total Expense"];

    if mode_selection.index != 0 {
        method_headers.push("Average Income");
        method_headers.push("Average Expense");
    }

    method_headers.push("Income %");
    method_headers.push("Expense %");

    if mode_selection.index == 0 {
        method_headers.push("MoM Income %");
        method_headers.push("MoM Expense %");
    } else if mode_selection.index == 1 {
        method_headers.push("YoY Income %");
        method_headers.push("YoY Expense %");
    }

    let method_header_cells = method_headers
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(theme.background())));

    let header = Row::new(header_cells)
        .style(Style::default().bg(theme.header()))
        .height(1)
        .bottom_margin(0);

    let method_header = Row::new(method_header_cells)
        .style(Style::default().bg(theme.header()))
        .height(1)
        .bottom_margin(0);

    let method_len = conn.get_tx_methods().len() as u16;

    let mut main_layout = Layout::default().direction(Direction::Vertical).margin(2);
    let mut summary_layout = Layout::default().direction(Direction::Horizontal);

    if summary_hidden_mode {
        main_layout = main_layout.constraints([
            Constraint::Length(method_len + 2),
            Constraint::Length(3),
            Constraint::Length(4),
            Constraint::Min(0),
        ]);
        summary_layout =
            summary_layout.constraints([Constraint::Percentage(50), Constraint::Percentage(50)]);
    } else {
        match mode_selection.index {
            0 => {
                main_layout = main_layout.constraints([
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(method_len + 2),
                    Constraint::Length(3),
                    Constraint::Length(4),
                    Constraint::Min(0),
                ]);
                summary_layout = summary_layout
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)]);
            }
            1 => {
                main_layout = main_layout.constraints([
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(method_len + 2),
                    Constraint::Length(3),
                    Constraint::Length(4),
                    Constraint::Min(0),
                ]);
                summary_layout = summary_layout
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)]);
            }
            2 => {
                main_layout = main_layout.constraints([
                    Constraint::Length(3),
                    Constraint::Length(method_len + 2),
                    Constraint::Length(3),
                    Constraint::Length(4),
                    Constraint::Min(0),
                ]);
                summary_layout = summary_layout
                    .constraints([Constraint::Percentage(100), Constraint::Percentage(50)]);
            }
            _ => {}
        }
    }

    let chunks = main_layout.split(size);
    let summary_chunk = if summary_hidden_mode {
        summary_layout.split(chunks[2])
    } else {
        summary_layout.split(chunks[5 - mode_selection.index])
    };

    f.render_widget(main_block(theme), size);

    let mut month_tab = create_tab(months, "Months", theme);

    let mut year_tab = create_tab(years, "Years", theme);

    let mut mode_selection_tab = create_tab(mode_selection, "Modes", theme);

    // Goes through all tags provided and creates row for the table
    let rows = table_data
        .items
        .iter()
        .enumerate()
        .map(|(row_index, item)| {
            let cells = item.iter().enumerate().map(|(index, c)| {
                if index == 0 {
                    return Cell::from(c.to_string());
                }

                let lerp_id = format!("summary_table_main:{index}:{row_index}");

                let Ok(parsed_num) = c.parse::<f64>() else {
                    if c == "∞" {
                        let new_c = lerp_state.lerp(&lerp_id, 0.0);
                        return Cell::from(format!("{new_c:.2}").separate_with_commas());
                    }

                    let symbol = if c.contains('↑') || c.contains('↓') {
                        c.chars().next()
                    } else {
                        None
                    };

                    if let Some(sym) = symbol {
                        let c = c.replace(sym, "");
                        if let Ok(parsed_num) = c.parse::<f64>() {
                            let new_c = lerp_state.lerp(&lerp_id, parsed_num);

                            return Cell::from(format!("{sym}{new_c:.2}").separate_with_commas());
                        }
                    } else {
                        log::info!("No symbol found. {c}");
                    }
                    return Cell::from(c.separate_with_commas());
                };

                let new_c = lerp_state.lerp(&lerp_id, parsed_num);

                if index == 7 && row_index == 0 && new_c != 0.0 {
                    log::info!("{new_c} {c} {parsed_num} {lerp_id}")
                }

                Cell::from(format!("{new_c:.2}").separate_with_commas())
            });
            Row::new(cells)
                .height(1)
                .bottom_margin(0)
                .style(Style::default().fg(theme.text()))
        });

    let table_width = if mode_selection.index == 2 {
        vec![
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(14),
            Constraint::Percentage(14),
            Constraint::Percentage(14),
            Constraint::Percentage(14),
            Constraint::Percentage(14),
        ]
    } else {
        vec![
            Constraint::Percentage(12),
            Constraint::Percentage(11),
            Constraint::Percentage(11),
            Constraint::Percentage(11),
            Constraint::Percentage(11),
            Constraint::Percentage(11),
            Constraint::Percentage(11),
            Constraint::Percentage(11),
            Constraint::Percentage(11),
        ]
    };

    let mut table_area = Table::new(rows, table_width)
        .header(header)
        .block(styled_block("Tags", theme))
        .style(Style::default().fg(theme.border()));

    let summary_rows_largest =
        full_summary
            .largest_array()
            .into_iter()
            .enumerate()
            .map(|(row_index, item)| {
                let cells = item.into_iter().enumerate().map(|(index, c)| {
                    let mut cell = if let Ok(parsed_num) = c.parse::<f64>() {
                        let lerp_id = format!("summary_table_largest:{index}:{row_index}");
                        let new_c = lerp_state.lerp(&lerp_id, parsed_num);

                        Cell::from(format!("{new_c:.2}").separate_with_commas())
                    } else {
                        Cell::from(c.separate_with_commas())
                    };

                    if index == 0 {
                        cell = cell.style(
                            Style::default()
                                .fg(theme.text())
                                .add_modifier(Modifier::BOLD),
                        );
                    }
                    cell
                });
                Row::new(cells)
                    .height(1)
                    .bottom_margin(0)
                    .style(Style::default().fg(theme.text()))
            });

    let summary_area_largest = Table::new(
        summary_rows_largest,
        [
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ],
    )
    .block(styled_block("", theme))
    .style(Style::default().fg(theme.border()));

    let summary_rows_peak =
        full_summary
            .peak_array()
            .into_iter()
            .enumerate()
            .map(|(row_index, item)| {
                let height = 1;
                let cells = item.into_iter().enumerate().map(|(index, c)| {
                    let mut cell = if let Ok(parsed_num) = c.parse::<f64>() {
                        let lerp_id = format!("summary_table_peak:{index}:{row_index}");
                        let new_c = lerp_state.lerp(&lerp_id, parsed_num);

                        let text = if index == 1 && row_index == 2 {
                            // Month checked value. No need float for this
                            let new_c = new_c as i64;
                            format!("{new_c}").separate_with_commas()
                        } else {
                            format!("{new_c:.2}").separate_with_commas()
                        };

                        Cell::from(text)
                    } else {
                        Cell::from(c.separate_with_commas())
                    };

                    if index == 0 {
                        cell = cell.style(
                            Style::default()
                                .fg(theme.text())
                                .add_modifier(Modifier::BOLD),
                        );
                    }
                    cell
                });
                Row::new(cells)
                    .height(height)
                    .bottom_margin(0)
                    .style(Style::default().fg(theme.text()))
            });

    let summary_area_peak = Table::new(
        summary_rows_peak,
        [
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ],
    )
    .block(styled_block("", theme))
    .style(Style::default().fg(theme.border()));

    let method_rows =
        full_summary
            .method_array()
            .into_iter()
            .enumerate()
            .map(|(row_index, item)| {
                let cells = item.into_iter().enumerate().map(|(index, c)| {
                    let mut cell = if let Ok(parsed_num) = c.parse::<f64>() {
                        let lerp_id = format!("method_table:{index}:{row_index}");
                        let new_c = lerp_state.lerp(&lerp_id, parsed_num);

                        Cell::from(format!("{new_c:.2}").separate_with_commas())
                    } else {
                        let symbol = if c.contains('↑') || c.contains('↓') {
                            c.chars().next()
                        } else {
                            None
                        };

                        if let Some(sym) = symbol {
                            let c = c.replace(sym, "");
                            if let Ok(parsed_num) = c.parse::<f64>() {
                                let lerp_id = format!("method_table:{index}:{row_index}");
                                let new_c = lerp_state.lerp(&lerp_id, parsed_num);

                                return Cell::from(
                                    format!("{sym}{new_c:.2}").separate_with_commas(),
                                );
                            }
                        }

                        if c == "∞" {
                            let lerp_id = format!("method_table:{index}:{row_index}");
                            lerp_state.lerp(&lerp_id, 0.0);
                        }
                        Cell::from(c.separate_with_commas())
                    };

                    if index == 0 {
                        cell = cell.style(
                            Style::default()
                                .fg(theme.text())
                                .add_modifier(Modifier::BOLD),
                        );
                    }
                    cell
                });
                Row::new(cells)
                    .height(1)
                    .bottom_margin(0)
                    .style(Style::default().fg(theme.text()))
            });

    let method_widths = if mode_selection.index == 2 {
        vec![
            Constraint::Percentage(10),
            Constraint::Percentage(14),
            Constraint::Percentage(14),
            Constraint::Percentage(14),
            Constraint::Percentage(14),
            Constraint::Percentage(16),
            Constraint::Percentage(16),
        ]
    } else if mode_selection.index == 1 {
        vec![
            Constraint::Percentage(10),
            Constraint::Percentage(12),
            Constraint::Percentage(12),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(12),
            Constraint::Percentage(12),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
        ]
    } else {
        vec![
            Constraint::Percentage(10),
            Constraint::Percentage(14),
            Constraint::Percentage(14),
            Constraint::Percentage(14),
            Constraint::Percentage(14),
            Constraint::Percentage(16),
            Constraint::Percentage(16),
        ]
    };

    let method_area = Table::new(method_rows, &method_widths)
        .header(method_header)
        .block(styled_block_no_bottom("", theme))
        .style(Style::default().fg(theme.border()));

    let net_row = full_summary
        .net_array()
        .into_iter()
        .enumerate()
        .map(|(row_index, item)| {
            let cells = item.iter().enumerate().map(|(index, c)| {
                let mut cell = if let Ok(parsed_num) = c.parse::<f64>() {
                    let lerp_id = format!("summary_rows_net:{index}:{row_index}");
                    let new_c = lerp_state.lerp(&lerp_id, parsed_num);

                    Cell::from(format!("{new_c:.2}").separate_with_commas())
                } else {
                    if c == "∞" {
                        let lerp_id = format!("summary_rows_net:{index}:{row_index}");
                        lerp_state.lerp(&lerp_id, 0.0);
                    }

                    let symbol = if c.contains('↑') || c.contains('↓') {
                        c.chars().next()
                    } else {
                        None
                    };

                    if let Some(sym) = symbol {
                        let c = c.replace(sym, "");
                        if let Ok(parsed_num) = c.parse::<f64>() {
                            let lerp_id = format!("summary_net_rows:{index}:{row_index}");
                            let new_c = lerp_state.lerp(&lerp_id, parsed_num);

                            return Cell::from(format!("{sym}{new_c:.2}").separate_with_commas());
                        }
                    }
                    Cell::from(c.separate_with_commas())
                };

                if index == 0 {
                    cell = cell.style(
                        Style::default()
                            .fg(theme.text())
                            .add_modifier(Modifier::BOLD),
                    );
                }
                cell
            });
            Row::new(cells)
                .height(1)
                .bottom_margin(0)
                .style(Style::default().fg(theme.text()))
        });

    let net_area = Table::new(net_row, &method_widths)
        .block(styled_block_no_top("", theme))
        .style(Style::default().fg(theme.border()));

    match current_page {
        // Previously added a black block to year and month widget if a value is not selected
        // Now we will turn that black block into green if a value is selected
        SummaryTab::Months => {
            month_tab = month_tab.highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(theme.selected()),
            );
        }

        SummaryTab::Years => {
            year_tab = year_tab.highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(theme.selected()),
            );
        }
        SummaryTab::ModeSelection => {
            mode_selection_tab = mode_selection_tab.highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(theme.selected()),
            );
        }
        SummaryTab::Table => {
            table_area = table_area
                .row_highlight_style(Style::default().bg(theme.selected()))
                .highlight_symbol(">> ");
        }
    }

    // Always keep some items rendered on the upper side of the table
    if let Some(index) = table_data.state.selected()
        && index > 7
    {
        *table_data.state.offset_mut() = index - 7;
    }

    f.render_widget(summary_area_largest, summary_chunk[1]);
    f.render_widget(summary_area_peak, summary_chunk[0]);

    if summary_hidden_mode {
        f.render_stateful_widget(table_area, chunks[3], &mut table_data.state);
        f.render_widget(net_area, chunks[1]);
        f.render_widget(method_area, chunks[0]);
    } else {
        f.render_widget(mode_selection_tab, chunks[0]);

        match mode_selection.index {
            0 => {
                f.render_widget(year_tab, chunks[1]);
                f.render_widget(month_tab, chunks[2]);
                f.render_stateful_widget(table_area, chunks[6], &mut table_data.state);
                f.render_widget(net_area, chunks[4]);
                f.render_widget(method_area, chunks[3]);
            }
            1 => {
                f.render_widget(year_tab, chunks[1]);
                f.render_stateful_widget(table_area, chunks[5], &mut table_data.state);
                f.render_widget(net_area, chunks[3]);
                f.render_widget(method_area, chunks[2]);
            }
            2 => {
                f.render_stateful_widget(table_area, chunks[4], &mut table_data.state);
                f.render_widget(net_area, chunks[2]);
                f.render_widget(method_area, chunks[1]);
            }
            _ => {}
        }
    }
}
