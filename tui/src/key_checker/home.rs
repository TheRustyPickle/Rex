use anyhow::Result;
use crossterm::event::KeyCode;

use crate::key_checker::{InputKeyHandler, popup_keys};
use crate::outputs::HandlingOutput;
use crate::pages::PopupType;

/// Tracks the keys of the Homepage and calls relevant function based on it
pub fn home_keys(handler: &mut InputKeyHandler) -> Result<Option<HandlingOutput>> {
    match handler.popup_status {
        PopupType::Nothing => match handler.key.code {
            KeyCode::Char('q') => return Ok(Some(HandlingOutput::QuitUi)),
            KeyCode::Char('a') => handler.go_add_tx()?,
            KeyCode::Char('r') => handler.go_chart(),
            KeyCode::Char('j') => handler.do_config_popup(),
            KeyCode::Char('h') => handler.do_help_popup(),
            KeyCode::Char('z') => handler.go_summary()?,
            KeyCode::Char('w') => handler.go_search(),
            KeyCode::Char('e') => handler.home_edit_tx()?,
            KeyCode::Char('d') => handler.do_deletion_popup(),
            KeyCode::Char('y') => handler.go_activity(),
            KeyCode::Char(',') => handler.switch_tx_position_up()?,
            KeyCode::Char('.') => handler.switch_tx_position_down()?,
            KeyCode::Char('v') => handler.show_home_tx_details(),
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
