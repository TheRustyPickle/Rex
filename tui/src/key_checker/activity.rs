use anyhow::Result;
use crossterm::event::KeyCode;

use crate::key_checker::{InputKeyHandler, popup_keys};
use crate::outputs::HandlingOutput;
use crate::pages::PopupType;

pub fn activity_keys(handler: &mut InputKeyHandler) -> Result<Option<HandlingOutput>> {
    match handler.popup_status {
        PopupType::Nothing => match handler.key.code {
            KeyCode::Char('q') => return Ok(Some(HandlingOutput::QuitUi)),
            KeyCode::Char('f') => handler.go_home(),
            KeyCode::Char('a') => handler.go_add_tx()?,
            KeyCode::Char('r') => handler.go_chart(),
            KeyCode::Char('h') => handler.do_help_popup(),
            KeyCode::Char('z') => handler.go_summary()?,
            KeyCode::Char('w') => handler.go_search(),
            KeyCode::Char('v') => handler.show_activity_tx_details()?,
            KeyCode::Right => handler.handle_right_arrow()?,
            KeyCode::Left => handler.handle_left_arrow()?,
            KeyCode::Up => handler.handle_up_arrow(),
            KeyCode::Down => handler.handle_down_arrow(),
            _ => {}
        },
        _ => return popup_keys(handler),
    }

    Ok(None)
}
