use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, Tabs};
use tui::Frame;

use crate::page_handler::{IndexedData, SummaryTab, TableData};

/// Renders the Summary UI page
pub fn summary_ui<B: Backend>(
    f: &mut Frame<B>,
    months: &IndexedData,
    years: &IndexedData,
    mode_selection: &IndexedData,
    table_data: &mut TableData,
    text_data: &[(f64, String)],
    current_page: &SummaryTab,
) {
    let size = f.size();

    let normal_style = Style::default().bg(Color::LightBlue);
    let selected_style = Style::default().bg(Color::Rgb(255, 245, 238));

    let header_cells = ["Tag", "Total Income", "Total Expense"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Rgb(255, 255, 255))));

    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(0);

    let mut main_layout = Layout::default().direction(Direction::Vertical).margin(2);

    match mode_selection.index {
        0 => {
            main_layout = main_layout.constraints(
                [
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(7),
                    Constraint::Min(0),
                ]
                .as_ref(),
            )
        }
        1 => {
            main_layout = main_layout.constraints(
                [
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(7),
                    Constraint::Min(0),
                ]
                .as_ref(),
            )
        }
        2 => {
            main_layout = main_layout.constraints(
                [
                    Constraint::Length(3),
                    Constraint::Length(7),
                    Constraint::Min(0),
                ]
                .as_ref(),
            )
        }
        _ => {}
    };

    let chunks = main_layout.split(size);

    let block = Block::default().style(
        Style::default()
            .bg(Color::Rgb(255, 255, 255))
            .fg(Color::Rgb(50, 205, 50)),
    );

    f.render_widget(block, size);

    let month_titles = months
        .titles
        .iter()
        .map(|t| Spans::from(vec![Span::styled(t, Style::default().fg(Color::Blue))]))
        .collect();

    //color the first two letters of the year to blue
    let year_titles = years
        .titles
        .iter()
        .map(|t| Spans::from(vec![Span::styled(t, Style::default().fg(Color::Blue))]))
        .collect();

    let mode_selection_titles = mode_selection
        .titles
        .iter()
        .map(|t| Spans::from(vec![Span::styled(t, Style::default().fg(Color::Blue))]))
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

    let mut mode_selection_tab = Tabs::new(mode_selection_titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Mode Selection"),
        )
        .select(mode_selection.index)
        .style(Style::default().fg(Color::Rgb(50, 205, 50)))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Black),
        );

    // * contains the text for the upper side of the Summary UI
    let text = vec![
        Spans::from(Span::styled(
            format!("{} {:.2}", text_data[0].1, text_data[0].0),
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Blue),
        )),
        Spans::from(Span::styled(
            format!("{} {:.2}", text_data[1].1, text_data[1].0),
            Style::default().add_modifier(Modifier::BOLD).fg(Color::Red),
        )),
        Spans::from(vec![
            Span::styled(
                format!("Largest Income: {:.2}, ", text_data[2].0),
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Blue),
            ),
            Span::styled(
                format!("Method: {}", text_data[2].1),
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Rgb(205, 133, 63)),
            ),
        ]),
        Spans::from(vec![
            Span::styled(
                format!("Largest Expense: {:.2}, ", text_data[3].0),
                Style::default().add_modifier(Modifier::BOLD).fg(Color::Red),
            ),
            Span::styled(
                format!("Method: {}", text_data[3].1),
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Rgb(205, 133, 63)),
            ),
        ]),
        Spans::from(vec![
            Span::styled(
                format!("Most Earning Month: {}, ", text_data[4].1),
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Rgb(100, 149, 237)),
            ),
            Span::styled(
                format!("Income: {:.2}", text_data[4].0),
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Blue),
            ),
        ]),
        Spans::from(vec![
            Span::styled(
                format!("Most Expensive Month: {}, ", text_data[5].1),
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Rgb(205, 92, 92)),
            ),
            Span::styled(
                format!("Expense: {:.2}", text_data[5].0),
                Style::default().add_modifier(Modifier::BOLD).fg(Color::Red),
            ),
        ]),
    ];

    // * Goes through all tags provided and creates row for the table
    let rows = table_data.items.iter().map(|item| {
        let height = 1;
        let cells = item.iter().map(|c| Cell::from(c.to_string()));
        Row::new(cells).height(height as u16).bottom_margin(0)
    });

    let mut table_area = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Tags"))
        .widths(&[
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ]);

    let paragraph = Paragraph::new(text).style(
        Style::default()
            .bg(Color::Rgb(255, 255, 255))
            .fg(Color::Rgb(50, 205, 50)),
    );

    match current_page {
        // previously added a black block to year and month widget if a value is not selected
        // Now we will turn that black block into green if a value is selected
        SummaryTab::Months => {
            month_tab = month_tab.highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Rgb(152, 251, 152)),
            );
        }

        SummaryTab::Years => {
            year_tab = year_tab.highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Rgb(152, 251, 152)),
            );
        }
        SummaryTab::ModeSelection => {
            mode_selection_tab = mode_selection_tab.highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Rgb(152, 251, 152)),
            );
        }
        SummaryTab::Table => {
            table_area = table_area
                .highlight_style(selected_style)
                .highlight_symbol(">> ")
        }
    }

    f.render_widget(mode_selection_tab, chunks[0]);

    match mode_selection.index {
        0 => {
            f.render_widget(year_tab, chunks[1]);
            f.render_widget(month_tab, chunks[2]);
            f.render_widget(paragraph, chunks[3]);
            f.render_stateful_widget(table_area, chunks[4], &mut table_data.state)
        }
        1 => {
            f.render_widget(year_tab, chunks[1]);
            f.render_widget(paragraph, chunks[2]);
            f.render_stateful_widget(table_area, chunks[3], &mut table_data.state)
        }
        2 => {
            f.render_widget(paragraph, chunks[1]);
            f.render_stateful_widget(table_area, chunks[2], &mut table_data.state)
        }
        _ => {}
    }
}
