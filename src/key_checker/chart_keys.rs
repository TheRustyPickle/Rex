use crossterm::event::KeyCode;

use crate::key_checker::InputKeyHandler;
use crate::outputs::HandlingOutput;
use crate::page_handler::PopupState;

/// Tracks the keys of the Chart page and calls relevant function based on it
#[cfg(not(tarpaulin_include))]
pub fn chart_keys(handler: &mut InputKeyHandler) -> Option<HandlingOutput> {
    match handler.popup {
        PopupState::Nothing => match handler.key.code {
            KeyCode::Char('a') => handler.go_add_tx(),
            KeyCode::Char('z') => handler.go_summary(),
            KeyCode::Char('q') => return Some(HandlingOutput::QuitUi),
            KeyCode::Char('f') => handler.go_home(),
            KeyCode::Char('h') => handler.do_help_popup(),
            KeyCode::Char('r') => handler.do_chart_hidden_mode(),
            KeyCode::Char('R') => handler.do_chart_lgeneds(),
            KeyCode::Char('w') => handler.go_search(),
            KeyCode::Char('y') => handler.go_activity(),
            KeyCode::Right => handler.handle_right_arrow(),
            KeyCode::Left => handler.handle_left_arrow(),
            KeyCode::Up => handler.handle_up_arrow(),
            KeyCode::Down => handler.handle_down_arrow(),
            KeyCode::Char(' ') => handler.switch_chart_tx_method_activation(),
            _ => {}
        },
        PopupState::ChartHelp => match handler.key.code {
            KeyCode::Up => handler.popup_scroll_up(),
            KeyCode::Down => handler.popup_scroll_down(),
            _ => handler.do_empty_popup(),
        },
        _ => handler.do_empty_popup(),
    }
    None
}
