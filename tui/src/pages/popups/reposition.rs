use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{BorderType, Borders, Clear, Row, Table};

use crate::pages::RepositionPopup;
use crate::theme::Theme;
use crate::utility::{centered_rect_exact, main_block, styled_block};

impl RepositionPopup {
    pub fn show_ui(&mut self, f: &mut Frame, theme: &Theme) {
        let size = f.area();
        let x_value = 40;
        let mut y_value = self.reposition_table.items.len() as u16 + 4 + 3;

        if y_value > 20 {
            y_value = 20;
        }

        let title = "Reposition Tx Methods";

        let title = Span::styled(title, Style::default().add_modifier(Modifier::BOLD));

        let block = main_block(theme)
            .border_type(BorderType::Rounded)
            .title(title)
            .borders(Borders::ALL);

        let area = centered_rect_exact(x_value, y_value, size);

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
            .map(|r| Row::new(r.clone()).style(Style::default().fg(theme.text())));

        let confirmation_rows = self
            .confirm_table
            .items
            .iter()
            .map(|r| Row::new(r.clone()).style(Style::default().fg(theme.text())));

        let mut reposition_table = Table::new(reposition_rows, [Constraint::Percentage(100)])
            .block(styled_block("H for help", theme))
            .style(Style::default().fg(theme.border()));

        let mut confirmation_table = Table::new(confirmation_rows, [Constraint::Percentage(100)])
            .block(styled_block("Confirm", theme))
            .style(Style::default().fg(theme.border()));

        let highlight_style = Style::default()
            .fg(theme.positive())
            .add_modifier(Modifier::REVERSED);

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
