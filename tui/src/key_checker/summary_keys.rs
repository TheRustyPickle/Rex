use anyhow::Result;
use crossterm::event::KeyCode;

use crate::key_checker::InputKeyHandler;
use crate::outputs::HandlingOutput;
use crate::pages::PopupType;

/// Tracks the keys of the Summary page and calls relevant function based on it
pub fn summary_keys(handler: &mut InputKeyHandler) -> Result<Option<HandlingOutput>> {
    match handler.popup_status {
        PopupType::Nothing => match handler.key.code {
            KeyCode::Char('a') => handler.go_add_tx()?,
            KeyCode::Char('r') => handler.go_chart(),
            KeyCode::Char('q') => return Ok(Some(HandlingOutput::QuitUi)),
            KeyCode::Char('f') => handler.go_home(),
            KeyCode::Char('w') => handler.go_search(),
            KeyCode::Char('h') => handler.do_help_popup(),
            KeyCode::Char('z') => handler.do_summary_hidden_mode(),
            KeyCode::Char('x') => handler.change_summary_sort(),
            KeyCode::Char('y') => handler.go_activity(),
            KeyCode::Right => handler.handle_right_arrow()?,
            KeyCode::Left => handler.handle_left_arrow()?,
            KeyCode::Up => handler.handle_up_arrow(),
            KeyCode::Down => handler.handle_down_arrow(),
            KeyCode::Enter => handler.search_tag()?,
            _ => {}
        },
        PopupType::Info(_) => match handler.key.code {
            KeyCode::Up => handler.popup_up(),
            KeyCode::Down => handler.popup_down(),
            _ => handler.do_empty_popup(),
        },
        _ => handler.do_empty_popup(),
    }

    Ok(None)
}
