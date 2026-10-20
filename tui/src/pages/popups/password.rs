use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Position};
use ratatui::style::{Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{BorderType, Borders, Clear, Paragraph, Wrap};

use crate::pages::{PasswordPopup, PasswordStage};
use crate::theme::Theme;
use crate::utility::{centered_rect_exact, main_block, styled_block};

impl PasswordPopup {
    pub fn show_ui(&mut self, f: &mut Frame, theme: &Theme) {
        let area = centered_rect_exact(56, 12, f.area());

        let title = if self.changing {
            "Change Password"
        } else {
            "Set Password"
        };

        let block_name = match self.stage {
            PasswordStage::Enter => "New password",
            PasswordStage::Confirm => "Confirm password",
        };

        let block = main_block(theme)
            .border_type(BorderType::Rounded)
            .title(Span::styled(
                title,
                Style::default().add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(2),
                Constraint::Min(1),
            ])
            .split(area);

        f.render_widget(Clear, area);
        f.render_widget(block, area);

        let masked = "*".repeat(self.text.chars().count());
        let cursor_x = chunks[0].x + masked.len() as u16 + 1;

        f.render_widget(
            Paragraph::new(masked)
                .style(Style::default().bg(theme.background()).fg(theme.text()))
                .block(styled_block(block_name, theme)),
            chunks[0],
        );

        f.set_cursor_position(Position {
            x: cursor_x,
            y: chunks[0].y + 1,
        });

        f.render_widget(
            Paragraph::new(format!("Status: {}", self.status))
                .style(
                    Style::default()
                        .bg(theme.background())
                        .fg(theme.text())
                        .add_modifier(Modifier::BOLD),
                )
                .wrap(Wrap { trim: true }),
            chunks[1],
        );

        f.render_widget(
            Paragraph::new(
                "The database gets encrypted. A forgotten password cannot be recovered. Esc to cancel",
            )
            .style(Style::default().bg(theme.background()).fg(theme.negative()))
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Left),
            chunks[2],
        );
    }
}
