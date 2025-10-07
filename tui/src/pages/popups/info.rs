use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Span, Text};
use ratatui::widgets::{
    BorderType, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap,
};

use crate::page_handler::{BACKGROUND, RED, TEXT};
use crate::pages::{
    InfoPopup, InfoPopupState, activity_help_text, add_tx_help_text, chart_help_text,
    delete_tx_help, home_help_text, new_update_text, reposition_help, search_help_text,
    summary_help_text,
};
use crate::utility::{centered_rect, create_bolded_text, main_block};

impl InfoPopup {
    pub fn show_ui(&mut self, f: &mut Frame) {
        let size = f.area();
        let mut x_value = 60;
        let mut y_value = 60;

        let mut title = "Help";
        let message;

        match &self.showing {
            InfoPopupState::NewUpdate(data) => {
                title = "New Update";
                message = new_update_text(data);
            }
            InfoPopupState::HomeHelp => {
                message = home_help_text();
            }
            InfoPopupState::AddTxHelp => {
                message = add_tx_help_text();
            }
            InfoPopupState::ChartHelp => {
                message = chart_help_text();
            }
            InfoPopupState::SummaryHelp => {
                message = summary_help_text();
            }
            InfoPopupState::SearchHelp => {
                message = search_help_text();
            }
            InfoPopupState::ActivityHelp => {
                message = activity_help_text();
            }
            InfoPopupState::Error(err) => {
                title = "Error";
                message = err.to_string();
            }
            InfoPopupState::ShowDetails(details) => {
                title = "Transaction Details";
                message = details.to_string();

                x_value = 40;
                y_value = 20;
            }
            InfoPopupState::ChoiceHelp => {
                message = delete_tx_help();

                y_value = 20;
            }
            InfoPopupState::RepositionHelp => {
                message = reposition_help();

                y_value = 20;
            }
        }

        if !message.is_empty() && self.max_scroll == 0 {
            let new_line_count = message.split('\n').count();
            self.max_scroll = if new_line_count > 5 {
                new_line_count - 2
            } else {
                new_line_count
            };
        }

        let title = Span::styled(title, Style::default().add_modifier(Modifier::BOLD));

        let text_len = message.split('\n').count() + 5;

        let text = create_bolded_text(&message);

        let area = centered_rect(x_value, y_value, size);

        let block = main_block()
            .title(title)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        let new_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(area);

        f.render_widget(Clear, area);
        f.render_widget(block, area);

        let help_sec = Paragraph::new(Text::from(text))
            .style(Style::default().bg(BACKGROUND).fg(TEXT))
            .wrap(Wrap { trim: true })
            .scroll((self.scroll_position as u16, 0));

        let dismiss_sec =
            Paragraph::new("Use Arrow Keys To Scroll. Press Any Other Key To Dismiss")
                .style(
                    Style::default()
                        .bg(BACKGROUND)
                        .fg(RED)
                        .add_modifier(Modifier::BOLD),
                )
                .alignment(Alignment::Center);

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
        let mut scrollbar_state = ScrollbarState::new(text_len)
            .position(self.scroll_position)
            .content_length(text_len - 5);

        f.render_widget(help_sec, new_chunks[0]);
        f.render_widget(dismiss_sec, new_chunks[1]);
        f.render_stateful_widget(scrollbar, new_chunks[0], &mut scrollbar_state);
    }
}
