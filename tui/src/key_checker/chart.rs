use anyhow::Result;
use crossterm::event::KeyCode;

use crate::key_checker::{InputKeyHandler, popup_keys};
use crate::outputs::HandlingOutput;
use crate::pages::PopupType;

/// Tracks the keys of the Chart page and calls relevant function based on it
pub fn chart_keys(handler: &mut InputKeyHandler) -> Result<Option<HandlingOutput>> {
    match handler.popup_status {
        PopupType::Nothing => match handler.key.code {
            KeyCode::Char('a') => handler.go_add_tx()?,
            KeyCode::Char('z') => handler.go_summary()?,
            KeyCode::Char('q') => return Ok(Some(HandlingOutput::QuitUi)),
            KeyCode::Char('f') => handler.go_home(),
            KeyCode::Char('h') => handler.do_help_popup(),
            KeyCode::Char('r') => handler.do_chart_hidden_mode(),
            KeyCode::Char('R') => handler.do_chart_lgeneds(),
            KeyCode::Char('w') => handler.go_search(),
            KeyCode::Char('y') => handler.go_activity(),
            KeyCode::Right => handler.handle_right_arrow()?,
            KeyCode::Left => handler.handle_left_arrow()?,
            KeyCode::Up => handler.handle_up_arrow(),
            KeyCode::Down => handler.handle_down_arrow(),
            KeyCode::Char(' ') => handler.switch_chart_tx_method_activation()?,
            _ => {}
        },
        _ => return popup_keys(handler),
    }
    Ok(None)
}
