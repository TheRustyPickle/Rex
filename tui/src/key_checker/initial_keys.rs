use crossterm::event::KeyCode;

use crate::key_checker::InputKeyHandler;
use crate::outputs::HandlingOutput;
use crate::pages::PopupType;

/// Tracks the keys of the Initial page and calls relevant function based on it
pub fn initial_keys(handler: &mut InputKeyHandler) -> Option<HandlingOutput> {
    match handler.popup_status {
        PopupType::Nothing => match handler.key.code {
            KeyCode::Char('q') => return Some(HandlingOutput::QuitUi),
            _ => handler.go_home(),
        },
        PopupType::Info(info) => {
            if info.is_new_update() {
                match handler.key.code {
                    KeyCode::Enter => {
                        if let Err(e) = handler.handle_update_popup() {
                            return Some(e);
                        }
                    }
                    KeyCode::Up => handler.popup_up(),
                    KeyCode::Down => handler.popup_down(),
                    _ => handler.do_empty_popup(),
                }
            }
        }
        _ => handler.do_empty_popup(),
    }
    None
}
