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

/// The function draws the Home page of the interface.
#[cfg(not(tarpaulin_include))]
pub fn home_ui(
    f: &mut Frame,
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
    balance_load_percentage: &mut f64,
    income_load_percentage: &mut f64,
    expense_load_percentage: &mut f64,
    conn: &Connection,
) {
    let all_methods = get_all_tx_methods(conn);
    let size = f.size();

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
        .constraints(
            [
                Constraint::Length(7),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
            ]
            .as_ref(),
        )
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

    if *balance_load_percentage < 1.0 {
        *balance_load_percentage += 0.004;
    } else {
        *balance_load_percentage = 1.0
    }

    if *income_load_percentage < 1.0 {
        *income_load_percentage += 0.004;
    } else {
        *income_load_percentage = 1.0
    }

    if *expense_load_percentage < 1.0 {
        *expense_load_percentage += 0.004;
    } else {
        *expense_load_percentage = 1.0
    }

    // go through all data of the Balance widget and style it as necessary
    let bal_data = balance.iter().map(|item| {
        let height = 1;

        let row_type = HomeRow::get_row(item);
        let mut index = 0;

        match row_type {
            HomeRow::Balance => {
                if item[1..] != *ongoing_balance {
                    *last_balance = ongoing_balance.clone();
                    *ongoing_balance = item[1..].to_owned();
                    *balance_load_percentage = 0.0;
                }
            }
            HomeRow::Changes => {
                if item[1..] != *ongoing_changes {
                    *last_changes = ongoing_changes.clone();
                    *ongoing_changes = item[1..].to_owned();
                    *balance_load_percentage = 0.0;
                }
            }
            HomeRow::Income => {
                if item[1..] != *ongoing_income {
                    *last_income = ongoing_income.clone();
                    *ongoing_income = item[1..].to_owned();
                    *income_load_percentage = 0.0;
                }
            }
            HomeRow::Expense => {
                if item[1..] != *ongoing_expense {
                    *last_expense = ongoing_expense.clone();
                    *ongoing_expense = item[1..].to_owned();
                    *expense_load_percentage = 0.0;
                }
            }
            HomeRow::TopRow => {}
        }

        let cells = item.iter().map(|c| {
            let c = if row_type != HomeRow::TopRow
                && !["Balance", "Changes", "Income", "Expense"].contains(&c.as_str())
            {
                let load_data = match row_type {
                    HomeRow::Balance => balance_load.get_mut(index).unwrap(),
                    HomeRow::Changes => changes_load.get_mut(index).unwrap(),
                    HomeRow::Expense => expense_load.get_mut(index).unwrap(),
                    HomeRow::Income => income_load.get_mut(index).unwrap(),
                    HomeRow::TopRow => unreachable!(),
                };

                let symbol = if c.contains('↑') || c.contains('↓') {
                    c.chars().next()
                } else {
                    None
                };

                let actual_data: f64 = if row_type != HomeRow::Changes {
                    c.parse().unwrap()
                } else if let Some(sym) = symbol {
                    let without_symbol = c.replace(sym, "");
                    without_symbol.parse().unwrap()
                } else {
                    c.parse().unwrap()
                };

                let last_data: f64 = match row_type {
                    HomeRow::Balance => last_balance.get(index).unwrap().parse().unwrap(),
                    HomeRow::Changes => {
                        let last_change_symbol = last_changes.get(index).unwrap();

                        if last_change_symbol.contains('↑') || last_change_symbol.contains('↓')
                        {
                            let last_symbol = last_change_symbol.chars().next().unwrap();
                            let without_symbol = last_change_symbol.replace(last_symbol, "");
                            without_symbol.parse().unwrap()
                        } else {
                            last_change_symbol.parse().unwrap()
                        }
                    }
                    HomeRow::Expense => last_expense.get(index).unwrap().parse().unwrap(),
                    HomeRow::Income => last_income.get(index).unwrap().parse().unwrap(),
                    HomeRow::TopRow => unreachable!(),
                };

                let difference = if last_data > actual_data {
                    last_data - actual_data
                } else {
                    actual_data - last_data
                };

                index += 1;

                if actual_data > last_data {
                    match row_type {
                        HomeRow::Balance | HomeRow::Changes => {
                            *load_data = last_data + (difference * *balance_load_percentage)
                        }
                        HomeRow::Expense => {
                            *load_data = last_data + (difference * *expense_load_percentage)
                        }
                        HomeRow::Income => {
                            *load_data = last_data + (difference * *income_load_percentage)
                        }
                        HomeRow::TopRow => unreachable!(),
                    }
                } else if last_data > actual_data {
                    match row_type {
                        HomeRow::Balance | HomeRow::Changes => {
                            *load_data = last_data - (difference * *balance_load_percentage)
                        }
                        HomeRow::Income => {
                            *load_data = last_data - (difference * *income_load_percentage)
                        }
                        HomeRow::Expense => {
                            *load_data = last_data - (difference * *expense_load_percentage)
                        }
                        HomeRow::TopRow => unreachable!(),
                    }
                } else {
                    *load_data = actual_data;
                }

                if *load_data < 0.0 {
                    *load_data = 0.0
                }

                if row_type != HomeRow::Changes {
                    format!("{load_data:.2}").separate_with_commas()
                } else if let Some(sym) = symbol {
                    format!("{}{load_data:.2}", sym).separate_with_commas()
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
            } else if all_methods.contains(&c)
                || c == "Balance"
                || c == "Changes"
                || c == "Total"
                || c == "Income"
                || c == "Expense"
            {
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
    let balance_area = Table::new(bal_data, width_data)
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
                    table_area = table_area.highlight_style(selected_style_expense)
                } else if table.items[a][4] == "Income" {
                    table_area = table_area.highlight_style(selected_style_income)
                } else if table.items[a][4] == "Transfer" {
                    table_area = table_area.highlight_style(Style::default().bg(SELECTED))
                }
            }
        }
    }

    // after all data is in place, render the widgets one by one
    // the chunks are selected based on the format I want the widgets to render
    f.render_widget(balance_area, chunks[0]);
    f.render_widget(month_tab, chunks[2]);
    f.render_widget(year_tab, chunks[1]);

    // this one is different because the Transaction widget interface works differently
    f.render_stateful_widget(table_area, chunks[3], &mut table.state)
}
