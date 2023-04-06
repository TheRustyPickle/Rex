use crate::key_checker::InputKeyHandler;
use crate::outputs::HandlingOutput;
use crate::ui_handler::PopupState;
use crossterm::event::KeyCode;

pub fn summary_keys(handler: &mut InputKeyHandler) -> Option<HandlingOutput> {
    match handler.current_popup {
        PopupState::Nothing => match handler.key.code {
            KeyCode::Char('q') => return Some(HandlingOutput::QuitUi),
            KeyCode::Char('f') => handler.go_home(),
            KeyCode::Char('h') => handler.do_help_popup(),
            KeyCode::Up => handler.handle_up_arrow(),
            KeyCode::Down => handler.handle_down_arrow(),
            _ => {}
        },
        _ => handler.do_empty_popup(),
    }

    None
}
