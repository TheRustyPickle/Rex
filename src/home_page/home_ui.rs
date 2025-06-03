use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Cell, Row, Table};
use ratatui::Frame;
use rusqlite::Connection;
use thousands::Separable;

use crate::page_handler::{
    HomeRow, HomeTab, IndexedData, TableData, BACKGROUND, BLUE, BOX, HEADER, RED, SELECTED, TEXT,
};
use crate::utility::{create_tab, get_all_tx_methods, main_block, styled_block};

pub const BALANCE_BOLD: [&str; 7] = [
    "Balance",
    "Changes",
    "Total",
    "Income",
    "Expense",
    "Daily Income",
    "Daily Expense",
];

/// The function draws the Home page of the interface.
#[cfg(not(tarpaulin_include))]
pub fn home_ui(
    f: &mut Frame,
    to_reset: bool,
    months: &IndexedData,
    years: &IndexedData,
    table: &mut TableData,
    balance: &mut [Vec<String>],
    current_tab: &HomeTab,
    width_data: &mut [Constraint],
    balance_load: &mut [f64],
    ongoing_balance: &mut Vec<String>,
    last_balance: &mut Vec<String>,
    changes_load: &mut [f64],
    ongoing_changes: &mut Vec<String>,
    last_changes: &mut Vec<String>,
    income_load: &mut [f64],
    ongoing_income: &mut Vec<String>,
    last_income: &mut Vec<String>,
    expense_load: &mut [f64],
    ongoing_expense: &mut Vec<String>,
    last_expense: &mut Vec<String>,
    daily_income_load: &mut [f64],
    daily_ongoing_income: &mut Vec<String>,
    daily_last_income: &mut Vec<String>,
    daily_expense_load: &mut [f64],
    daily_ongoing_expense: &mut Vec<String>,
    daily_last_expense: &mut Vec<String>,
    load_percentage: &mut f64,
    conn: &Connection,
) {
    let all_methods = get_all_tx_methods(conn);
    let size = f.area();

    // Used to highlight Changes on Balance section of Home Page
    let selected_style_income = Style::default().fg(BLUE).add_modifier(Modifier::REVERSED);
    let selected_style_expense = Style::default().fg(RED).add_modifier(Modifier::REVERSED);

    let mut table_name = "Transactions".to_string();

    if !table.items.is_empty() {
        table_name = format!("Transactions: {}", table.items.len());
    }

    // Transaction widget's top row/header to highlight what each data will mean
    let header_cells = ["Date", "Details", "TX Method", "Amount", "Type", "Tags"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(BACKGROUND)));

    let header = Row::new(header_cells)
        .style(Style::default().bg(HEADER))
        .height(1)
        .bottom_margin(0);

    // iter through table data and turn them into rows and columns
    let rows = table.items.iter().map(|item| {
        let height = 1;
        let cells = item.iter().map(|c| Cell::from(c.separate_with_commas()));
        Row::new(cells)
            .height(height as u16)
            .bottom_margin(0)
            .style(Style::default().bg(BACKGROUND).fg(TEXT))
    });

    // Decides how many chunks of spaces in the terminal will be.
    // Each constraint creates an empty space in the terminal with the given
    // length. The final one was given 0 as minimum value which is the Transaction
    // field to keep it expanding.

    // chunks are used in this format respectively
    // - The Balance tab
    // - The year tab
    // - The month tab
    // - The transaction list/Table

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(9),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(size);

    f.render_widget(main_block(), size);

    let mut month_tab = create_tab(months, "Months");

    let mut year_tab = create_tab(years, "Years");

    // set up the table columns and their size
    // resizing the table headers to match a % of the
    // terminal space

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

    if (*load_percentage + 0.004) <= 1.0 {
        *load_percentage += 0.004;
    } else {
        *load_percentage = 1.0;
    }

    let mut discrepancy_exists = false;

    // go through all data of the Balance widget and style it as necessary
    let bal_data = balance.iter().map(|item| {
        let height = 1;

        let row_type = HomeRow::get_row(item);
        let mut index = 0;

        // Check all the row types
        // If `to_rest` is true, it means a key was pressed which may have an impact on the shown data
        // If it's true, if current data that is being shown and the new data to be shown is the same
        // update the last_ var data with the current data
        // Example went from month 5 to 6, last_ var will contain 5'th months data. If going from 6 to 5, then 6'th data
        // If not the same, that means there were data changes so
        // Update ongoing data that is to be shown, along with last data that is being shown before the update
        match row_type {
            HomeRow::Balance => {
                if to_reset {
                    if *ongoing_balance != item[1..] {
                        last_balance.clone_from(ongoing_balance);
                        item[1..].clone_into(ongoing_balance);
                        discrepancy_exists = true;
                    } else {
                        last_balance.clone_from(ongoing_balance);
                    }
                }
            }
            HomeRow::Changes => {
                if to_reset {
                    if *ongoing_changes != item[1..] {
                        last_changes.clone_from(ongoing_changes);
                        item[1..].clone_into(ongoing_changes);
                        discrepancy_exists = true;
                    } else {
                        last_changes.clone_from(ongoing_changes);
                    }
                }
            }
            HomeRow::Income => {
                if to_reset {
                    if *ongoing_income != item[1..] {
                        last_income.clone_from(ongoing_income);
                        item[1..].clone_into(ongoing_income);
                        discrepancy_exists = true;
                    } else {
                        last_income.clone_from(ongoing_income);
                    }
                }
            }
            HomeRow::Expense => {
                if to_reset {
                    if *ongoing_expense != item[1..] {
                        last_expense.clone_from(ongoing_expense);
                        item[1..].clone_into(ongoing_expense);
                        discrepancy_exists = true;
                    } else {
                        last_expense.clone_from(ongoing_expense);
                    }
                }
            }
            HomeRow::DailyIncome => {
                if to_reset {
                    if *daily_ongoing_income != item[1..] {
                        daily_last_income.clone_from(daily_ongoing_income);
                        item[1..].clone_into(daily_ongoing_income);
                        discrepancy_exists = true;
                    } else {
                        daily_last_income.clone_from(daily_ongoing_income);
                    }
                }
            }
            HomeRow::DailyExpense => {
                if to_reset {
                    if *daily_ongoing_expense != item[1..] {
                        daily_last_expense.clone_from(daily_ongoing_expense);
                        item[1..].clone_into(daily_ongoing_expense);
                        discrepancy_exists = true;
                    } else {
                        daily_last_expense.clone_from(daily_ongoing_expense);
                    }
                }
            }
            HomeRow::TopRow => {}
        }

        if to_reset && discrepancy_exists {
            *load_percentage = 0.0;
        }

        let cells = item.iter().map(|c| {
            let c = if row_type != HomeRow::TopRow
                && ![
                    "Balance",
                    "Changes",
                    "Income",
                    "Expense",
                    "Daily Income",
                    "Daily Expense",
                ]
                .contains(&c.as_str())
            {
                // Get the load data we need to update for this redraw
                let load_data = match row_type {
                    HomeRow::Balance => balance_load.get_mut(index).unwrap(),
                    HomeRow::Changes => changes_load.get_mut(index).unwrap(),
                    HomeRow::Expense => expense_load.get_mut(index).unwrap(),
                    HomeRow::Income => income_load.get_mut(index).unwrap(),
                    HomeRow::DailyIncome => daily_income_load.get_mut(index).unwrap(),
                    HomeRow::DailyExpense => daily_expense_load.get_mut(index).unwrap(),
                    HomeRow::TopRow => unreachable!(),
                };

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

                // The data that is currently being shown, before this redraw happens
                let last_data: f64 = match row_type {
                    HomeRow::Balance => last_balance
                        .get(index)
                        .unwrap_or(&format!("0"))
                        .parse()
                        .unwrap(),
                    HomeRow::Changes => {
                        // let last_change_symbol = last_changes.get(index).unwrap();
                        let last_change_symbol = if let Some(d) = last_changes.get(index) {
                            d
                        } else {
                            &format!("0")
                        };

                        if last_change_symbol.contains('↑') || last_change_symbol.contains('↓')
                        {
                            let last_symbol = last_change_symbol.chars().next().unwrap();
                            let without_symbol = last_change_symbol.replace(last_symbol, "");
                            without_symbol.parse().unwrap()
                        } else {
                            last_change_symbol.parse().unwrap()
                        }
                    }
                    HomeRow::Expense => last_expense
                        .get(index)
                        .unwrap_or(&format!("0"))
                        .parse()
                        .unwrap(),
                    HomeRow::Income => last_income
                        .get(index)
                        .unwrap_or(&format!("0"))
                        .parse()
                        .unwrap(),
                    HomeRow::DailyIncome => daily_last_income
                        .get(index)
                        .unwrap_or(&format!("0"))
                        .parse()
                        .unwrap(),
                    HomeRow::DailyExpense => daily_last_expense
                        .get(index)
                        .unwrap_or(&format!("0"))
                        .parse()
                        .unwrap(),
                    HomeRow::TopRow => unreachable!(),
                };

                // Difference can go both ways, either from 0 to a positive number or to a negative number
                let difference = if last_data > actual_data {
                    last_data - actual_data
                } else {
                    actual_data - last_data
                };

                let load_difference = if *load_data > actual_data {
                    *load_data - actual_data
                } else {
                    actual_data - *load_data
                };

                index += 1;

                // If going from 0 to positive number, we add the load percentage % amount from the actual amount that is to be shown
                // to the load data
                // If going the other way, we remove load percentage % amount
                // If neither then they are both equal, nothing to do, loading has finished
                if actual_data > last_data {
                    match row_type {
                        HomeRow::TopRow => unreachable!(),
                        _ => *load_data = last_data + (difference * *load_percentage),
                    }
                } else if last_data > actual_data {
                    match row_type {
                        HomeRow::TopRow => unreachable!(),
                        _ => *load_data = last_data - (difference * *load_percentage),
                    }
                } else {
                    // If we are here it means an interaction happened before the previous load percentage could reach 100%
                    // This is a fallback option and won't be 100% similar to normal animation
                    // Instead of keeping on adding/reducing the same amount each frame load by calculating the difference between
                    // the actual data the last data, we will find the difference by calculating different between the load amount
                    // or the amount that is being shown in the UI in this frame load and the actual data.
                    //
                    // Unlike normal animation, this is non-linear. Example actual data 1000, load data 0.
                    // First frame load (1000 - 0) * 0.004 = 4. So load data would be 4
                    // Second frame load (1000 - 4) * 0.008 = 7.968. Load data would be 4 + 7.968 = 11.968
                    //
                    // Difference will continue to be higher and higher until a certain point then it will start going down
                    // Number animation difference will be noticeable compared with normal animation
                    if load_difference != 0.0 {
                        match row_type {
                            HomeRow::TopRow => unreachable!(),
                            _ => {
                                if actual_data > *load_data {
                                    match row_type {
                                        HomeRow::TopRow => unreachable!(),
                                        _ => *load_data += load_difference * *load_percentage,
                                    }
                                } else if *load_data > actual_data {
                                    match row_type {
                                        HomeRow::TopRow => unreachable!(),
                                        _ => *load_data -= load_difference * *load_percentage,
                                    }
                                }
                            }
                        }
                    }
                }

                // 100% has been reached, show the actual balance to the UI now
                if *load_percentage == 1.0 {
                    *load_data = actual_data;
                }

                // re-add the previously removed symbol if is the Changes row
                // Otherwise separate the number with commas
                if let Some(sym) = symbol {
                    format!("{sym}{load_data:.2}",).separate_with_commas()
                } else {
                    format!("{load_data:.2}").separate_with_commas()
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

    // use the acquired width data to allocated spaces
    // between columns on Balance widget.
    let balance_area = Table::new(bal_data, width_data.to_owned())
        .block(styled_block("Balance"))
        .style(Style::default().fg(BOX));

    match current_tab {
        // previously added a black block to year and month widget if a value is not selected
        // Now we will turn that black block into green if a value is selected
        HomeTab::Months => {
            month_tab = month_tab
                .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(SELECTED));
        }

        HomeTab::Years => {
            year_tab = year_tab
                .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(SELECTED));
        }
        // changes the color of row based on Expense or Income tx type on Transaction widget.
        HomeTab::Table => {
            if let Some(a) = table.state.selected() {
                table_area = table_area.highlight_symbol(">> ");
                if table.items[a][4] == "Expense" {
                    table_area = table_area.row_highlight_style(selected_style_expense);
                } else if table.items[a][4] == "Income" {
                    table_area = table_area.row_highlight_style(selected_style_income);
                } else if table.items[a][4] == "Transfer" {
                    table_area = table_area.row_highlight_style(Style::default().bg(SELECTED));
                }
            }
        }
    }

    // Always keep some items rendered on the upper side of the table
    if let Some(index) = table.state.selected() {
        if index > 10 {
            *table.state.offset_mut() = index - 10;
        }
    }

    // after all data is in place, render the widgets one by one
    // the chunks are selected based on the format I want the widgets to render
    f.render_widget(balance_area, chunks[0]);
    f.render_widget(month_tab, chunks[2]);
    f.render_widget(year_tab, chunks[1]);

    // this one is different because the Transaction widget interface works differently
    f.render_stateful_widget(table_area, chunks[3], &mut table.state);
}
