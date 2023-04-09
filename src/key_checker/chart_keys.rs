use crate::key_checker::InputKeyHandler;
use crate::outputs::HandlingOutput;
use crate::page_handler::PopupState;
use crossterm::event::KeyCode;

pub fn chart_keys(handler: &mut InputKeyHandler) -> Option<HandlingOutput> {
    match handler.popup {
        PopupState::Nothing => match handler.key.code {
            KeyCode::Char('a') => handler.go_add_tx(),
            KeyCode::Char('t') => handler.go_transfer(),
            KeyCode::Char('q') => return Some(HandlingOutput::QuitUi),
            KeyCode::Char('f') => handler.go_home(),
            KeyCode::Char('h') => handler.do_help_popup(),
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
