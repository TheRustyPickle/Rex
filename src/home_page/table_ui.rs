use crate::home_page::{SelectedTab, TableData, TimeData};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, Row, Table, Tabs},
    Frame,
};

/// This function is responsible for drawing all the widgets in the Home page,
/// coloring everything and all related things.  This function takes several arguments
/// from the run_app function with the necessary data and fields.

pub fn ui<B: Backend>(
    f: &mut Frame<B>,
    months: &TimeData,
    years: &TimeData,
    table: &mut TableData,
    balance: &mut Vec<Vec<String>>,
    cu_tab: &SelectedTab,
    width_data: &mut Vec<Constraint>,
) {
    let size = f.size();

    // These two colors are used with the Changes value when a row is selected
    // to color the Changes row in Balance widget.
    let selected_style_blue = Style::default()
        .fg(Color::Blue)
        .add_modifier(Modifier::REVERSED);
    let selected_style_red = Style::default()
        .fg(Color::Red)
        .add_modifier(Modifier::REVERSED);

    let normal_style = Style::default().bg(Color::LightBlue);

    // Transaction widget's top row/header to highlight what each data will mean
    let header_cells = ["Date", "Details", "TX Method", "Amount", "Type"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Rgb(255, 255, 255))));

    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(0);

    // iter through table data and turn them into rows and columns
    let rows = table.items.iter().map(|item| {
        let height = 1;
        let cells = item.iter().map(|c| Cell::from(c.to_string()));
        Row::new(cells).height(height as u16).bottom_margin(0)
    });

    // Decides how many chunks of spaces in the terminal will be.
    // Each constraint creates an empty space in the terminal with the given
    // length. The final one was given 0 as minimum value which is the Transaction
    // field to keep it expanding.
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

    let block = Block::default().style(
        Style::default()
            .bg(Color::Rgb(255, 255, 255))
            .fg(Color::Rgb(50, 205, 50)),
    );
    f.render_widget(block, size);

    //color the first three letters of the month to blue
    let month_titles = months
        .titles
        .iter()
        .map(|t| {
            let (first, rest) = t.split_at(3);
            Spans::from(vec![
                Span::styled(first, Style::default().fg(Color::Blue)),
                Span::styled(rest, Style::default().fg(Color::Rgb(50, 205, 50))),
            ])
        })
        .collect();

    //color the first letter of the year to blue
    let year_titles = years
        .titles
        .iter()
        .map(|t| {
            let (first, rest) = t.split_at(2);
            Spans::from(vec![
                Span::styled(first, Style::default().fg(Color::Blue)),
                Span::styled(rest, Style::default().fg(Color::Rgb(50, 205, 50))),
            ])
        })
        .collect();

    // The default style for the select index in the month section if
    // the Month widget is not selected
    let mut month_tab = Tabs::new(month_titles)
        .block(Block::default().borders(Borders::ALL).title("Months"))
        .select(months.index)
        .style(Style::default().fg(Color::Rgb(50, 205, 50)))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Black),
        );

    // The default style for the select index in the year section if
    // the Year widget is not selected
    let mut year_tab = Tabs::new(year_titles)
        .block(Block::default().borders(Borders::ALL).title("Years"))
        .select(years.index)
        .style(Style::default().fg(Color::Rgb(50, 205, 50)))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Black),
        );

    // set up the table columns and their size
    // resizing the table headers to match a % of the
    // terminal space
    let mut table_area = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Transactions"))
        .widths(&[
            Constraint::Percentage(10),
            Constraint::Percentage(40),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
        ]);

    // This is what makes the Changes row in the Balance widget red or blue based on
    // a selected transaction inside the Table/Transaction widget
    let bal_data = balance.iter().map(|item| {
        let height = 1;
        let cells = item.iter().map(|c| {
            if c.contains("↑") {
                Cell::from(c.to_string()).style(Style::default().fg(Color::Blue))
            } else if c.contains("↓") {
                Cell::from(c.to_string()).style(Style::default().fg(Color::Red))
            } else {
                Cell::from(c.to_string())
            }
        });
        Row::new(cells).height(height as u16).bottom_margin(0)
    });

    // use the acquired width data to allocated spaces
    // between columns on Balance widget.
    let balance_area = Table::new(bal_data)
        .block(Block::default().borders(Borders::ALL).title("Balance"))
        .widths(&width_data);

    match cu_tab {
        // previously added a black block to year and month widget if a value is not selected
        // Now we will turn that black block into green if a value is selected
        SelectedTab::Months => {
            month_tab = month_tab.highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Rgb(152, 251, 152)),
            );
        }

        SelectedTab::Years => {
            year_tab = year_tab.highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Rgb(152, 251, 152)),
            );
        }
        // changes the color of row based on Expense or Income tx type on Transaction widget.
        SelectedTab::Table => {
            if let Some(a) = table.state.selected() {
                if table.items[a][4] == "Expense" {
                    table_area = table_area
                        .highlight_style(selected_style_red)
                        .highlight_symbol(">> ")
                } else if table.items[a][4] == "Income" {
                    table_area = table_area
                        .highlight_style(selected_style_blue)
                        .highlight_symbol(">> ")
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
