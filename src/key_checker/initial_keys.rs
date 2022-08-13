use crate::home_page::{CurrentUi, PopupState};
use crossterm::event::{KeyCode, KeyEvent};
use std::error::Error;

pub fn initial_checker(
    key: KeyEvent,
    cu_page: &mut CurrentUi,
    cu_popup: &mut PopupState,
) -> Result<String, Box<dyn Error>> {
    match cu_popup {
        PopupState::Nothing => match key.code {
            KeyCode::Char('q') => return Ok("".to_string()),
            _ => *cu_page = CurrentUi::Home,
        },
        PopupState::NewUpdate => {
            match key.code {
                KeyCode::Enter => {
                    match open::that("https://github.com/WaffleMixer/Rex/releases/latest") {
                        Ok(_) => *cu_popup = PopupState::Nothing,

                        // if it fails for any reason, break interface and print the link
                        Err(_) => return Ok("Link".to_string()),
                    }
                }
                _ => *cu_popup = PopupState::Nothing,
            }
        }
        _ => *cu_popup = PopupState::Nothing,
    }
    Ok("0".to_string())
}
