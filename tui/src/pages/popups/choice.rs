use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Span, Text};
use ratatui::widgets::{BorderType, Borders, Clear, Paragraph, Row, Table, Wrap};

use crate::page_handler::{BACKGROUND, BOX, TEXT};
use crate::pages::ChoicePopup;
use crate::utility::{centered_rect, main_block, styled_block};

impl ChoicePopup {
    pub fn show_ui(&mut self, f: &mut Frame) {
        let size = f.area();
        let x_value = 40;
        let y_value = 10;

        let title = "Transaction Deletion";
        let message = "Are you sure you want to delete this transaction?";

        let title = Span::styled(title, Style::default().add_modifier(Modifier::BOLD));

        let text = Span::styled(message, Style::default().add_modifier(Modifier::BOLD));

        let block = main_block()
            .border_type(BorderType::Rounded)
            .title(title)
            .borders(Borders::ALL);

        let area = centered_rect(x_value, y_value, size);

        let new_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Min(1),
                Constraint::Length(1),
                Constraint::Length(4),
            ])
            .split(area);

        f.render_widget(Clear, area);
        f.render_widget(block, area);

        let deletion_text = Paragraph::new(Text::from(text))
            .style(Style::default().bg(BACKGROUND).fg(TEXT))
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Center);

        let rows = self
            .table
            .items
            .iter()
            .map(|r| Row::new(r.clone()).style(Style::default().fg(TEXT)));

        let mut table = Table::new(rows, [Constraint::Percentage(100)])
            .highlight_symbol(">> ")
            .block(styled_block(
                "Confirmation: Scroll with Arrow keys and Enter to confirm",
            ))
            .style(Style::default().fg(BOX));

        let selected_index = self.table.state.selected().unwrap();

        let target_color = self.choices.get(selected_index).unwrap().color;

        let highlight_style = Style::default()
            .fg(target_color)
            .add_modifier(Modifier::REVERSED);

        table = table.row_highlight_style(highlight_style);

        f.render_widget(deletion_text, new_chunks[0]);
        f.render_stateful_widget(table, new_chunks[2], &mut self.table.state);
    }
}
