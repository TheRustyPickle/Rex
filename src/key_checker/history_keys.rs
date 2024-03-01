use crossterm::event::KeyCode;

use crate::key_checker::InputKeyHandler;
use crate::outputs::HandlingOutput;
use crate::page_handler::PopupState;

pub fn history_keys(handler: &mut InputKeyHandler) -> Option<HandlingOutput> {
    match handler.popup {
        PopupState::Nothing => match handler.key.code {
            KeyCode::Char('q') => return Some(HandlingOutput::QuitUi),
            KeyCode::Char('f') => handler.go_home(),
            KeyCode::Char('a') => handler.go_add_tx(),
            KeyCode::Char('r') => handler.go_chart(),
            KeyCode::Char('h') => handler.do_help_popup(),
            KeyCode::Char('z') => handler.go_summary(),
            KeyCode::Char('w') => handler.go_search(),
            KeyCode::Right => handler.handle_right_arrow(),
            KeyCode::Left => handler.handle_left_arrow(),
            KeyCode::Up => handler.handle_up_arrow(),
            KeyCode::Down => handler.handle_down_arrow(),
            _ => {}
        },
        _ => handler.do_empty_popup(),
    }

    None
}
