use anyhow::Result;
use crossterm::event::KeyCode;

use crate::key_checker::{InputKeyHandler, popup_keys};
use crate::outputs::HandlingOutput;
use crate::pages::PopupType;

/// Tracks the keys of the Initial page and calls relevant function based on it
pub fn initial_keys(handler: &mut InputKeyHandler) -> Result<Option<HandlingOutput>> {
    if let PopupType::Nothing = handler.popup_status { match handler.key.code {
        KeyCode::Char('q') => return Ok(Some(HandlingOutput::QuitUi)),
        _ => handler.go_home(),
    } } else {
        if let KeyCode::Char('q') = handler.key.code {
            return Ok(Some(HandlingOutput::QuitUi));
        }
        return popup_keys(handler);
    }
    Ok(None)
}
