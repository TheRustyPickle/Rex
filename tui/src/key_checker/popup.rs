use anyhow::Result;
use crossterm::event::KeyCode;

use crate::key_checker::InputKeyHandler;
use crate::outputs::HandlingOutput;
use crate::pages::PopupType;

pub fn popup_keys(handler: &mut InputKeyHandler) -> Result<Option<HandlingOutput>> {
    match handler.popup_status {
        PopupType::Choice(_) => match handler.key.code {
            KeyCode::Up => handler.popup_up(),
            KeyCode::Down => handler.popup_down(),
            KeyCode::Enter => handler.handle_choice_popup_selection()?,
            KeyCode::Char('h') => handler.do_popup_help_popup(),
            _ => handler.do_empty_popup(),
        },
        PopupType::Info(info) => {
            if info.is_new_update() {
                match handler.key.code {
                    KeyCode::Enter => {
                        if let Err(e) = handler.handle_update_popup() {
                            return Ok(Some(e));
                        }
                    }
                    KeyCode::Up => handler.popup_up(),
                    KeyCode::Down => handler.popup_down(),
                    _ => handler.do_empty_popup(),
                }
            } else {
                match handler.key.code {
                    KeyCode::Up => handler.popup_up(),
                    KeyCode::Down => handler.popup_down(),
                    KeyCode::Enter => handler.handle_choice_popup_selection()?,
                    KeyCode::Char('h') => handler.do_popup_help_popup(),
                    _ => handler.do_empty_popup(),
                }
            }
        }
        PopupType::Reposition(_) => match handler.key.code {
            KeyCode::Up => handler.popup_up(),
            KeyCode::Down => handler.popup_down(),
            KeyCode::Enter => handler.handle_reposition_popup_selection()?,
            KeyCode::Char('h') => handler.do_popup_help_popup(),
            KeyCode::Char(',') => handler.popup_move_up(),
            KeyCode::Char('.') => handler.popup_move_down(),
            _ => handler.do_empty_popup(),
        },
        PopupType::NewPaths(_) => match handler.key.code {
            KeyCode::Up => handler.popup_up(),
            KeyCode::Down => handler.popup_down(),
            KeyCode::Enter => return handler.handle_new_path_popup_selection(),
            KeyCode::Char('h') => handler.do_popup_help_popup(),
            _ => handler.do_empty_popup(),
        },
        PopupType::Input(_) => handler.handle_popup_input()?,
        PopupType::Nothing => unreachable!(),
    }
    Ok(None)
}
