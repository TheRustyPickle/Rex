use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Span, Text};
use ratatui::widgets::{BorderType, Borders, Clear, Paragraph, Row, Table, Wrap};
use std::fmt::Write;

use crate::page_handler::{BACKGROUND, BLUE, BOX, TEXT};
use crate::pages::NewPathsPopup;
use crate::utility::{centered_rect_exact, main_block, styled_block};

impl NewPathsPopup {
    pub fn show_ui(&mut self, f: &mut Frame) {
        let size = f.area();
        let x_value = 50;
        let y_value = 20;

        let title = if self.new_location {
            "New Location"
        } else {
            "Backup Locations"
        };

        let title = Span::styled(title, Style::default().add_modifier(Modifier::BOLD));

        let block = main_block()
            .border_type(BorderType::Rounded)
            .title(title)
            .borders(Borders::ALL);

        let area = centered_rect_exact(x_value, y_value, size);

        let new_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Min(1), Constraint::Length(5)])
            .split(area);

        f.render_widget(Clear, area);
        f.render_widget(block, area);

        let choice_rows = self
            .table
            .items
            .iter()
            .map(|r| Row::new(r.clone()).style(Style::default().fg(TEXT)));

        let mut path_text = String::new();

        for path in &self.paths {
            writeln!(path_text, "{}", path.display()).unwrap();
        }

        let path_list = Paragraph::new(Text::from(path_text))
            .style(Style::default().bg(BACKGROUND).fg(TEXT))
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Left);

        let highlight_style = Style::default().fg(BLUE).add_modifier(Modifier::REVERSED);

        let table = Table::new(choice_rows, [Constraint::Percentage(100)])
            .highlight_symbol(">> ")
            .block(styled_block("H for help"))
            .row_highlight_style(highlight_style)
            .style(Style::default().fg(BOX));

        f.render_widget(path_list, new_chunks[0]);
        f.render_stateful_widget(table, new_chunks[1], &mut self.table.state);
    }
}
