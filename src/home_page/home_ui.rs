use crate::page_handler::{
    HomeTab, IndexedData, TableData, BACKGROUND, BLUE, BOX, HEADER, HIGHLIGHTED, RED, SELECTED,
    TEXT,
};
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, Cell, Row, Table, Tabs};
use tui::Frame;

/// This function is responsible for drawing all the widgets in the Home page,
/// coloring everything and all related things.  This function takes several arguments
/// from the run_app function with the necessary data and fields.

pub fn home_ui<B: Backend>(
    f: &mut Frame<B>,
    months: &IndexedData,
    years: &IndexedData,
    table: &mut TableData,
    balance: &mut [Vec<String>],
    current_tab: &HomeTab,
    width_data: &mut [Constraint],
) {
    let size = f.size();

    // These two colors are used with the Changes value when a row is selected
    // to color the Changes row in Balance widget.
    let selected_style_income = Style::default().fg(BLUE).add_modifier(Modifier::REVERSED);

    let selected_style_expense = Style::default().fg(RED).add_modifier(Modifier::REVERSED);

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
        let cells = item.iter().map(|c| Cell::from(c.to_string()));
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

    let block = Block::default().style(Style::default().bg(BACKGROUND).fg(BOX));
    f.render_widget(block, size);

    // color the first three letters of the month to blue
    let month_titles = months
        .titles
        .iter()
        .map(|t| Spans::from(vec![Span::styled(t, Style::default().fg(TEXT))]))
        .collect();

    //color the first two letters of the year to blue
    let year_titles = years
        .titles
        .iter()
        .map(|t| Spans::from(vec![Span::styled(t, Style::default().fg(TEXT))]))
        .collect();

    // The default style for the select index in the month section if
    // the Month widget is not selected
    let mut month_tab = Tabs::new(month_titles)
        .block(Block::default().borders(Borders::ALL).title("Months"))
        .select(months.index)
        .style(Style::default().fg(BOX))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(HIGHLIGHTED),
        );

    // The default style for the select index in the year section if
    // the Year widget is not selected
    let mut year_tab = Tabs::new(year_titles)
        .block(Block::default().borders(Borders::ALL).title("Years"))
        .select(years.index)
        .style(Style::default().fg(BOX))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(HIGHLIGHTED),
        );

    // set up the table columns and their size
    // resizing the table headers to match a % of the
    // terminal space

    let mut table_area = Table::new(rows)
        .header(header)
        .block(
            Block::default()
                .style(Style::default().fg(BOX))
                .borders(Borders::ALL)
                .title("Transactions"),
        )
        .widths(&[
            Constraint::Percentage(10),
            Constraint::Percentage(37),
            Constraint::Percentage(13),
            Constraint::Percentage(13),
            Constraint::Percentage(8),
            Constraint::Percentage(18),
        ]);

    // This is what makes the Changes row in the Balance widget red or blue based on
    // a selected transaction inside the Table/Transaction widget
    let bal_data = balance.iter().map(|item| {
        let height = 1;
        let cells = item.iter().map(|c| {
            if c.contains('↑') {
                Cell::from(c.to_string()).style(Style::default().fg(BLUE))
            } else if c.contains('↓') {
                Cell::from(c.to_string()).style(Style::default().fg(RED))
            } else {
                Cell::from(c.to_string())
            }
        });
        Row::new(cells)
            .height(height as u16)
            .bottom_margin(0)
            .style(Style::default().fg(TEXT))
    });

    // use the acquired width data to allocated spaces
    // between columns on Balance widget.
    let balance_area = Table::new(bal_data)
        .block(Block::default().borders(Borders::ALL).title("Balance"))
        .widths(width_data)
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
            table_area = table_area.highlight_symbol(">> ");
            if let Some(a) = table.state.selected() {
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
