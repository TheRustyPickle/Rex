use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Cell, Row, Table};
use ratatui::Frame;
use rusqlite::Connection;
use thousands::Separable;

use crate::page_handler::{
    HomeRow, HomeTab, IndexedData, TableData, BACKGROUND, BLUE, BOX, HEADER, RED, SELECTED, TEXT,
};
use crate::utility::{create_tab, get_all_tx_methods, main_block, styled_block, LerpState};

pub const BALANCE_BOLD: [&str; 7] = [
    "Balance",
    "Changes",
    "Total",
    "Income",
    "Expense",
    "Daily Income",
    "Daily Expense",
];

/// The function draws the Homepage of the interface.
#[cfg(not(tarpaulin_include))]
pub fn home_ui(
    f: &mut Frame,
    months: &IndexedData,
    years: &IndexedData,
    table: &mut TableData,
    balance: &mut [Vec<String>],
    current_tab: &HomeTab,
    width_data: &mut [Constraint],
    lerp_state: &mut LerpState,
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

    // Transaction widget top row/header to highlight what each data will mean
    let header_cells = ["Date", "Details", "TX Method", "Amount", "Type", "Tags"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(BACKGROUND)));

    let header = Row::new(header_cells)
        .style(Style::default().bg(HEADER))
        .height(1)
        .bottom_margin(0);

    // Iter through table data and turn them into rows and columns
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

    // Chunks are used in this format respectively
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
    // resizing the table headers to match an % of the
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

    // Go through all data of the Balance widget and style it as necessary
    let bal_data = balance.iter().map(|item| {
        let height = 1_u16;

        let row_type = HomeRow::get_row(item);

        let mut index = 0;

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
                // Changes row can contain arrow symbols
                let symbol = if c.contains('↑') || c.contains('↓') {
                    c.chars().next()
                } else {
                    None
                };

                index += 1;

                // If loading was complete then this value is to be shown
                let actual_data: f64 = if row_type != HomeRow::Changes {
                    c.parse().unwrap()
                } else if let Some(sym) = symbol {
                    let without_symbol = c.replace(sym, "");
                    without_symbol.parse().unwrap()
                } else {
                    c.parse().unwrap()
                };

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
            .height(height)
            .bottom_margin(0)
            .style(Style::default().fg(TEXT))
    });

    // Use the acquired width data to the allocated spaces
    // between columns on Balance widget.
    let balance_area = Table::new(bal_data, width_data.to_owned())
        .block(styled_block("Balance"))
        .style(Style::default().fg(BOX));

    match current_tab {
        // Previously added a black block to year and month widget if a value is not selected
        // Now we will turn that black block into green if a value is selected
        HomeTab::Months => {
            month_tab = month_tab
                .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(SELECTED));
        }

        HomeTab::Years => {
            year_tab = year_tab
                .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(SELECTED));
        }
        // Changes the color of row based on Expense or Income tx type on Transaction widget.
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

    // After all data is in place, render the widgets one by one
    // the chunks are selected based on the format I want the widgets to render
    f.render_widget(balance_area, chunks[0]);
    f.render_widget(month_tab, chunks[2]);
    f.render_widget(year_tab, chunks[1]);

    // This one is different because the Transaction widget interface works differently
    f.render_stateful_widget(table_area, chunks[3], &mut table.state);
}
