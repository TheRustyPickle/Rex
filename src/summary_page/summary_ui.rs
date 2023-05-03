use crate::page_handler::{
    IndexedData, SummaryTab, TableData, BACKGROUND, BOX, HEADER, HIGHLIGHTED, SELECTED, TEXT,
};
use crate::summary_page::SummaryData;
use crate::utility::{main_block, styled_block};
use thousands::Separable;
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Cell, Row, Table, Tabs};
use tui::Frame;

/// The function draws the Summary page of the interface.
pub fn summary_ui<B: Backend>(
    f: &mut Frame<B>,
    months: &IndexedData,
    years: &IndexedData,
    mode_selection: &IndexedData,
    summary_data: &SummaryData,
    table_data: &mut TableData,
    current_page: &SummaryTab,
) {
    let (summary_data_1, summary_data_2, summary_data_3, summary_data_4) =
        summary_data.get_tx_data(mode_selection, months.index, years.index);

    let mut summary_table_1 = TableData::new(summary_data_1);
    let mut summary_table_2 = TableData::new(summary_data_2);
    let mut summary_table_3 = TableData::new(summary_data_3);
    let mut summary_table_4 = TableData::new(summary_data_4);

    let size = f.size();

    let header_cells = ["Tag", "Total Income", "Total Expense"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(BACKGROUND)));

    let header = Row::new(header_cells)
        .style(Style::default().bg(HEADER))
        .height(1)
        .bottom_margin(0);

    let mut main_layout = Layout::default().direction(Direction::Vertical).margin(2);
    let mut summary_layout = Layout::default().direction(Direction::Horizontal);

    match mode_selection.index {
        0 => {
            main_layout = main_layout.constraints(
                [
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(9),
                    Constraint::Min(0),
                ]
                .as_ref(),
            );
            summary_layout = summary_layout
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)]);
        }
        1 => {
            main_layout = main_layout.constraints(
                [
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(9),
                    Constraint::Min(0),
                ]
                .as_ref(),
            );
            summary_layout = summary_layout
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)]);
        }
        2 => {
            main_layout = main_layout.constraints(
                [
                    Constraint::Length(3),
                    Constraint::Length(9),
                    Constraint::Min(0),
                ]
                .as_ref(),
            );
            summary_layout = summary_layout
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)]);
        }
        _ => {}
    };

    let chunks = main_layout.split(size);
    let summary_chunk = summary_layout.split(chunks[3 - mode_selection.index]);

    let left_summary = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(summary_chunk[0]);

    let right_summary = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(summary_chunk[1]);

    f.render_widget(main_block(), size);

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

    let mode_selection_titles = mode_selection
        .titles
        .iter()
        .map(|t| Spans::from(vec![Span::styled(t, Style::default().fg(TEXT))]))
        .collect();

    // The default style for the select index in the month section if
    // the Month widget is not selected
    let mut month_tab = Tabs::new(month_titles)
        .block(styled_block("Months"))
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
        .block(styled_block("Years"))
        .select(years.index)
        .style(Style::default().fg(BOX))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(HIGHLIGHTED),
        );

    let mut mode_selection_tab = Tabs::new(mode_selection_titles)
        .block(styled_block("Mode Selection"))
        .select(mode_selection.index)
        .style(Style::default().fg(BOX))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(HIGHLIGHTED),
        );

    // * Goes through all tags provided and creates row for the table
    let rows = table_data.items.iter().map(|item| {
        let height = 1;
        let cells = item
            .iter()
            .map(|c| Cell::from(c.to_string().separate_with_commas()));
        Row::new(cells)
            .height(height as u16)
            .bottom_margin(0)
            .style(Style::default().fg(TEXT))
    });

    let summary_rows_1 = summary_table_1.items.iter().map(|item| {
        let height = 1;
        let cells = item.iter().enumerate().map(|(j, c)| {
            let mut cell = Cell::from(c.to_string().separate_with_commas());
            if j == 0 {
                cell = cell.style(Style::default().fg(TEXT).add_modifier(Modifier::BOLD));
            }
            cell
        });
        Row::new(cells)
            .height(height as u16)
            .bottom_margin(0)
            .style(Style::default().fg(TEXT))
    });

    let summary_rows_2 = summary_table_2.items.iter().map(|item| {
        let height = 1;
        let cells = item.iter().enumerate().map(|(j, c)| {
            let mut cell = Cell::from(c.to_string().separate_with_commas());
            if j == 0 {
                cell = cell.style(Style::default().fg(TEXT).add_modifier(Modifier::BOLD));
            }
            cell
        });
        Row::new(cells)
            .height(height as u16)
            .bottom_margin(0)
            .style(Style::default().fg(TEXT))
    });

    let summary_rows_3 = summary_table_3.items.iter().map(|item| {
        let height = 1;
        let cells = item.iter().enumerate().map(|(j, c)| {
            let mut cell = Cell::from(c.to_string().separate_with_commas());
            if j == 0 {
                cell = cell.style(Style::default().fg(TEXT).add_modifier(Modifier::BOLD));
            }
            cell
        });
        Row::new(cells)
            .height(height as u16)
            .bottom_margin(0)
            .style(Style::default().fg(TEXT))
    });

    let summary_rows_4 = summary_table_4.items.iter().map(|item| {
        let height = 1;
        let cells = item.iter().enumerate().map(|(j, c)| {
            let mut cell = Cell::from(c.to_string().separate_with_commas());
            if j == 0 {
                cell = cell.style(Style::default().fg(TEXT).add_modifier(Modifier::BOLD));
            }
            cell
        });
        Row::new(cells)
            .height(height as u16)
            .bottom_margin(0)
            .style(Style::default().fg(TEXT))
    });

    let mut table_area = Table::new(rows)
        .header(header)
        .block(styled_block("Tags"))
        .widths(&[
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ])
        .style(Style::default().fg(BOX));

    let summary_area_1 = Table::new(summary_rows_1)
        .block(styled_block(""))
        .widths(&[
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ])
        .style(Style::default().fg(BOX));

    let summary_area_2 = Table::new(summary_rows_2)
        .block(styled_block(""))
        .widths(&[
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ])
        .style(Style::default().fg(BOX));

    let summary_area_3 = Table::new(summary_rows_3)
        .block(styled_block(""))
        .widths(&[
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .style(Style::default().fg(BOX));

    let summary_area_4 = Table::new(summary_rows_4)
        .block(styled_block(""))
        .widths(&[
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .style(Style::default().fg(BOX));

    //let paragraph = Paragraph::new(text).style(Style::default().bg(BACKGROUND).fg(TEXT));

    match current_page {
        // previously added a black block to year and month widget if a value is not selected
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
                .highlight_style(Style::default().bg(SELECTED))
                .highlight_symbol(">> ")
        }
    }

    f.render_widget(mode_selection_tab, chunks[0]);
    f.render_stateful_widget(summary_area_1, left_summary[0], &mut summary_table_1.state);
    f.render_stateful_widget(summary_area_2, left_summary[1], &mut summary_table_2.state);
    f.render_stateful_widget(summary_area_3, right_summary[0], &mut summary_table_3.state);
    f.render_stateful_widget(summary_area_4, right_summary[1], &mut summary_table_4.state);

    match mode_selection.index {
        0 => {
            f.render_widget(year_tab, chunks[1]);
            f.render_widget(month_tab, chunks[2]);
            f.render_stateful_widget(table_area, chunks[4], &mut table_data.state)
        }
        1 => {
            f.render_widget(year_tab, chunks[1]);
            f.render_stateful_widget(table_area, chunks[3], &mut table_data.state)
        }
        2 => f.render_stateful_widget(table_area, chunks[2], &mut table_data.state),
        _ => {}
    }
}
