use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::Frame;

use crate::page_handler::{HistoryTab, IndexedData, SELECTED};
use crate::utility::{create_tab, main_block};

pub fn history_ui(
    f: &mut Frame,
    months: &IndexedData,
    years: &IndexedData,
    current_tab: &HistoryTab,
) {
    let size = f.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
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
            todo!()
        }
    }

    f.render_widget(year_tab, chunks[0]);
    f.render_widget(month_tab, chunks[1]);
}
