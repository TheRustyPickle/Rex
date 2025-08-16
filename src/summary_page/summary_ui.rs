use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Cell, Row, Table};
use rusqlite::Connection;
use thousands::Separable;

use crate::page_handler::{
    BACKGROUND, BOX, HEADER, IndexedData, SELECTED, SortingType, SummaryTab, TEXT, TableData,
};
use crate::summary_page::SummaryData;
use crate::utility::{LerpState, create_tab, get_all_tx_methods, main_block, styled_block};

/// The function draws the Summary page of the interface.
#[cfg(not(tarpaulin_include))]
pub fn summary_ui(
    f: &mut Frame,
    months: &IndexedData,
    years: &IndexedData,
    mode_selection: &IndexedData,
    summary_data: &SummaryData,
    table_data: &mut TableData,
    current_page: &SummaryTab,
    summary_hidden_mode: bool,
    summary_sort: &SortingType,
    lerp_state: &mut LerpState,
    conn: &Connection,
) {
    let (summary_data_1, summary_data_2, summary_data_3, summary_data_4, method_data) =
        summary_data.get_tx_data(mode_selection, months.index, years.index, conn);

    let mut summary_table_1 = TableData::new(summary_data_1);
    let mut summary_table_2 = TableData::new(summary_data_2);
    let mut summary_table_3 = TableData::new(summary_data_3);
    let mut summary_table_4 = TableData::new(summary_data_4);
    let mut method_table = TableData::new(method_data);

    let size = f.area();

    let tag_header = if let SortingType::ByTags = summary_sort {
        "Tags ↓"
    } else {
        "Tags"
    };

    let total_income_header = if let SortingType::ByIncome = summary_sort {
        "Total Income ↓"
    } else {
        "Total Income"
    };

    let total_expense_header = if let SortingType::ByExpense = summary_sort {
        "Total Expense ↓"
    } else {
        "Total Expense"
    };

    let header_cells = [
        tag_header,
        total_income_header,
        total_expense_header,
        "Income %",
        "Expense %",
    ]
    .into_iter()
    .map(|h| Cell::from(h).style(Style::default().fg(BACKGROUND)));

    let method_header_cells = [
        "Method",
        "Total Income",
        "Total Expense",
        "Income %",
        "Expense %",
        "Average Income",
        "Average Expense",
    ]
    .iter()
    .map(|h| Cell::from(*h).style(Style::default().fg(BACKGROUND)));

    let header = Row::new(header_cells)
        .style(Style::default().bg(HEADER))
        .height(1)
        .bottom_margin(0);

    let method_header = Row::new(method_header_cells)
        .style(Style::default().bg(HEADER))
        .height(1)
        .bottom_margin(0);

    let method_len = get_all_tx_methods(conn).len() as u16;

    let mut main_layout = Layout::default().direction(Direction::Vertical).margin(2);
    let mut summary_layout = Layout::default().direction(Direction::Horizontal);

    if summary_hidden_mode {
        main_layout = main_layout.constraints([
            Constraint::Length(method_len + 3),
            Constraint::Length(9),
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
                    Constraint::Length(method_len + 3),
                    Constraint::Length(9),
                    Constraint::Min(0),
                ]);
                summary_layout = summary_layout
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)]);
            }
            1 => {
                main_layout = main_layout.constraints([
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(method_len + 3),
                    Constraint::Length(9),
                    Constraint::Min(0),
                ]);
                summary_layout = summary_layout
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)]);
            }
            2 => {
                main_layout = main_layout.constraints([
                    Constraint::Length(3),
                    Constraint::Length(method_len + 3),
                    Constraint::Length(9),
                    Constraint::Min(0),
                ]);
                summary_layout = summary_layout
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)]);
            }
            _ => {}
        }
    }

    let chunks = main_layout.split(size);
    let summary_chunk = if summary_hidden_mode {
        summary_layout.split(chunks[1])
    } else {
        summary_layout.split(chunks[4 - mode_selection.index])
    };

    let left_summary = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(summary_chunk[0]);

    let right_summary = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(summary_chunk[1]);

    f.render_widget(main_block(), size);

    let mut month_tab = create_tab(months, "Months");

    let mut year_tab = create_tab(years, "Years");

    let mut mode_selection_tab = create_tab(mode_selection, "Modes");

    // Goes through all tags provided and creates row for the table
    let rows = table_data
        .items
        .iter()
        .enumerate()
        .map(|(row_index, item)| {
            let cells = item.iter().enumerate().map(|(index, c)| {
                let Ok(parsed_num) = c.parse::<f64>() else {
                    return Cell::from(c.separate_with_commas());
                };

                let lerp_id = format!("summary_table_main:{index}:{row_index}");
                let new_c = lerp_state.lerp(&lerp_id, parsed_num);

                Cell::from(format!("{new_c:.2}").separate_with_commas())
            });
            Row::new(cells)
                .height(1)
                .bottom_margin(0)
                .style(Style::default().fg(TEXT))
        });

    let mut table_area = Table::new(
        rows,
        [
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ],
    )
    .header(header)
    .block(styled_block("Tags"))
    .style(Style::default().fg(BOX));

    let summary_rows_1 = summary_table_1
        .items
        .iter()
        .enumerate()
        .map(|(row_index, item)| {
            let cells = item.iter().enumerate().map(|(index, c)| {
                let mut cell = if let Ok(parsed_num) = c.parse::<f64>() {
                    let lerp_id = format!("summary_table_1:{index}:{row_index}");
                    let new_c = lerp_state.lerp(&lerp_id, parsed_num);

                    let text = if index == 2 {
                        // Total income/expense % column. Add % char manually
                        format!("{new_c:.2}%").separate_with_commas()
                    } else {
                        format!("{new_c:.2}").separate_with_commas()
                    };

                    Cell::from(text)
                } else {
                    Cell::from(c.separate_with_commas())
                };

                if index == 0 {
                    cell = cell.style(Style::default().fg(TEXT).add_modifier(Modifier::BOLD));
                }
                cell
            });
            Row::new(cells)
                .height(1)
                .bottom_margin(0)
                .style(Style::default().fg(TEXT))
        });

    let summary_area_1 = Table::new(
        summary_rows_1,
        [
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ],
    )
    .block(styled_block(""))
    .style(Style::default().fg(BOX));

    let summary_rows_2 = summary_table_2
        .items
        .iter()
        .enumerate()
        .map(|(row_index, item)| {
            let cells = item.iter().enumerate().map(|(index, c)| {
                let mut cell = if let Ok(parsed_num) = c.parse::<f64>() {
                    let lerp_id = format!("summary_table_2:{index}:{row_index}");
                    let new_c = lerp_state.lerp(&lerp_id, parsed_num);

                    Cell::from(format!("{new_c:.2}").separate_with_commas())
                } else {
                    Cell::from(c.separate_with_commas())
                };

                if index == 0 {
                    cell = cell.style(Style::default().fg(TEXT).add_modifier(Modifier::BOLD));
                }
                cell
            });
            Row::new(cells)
                .height(1)
                .bottom_margin(0)
                .style(Style::default().fg(TEXT))
        });

    let summary_area_2 = Table::new(
        summary_rows_2,
        [
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ],
    )
    .block(styled_block(""))
    .style(Style::default().fg(BOX));

    let summary_rows_3 = summary_table_3
        .items
        .iter()
        .enumerate()
        .map(|(row_index, item)| {
            let height = 1;
            let cells = item.iter().enumerate().map(|(index, c)| {
                let mut cell = if let Ok(parsed_num) = c.parse::<f64>() {
                    let lerp_id = format!("summary_table_3:{index}:{row_index}");
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
                    cell = cell.style(Style::default().fg(TEXT).add_modifier(Modifier::BOLD));
                }
                cell
            });
            Row::new(cells)
                .height(height as u16)
                .bottom_margin(0)
                .style(Style::default().fg(TEXT))
        });

    let summary_area_3 = Table::new(
        summary_rows_3,
        [
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ],
    )
    .block(styled_block(""))
    .style(Style::default().fg(BOX));

    let summary_rows_4 = summary_table_4
        .items
        .iter()
        .enumerate()
        .map(|(row_index, item)| {
            let cells = item.iter().enumerate().map(|(index, c)| {
                let mut cell = if let Ok(parsed_num) = c.parse::<f64>() {
                    let lerp_id = format!("summary_table_4:{index}:{row_index}");
                    let new_c = lerp_state.lerp(&lerp_id, parsed_num);

                    Cell::from(format!("{new_c:.2}").separate_with_commas())
                } else {
                    Cell::from(c.separate_with_commas())
                };

                if index == 0 {
                    cell = cell.style(Style::default().fg(TEXT).add_modifier(Modifier::BOLD));
                }
                cell
            });
            Row::new(cells)
                .height(1)
                .bottom_margin(0)
                .style(Style::default().fg(TEXT))
        });

    let summary_area_4 = Table::new(
        summary_rows_4,
        [
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ],
    )
    .block(styled_block(""))
    .style(Style::default().fg(BOX));

    let method_rows = method_table
        .items
        .iter()
        .enumerate()
        .map(|(row_index, item)| {
            let cells = item.iter().enumerate().map(|(index, c)| {
                let mut cell = if let Ok(parsed_num) = c.parse::<f64>() {
                    let lerp_id = format!("method_table:{index}:{row_index}");
                    let new_c = lerp_state.lerp(&lerp_id, parsed_num);

                    Cell::from(format!("{new_c:.2}").separate_with_commas())
                } else {
                    Cell::from(c.separate_with_commas())
                };

                if index == 0 {
                    cell = cell.style(Style::default().fg(TEXT).add_modifier(Modifier::BOLD));
                }
                cell
            });
            Row::new(cells)
                .height(1)
                .bottom_margin(0)
                .style(Style::default().fg(TEXT))
        });

    let method_area = Table::new(
        method_rows,
        [
            Constraint::Percentage(14),
            Constraint::Percentage(14),
            Constraint::Percentage(14),
            Constraint::Percentage(14),
            Constraint::Percentage(14),
            Constraint::Percentage(14),
            Constraint::Percentage(14),
        ],
    )
    .header(method_header)
    .block(styled_block(""))
    .style(Style::default().fg(BOX));

    match current_page {
        // Previously added a black block to year and month widget if a value is not selected
        // Now we will turn that black block into green if a value is selected
        SummaryTab::Months => {
            month_tab = month_tab
                .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(SELECTED));
        }

        SummaryTab::Years => {
            year_tab = year_tab
                .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(SELECTED));
        }
        SummaryTab::ModeSelection => {
            mode_selection_tab = mode_selection_tab
                .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(SELECTED));
        }
        SummaryTab::Table => {
            table_area = table_area
                .row_highlight_style(Style::default().bg(SELECTED))
                .highlight_symbol(">> ");
        }
    }

    // Always keep some items rendered on the upper side of the table
    if let Some(index) = table_data.state.selected()
        && index > 7 {
            *table_data.state.offset_mut() = index - 7;
        }

    if summary_hidden_mode {
        f.render_stateful_widget(summary_area_1, left_summary[0], &mut summary_table_1.state);
        f.render_stateful_widget(summary_area_2, left_summary[1], &mut summary_table_2.state);
        f.render_stateful_widget(summary_area_3, right_summary[0], &mut summary_table_3.state);
        f.render_stateful_widget(summary_area_4, right_summary[1], &mut summary_table_4.state);
        f.render_stateful_widget(table_area, chunks[2], &mut table_data.state);
        f.render_stateful_widget(method_area, chunks[0], &mut method_table.state);
    } else {
        f.render_widget(mode_selection_tab, chunks[0]);
        f.render_stateful_widget(summary_area_1, left_summary[0], &mut summary_table_1.state);
        f.render_stateful_widget(summary_area_2, left_summary[1], &mut summary_table_2.state);
        f.render_stateful_widget(summary_area_3, right_summary[0], &mut summary_table_3.state);
        f.render_stateful_widget(summary_area_4, right_summary[1], &mut summary_table_4.state);

        match mode_selection.index {
            0 => {
                f.render_widget(year_tab, chunks[1]);
                f.render_widget(month_tab, chunks[2]);
                f.render_stateful_widget(table_area, chunks[5], &mut table_data.state);
                f.render_stateful_widget(method_area, chunks[3], &mut method_table.state);
            }
            1 => {
                f.render_widget(year_tab, chunks[1]);
                f.render_stateful_widget(table_area, chunks[4], &mut table_data.state);
                f.render_stateful_widget(method_area, chunks[2], &mut method_table.state);
            }
            2 => {
                f.render_stateful_widget(table_area, chunks[3], &mut table_data.state);
                f.render_stateful_widget(method_area, chunks[1], &mut method_table.state);
            }
            _ => {}
        }
    }
}
