use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{BorderType, Borders, Clear, Row, Table};

use crate::page_handler::{BLUE, BOX, TEXT};
use crate::pages::RepositionPopup;
use crate::utility::{centered_rect, main_block, styled_block};

impl RepositionPopup {
    pub fn show_ui(&mut self, f: &mut Frame) {
        let size = f.area();
        let x_value = 40;
        let y_value = 50;

        let title = "Reposition Tx Methods";

        let title = Span::styled(title, Style::default().add_modifier(Modifier::BOLD));

        let block = main_block()
            .border_type(BorderType::Rounded)
            .title(title)
            .borders(Borders::ALL);

        let area = centered_rect(x_value, y_value, size);

        let new_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Min(1), Constraint::Length(3)])
            .split(area);

        f.render_widget(Clear, area);
        f.render_widget(block, area);

        let reposition_rows = self
            .reposition_table
            .items
            .iter()
            .map(|r| Row::new(r.clone()).style(Style::default().fg(TEXT)));

        let confirmation_rows = self
            .confirm_table
            .items
            .iter()
            .map(|r| Row::new(r.clone()).style(Style::default().fg(TEXT)));

        let mut reposition_table = Table::new(reposition_rows, [Constraint::Percentage(100)])
            .block(styled_block("H for help"))
            .style(Style::default().fg(BOX));

        let mut confirmation_table = Table::new(confirmation_rows, [Constraint::Percentage(100)])
            .block(styled_block("Confirm"))
            .style(Style::default().fg(BOX));

        let highlight_style = Style::default().fg(BLUE).add_modifier(Modifier::REVERSED);

        if self.reposition_selected {
            reposition_table = reposition_table
                .row_highlight_style(highlight_style)
                .highlight_symbol(">> ");
        } else {
            confirmation_table = confirmation_table
                .row_highlight_style(highlight_style)
                .highlight_symbol(">> ");
        }

        f.render_stateful_widget(
            reposition_table,
            new_chunks[0],
            &mut self.reposition_table.state,
        );

        f.render_stateful_widget(
            confirmation_table,
            new_chunks[1],
            &mut self.confirm_table.state,
        );
    }
}
