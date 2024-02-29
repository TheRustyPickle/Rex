use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Cell, Row, Table};
use ratatui::Frame;
use thousands::Separable;

use crate::history_page::HistoryData;
use crate::page_handler::{HistoryTab, IndexedData, TableData, BACKGROUND, HEADER, SELECTED, TEXT};
use crate::utility::{create_tab, main_block, styled_block};

pub fn history_ui(
    f: &mut Frame,
    months: &IndexedData,
    years: &IndexedData,
    current_tab: &HistoryTab,
    history_data: &HistoryData,
    table_data: &mut TableData,
) {
    let activity_txs_data = history_data.get_activity_txs(table_data.state.selected());
    let mut activity_txs_table = TableData::new(activity_txs_data);

    let is_edit_tx = if let Some(index) = table_data.state.selected() {
        history_data.add_extra_field(index)
    } else {
        false
    };

    let activity_tx_header_vec = if is_edit_tx {
        vec![
            "Date",
            "Details",
            "TX Method",
            "Amount",
            "Type",
            "Tags",
            "ID",
            "Status",
        ]
    } else {
        vec![
            "Date",
            "Details",
            "TX Method",
            "Amount",
            "Type",
            "Tags",
            "ID",
        ]
    };

    let activity_tx_header_widths = if is_edit_tx {
        vec![
            Constraint::Percentage(10),
            Constraint::Percentage(33),
            Constraint::Percentage(13),
            Constraint::Percentage(13),
            Constraint::Percentage(8),
            Constraint::Percentage(13),
            Constraint::Percentage(5),
            Constraint::Percentage(5),
        ]
    } else {
        vec![
            Constraint::Percentage(10),
            Constraint::Percentage(37),
            Constraint::Percentage(13),
            Constraint::Percentage(13),
            Constraint::Percentage(8),
            Constraint::Percentage(13),
            Constraint::Percentage(5),
        ]
    };

    let size = f.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(5),
            ]
            .as_ref(),
        )
        .split(size);

    f.render_widget(main_block(), size);

    let mut table_name = "Activities".to_string();

    if !table_data.items.is_empty() {
        table_name = format!("Activities: {}", table_data.items.len());
    }

    let activity_header_cells = ["Created At", "Activity Type", "Description"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(BACKGROUND)));

    let activity_tx_header_cells = activity_tx_header_vec
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(BACKGROUND)));

    let activity_header = Row::new(activity_header_cells)
        .style(Style::default().bg(HEADER))
        .height(1)
        .bottom_margin(0);

    let activity_tx_header = Row::new(activity_tx_header_cells)
        .style(Style::default().bg(HEADER))
        .height(1)
        .bottom_margin(0);

    let activity_rows = table_data.items.iter().map(|item| {
        let height = 1;
        let cells = item.iter().map(|c| Cell::from(c.separate_with_commas()));
        Row::new(cells)
            .height(height as u16)
            .bottom_margin(0)
            .style(Style::default().bg(BACKGROUND).fg(TEXT))
    });

    let activity_tx_rows = activity_txs_table.items.iter().map(|item| {
        let height = 1;
        let cells = item.iter().map(|c| Cell::from(c.separate_with_commas()));
        Row::new(cells)
            .height(height as u16)
            .bottom_margin(0)
            .style(Style::default().bg(BACKGROUND).fg(TEXT))
    });

    let mut activity_table_area = Table::new(
        activity_rows,
        [
            Constraint::Percentage(10),
            Constraint::Percentage(15),
            Constraint::Percentage(75),
        ],
    )
    .header(activity_header)
    .block(styled_block(&table_name));

    let activity_txs_table_area = Table::new(activity_tx_rows, activity_tx_header_widths)
        .header(activity_tx_header)
        .block(styled_block("Affected TX Data"));

    let mut month_tab = create_tab(months, "Months");
    let mut year_tab = create_tab(years, "Years");

    match current_tab {
        // previously added a black block to year and month widget if a value is not selected
        // Now we will turn that black block into green if a value is selected
        HistoryTab::Months => {
            month_tab = month_tab
                .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(SELECTED));
        }

        HistoryTab::Years => {
            year_tab = year_tab
                .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(SELECTED));
        }
        // changes the color of row based on Expense or Income tx type on Transaction widget.
        HistoryTab::List => {
            if table_data.state.selected().is_some() {
                activity_table_area = activity_table_area
                    .highlight_symbol(">> ")
                    .highlight_style(Style::default().bg(SELECTED));
            }
        }
    }

    if let Some(index) = table_data.state.selected() {
        if index > 10 {
            *table_data.state.offset_mut() = index - 10;
        }
    }

    f.render_widget(year_tab, chunks[0]);
    f.render_widget(month_tab, chunks[1]);
    f.render_stateful_widget(activity_table_area, chunks[2], &mut table_data.state);
    f.render_stateful_widget(
        activity_txs_table_area,
        chunks[3],
        &mut activity_txs_table.state,
    );
}
