use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Position};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{BorderType, Borders, Clear, Paragraph};

use crate::page_handler::{BACKGROUND, TEXT};
use crate::pages::InputPopup;
use crate::utility::{centered_rect_exact, create_bolded_text, main_block, styled_block};

impl InputPopup {
    pub fn show_ui(&mut self, f: &mut Frame) {
        let size = f.area();
        let x_value = 50;
        let y_value = 7;

        let title = if self.modifying_method.is_none() {
            "New Method"
        } else {
            "Rename to"
        };

        let title = Span::styled(title, Style::default().add_modifier(Modifier::BOLD));

        let status_text = format!("Status: {}", self.status);

        let status_text = create_bolded_text(&status_text);

        let block = main_block()
            .border_type(BorderType::Rounded)
            .title(title)
            .borders(Borders::ALL);

        let area = centered_rect_exact(x_value, y_value, size);

        let new_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Length(3), Constraint::Length(3)])
            .split(area);

        f.render_widget(Clear, area);
        f.render_widget(block, area);

        let input_text = Line::from(vec![Span::from(format!("{} ", self.text))]);

        let input_section = Paragraph::new(input_text)
            .style(Style::default().bg(BACKGROUND).fg(TEXT))
            .block(styled_block("Method name"))
            .alignment(Alignment::Left);

        let status_section = Paragraph::new(status_text)
            .style(Style::default().bg(BACKGROUND).fg(TEXT))
            .alignment(Alignment::Left);

        f.set_cursor_position(Position {
            x: new_chunks[0].x + self.cursor_position as u16 + 1,
            y: new_chunks[0].y + 1,
        });

        f.render_widget(input_section, new_chunks[0]);
        f.render_widget(status_section, new_chunks[1]);
    }
}
