use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Cell, Row, Table};
use rex_app::views::ActivityView;
use thousands::Separable;

use crate::page_handler::{ActivityTab, IndexedData, TableData};
use crate::theme::Theme;
use crate::utility::{LerpState, create_tab, main_block, styled_block};

pub fn activity_ui(
    f: &mut Frame,
    months: &IndexedData,
    years: &IndexedData,
    current_tab: &ActivityTab,
    activity_view: &ActivityView,
    table_data: &mut TableData,
    lerp_state: &mut LerpState,
    theme: &Theme,
) {
    let activity_txs_data = activity_view.get_activity_txs_table(table_data.state.selected());
    let mut activity_txs_table = TableData::new(activity_txs_data);

    let add_extra_field = if let Some(index) = table_data.state.selected() {
        activity_view.add_extra_field(index)
    } else {
        false
    };

    // Extra status field for search txs and edit txs
    let activity_tx_header_vec = if add_extra_field {
        vec![
            "Date",
            "Details",
            "TX Method",
            "Amount",
            "Type",
            "Tags",
            "Status",
        ]
    } else {
        vec!["Date", "Details", "TX Method", "Amount", "Type", "Tags"]
    };

    // Based on extra field, allocate size
    let activity_tx_header_widths = if add_extra_field {
        vec![
            Constraint::Percentage(10),
            Constraint::Percentage(33),
            Constraint::Percentage(13),
            Constraint::Percentage(13),
            Constraint::Percentage(8),
            Constraint::Percentage(13),
            Constraint::Percentage(10),
        ]
    } else {
        vec![
            Constraint::Percentage(10),
            Constraint::Percentage(37),
            Constraint::Percentage(13),
            Constraint::Percentage(15),
            Constraint::Percentage(10),
            Constraint::Percentage(15),
        ]
    };

    let size = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(5),
        ])
        .split(size);

    f.render_widget(main_block(theme), size);

    let tx_count = activity_view.total_activity();
    let lerp_id = "activity_tx_count";
    let lerp_tx_count = lerp_state.lerp(lerp_id, tx_count as f64) as i64;

    let table_name = format!("Transactions: {lerp_tx_count}");

    let activity_header_cells = ["Created At", "Activity Type"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(theme.background())));

    let activity_tx_header_cells = activity_tx_header_vec
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(theme.background())));

    let activity_header = Row::new(activity_header_cells)
        .style(Style::default().bg(theme.header()))
        .height(1)
        .bottom_margin(0);

    let activity_tx_header = Row::new(activity_tx_header_cells)
        .style(Style::default().bg(theme.header()))
        .height(1)
        .bottom_margin(0);

    let activity_rows = table_data.items.iter().map(|item| {
        let height = 1;
        let cells = item.iter().map(|c| Cell::from(c.separate_with_commas()));
        Row::new(cells)
            .height(height as u16)
            .bottom_margin(0)
            .style(Style::default().bg(theme.background()).fg(theme.text()))
    });

    let activity_tx_rows = activity_txs_table.items.iter().map(|item| {
        let height = 1;
        // First index is the date field. Do not add commas to the value
        // In case search happens by yearly value, this can add comma to the year
        let mut first_index_passed = false;
        let cells = item.iter().map(|c| {
            if first_index_passed {
                Cell::from(c.separate_with_commas())
            } else {
                first_index_passed = true;
                Cell::from(c.clone())
            }
        });
        Row::new(cells)
            .height(height as u16)
            .bottom_margin(0)
            .style(Style::default().bg(theme.background()).fg(theme.text()))
    });

    let mut activity_table_area = Table::new(
        activity_rows,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .header(activity_header)
    .block(styled_block(&table_name, theme));

    let activity_txs_table_area = Table::new(activity_tx_rows, activity_tx_header_widths)
        .header(activity_tx_header)
        .block(styled_block("TX Details", theme));

    let mut month_tab = create_tab(months, "Months", theme);
    let mut year_tab = create_tab(years, "Years", theme);

    match current_tab {
        ActivityTab::Months => {
            month_tab = month_tab.highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(theme.selected()),
            );
        }
        ActivityTab::Years => {
            year_tab = year_tab.highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(theme.selected()),
            );
        }
        ActivityTab::List => {
            if table_data.state.selected().is_some() {
                activity_table_area = activity_table_area
                    .highlight_symbol(">> ")
                    .row_highlight_style(Style::default().bg(theme.selected()));
            }
        }
    }

    if let Some(index) = table_data.state.selected()
        && index > 10
    {
        *table_data.state.offset_mut() = index - 10;
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
