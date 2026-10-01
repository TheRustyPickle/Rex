use anyhow::Result;
use crossterm::event::KeyCode;

use crate::key_checker::{InputKeyHandler, popup_keys};
use crate::outputs::HandlingOutput;
use crate::page_handler::TxTab;
use crate::pages::PopupType;

/// Tracks the keys of the Recurring Transactions page and calls relevant function based on it
pub fn recurring_keys(handler: &mut InputKeyHandler) -> Result<Option<HandlingOutput>> {
    match handler.popup_status {
        // We don't want to move this interface while the popup is on
        PopupType::Nothing => match handler.recurring_tab {
            TxTab::Nothing => match handler.key.code {
                KeyCode::Char('a') => handler.go_add_tx()?,
                KeyCode::Char('r') => handler.go_chart(),
                KeyCode::Char('z') => handler.go_summary()?,
                KeyCode::Char('q') => return Ok(Some(HandlingOutput::QuitUi)),
                KeyCode::Char('j') => handler.do_config_popup(),
                KeyCode::Char('f') => handler.go_home(),
                KeyCode::Char('h') => handler.do_help_popup(),
                KeyCode::Char('s') => handler.save_recurring_tx()?,
                KeyCode::Char('c') => handler.clear_input()?,
                KeyCode::Char('e') => handler.recurring_edit_selected(),
                KeyCode::Char('d') => handler.do_deletion_popup(),
                KeyCode::Char('p') => handler.recurring_toggle_pause()?,
                KeyCode::Char('w') => handler.go_search(),
                KeyCode::Char('y') => handler.go_activity(),
                KeyCode::Char('t') => handler.next_theme()?,
                KeyCode::Up => handler.handle_up_arrow(),
                KeyCode::Down => handler.handle_down_arrow(),
                KeyCode::Enter => handler.select_date_field(),
                KeyCode::Char(c) if c.is_numeric() => {
                    handler.handle_number_press();
                }
                _ => {}
            },
            _ => match handler.key.code {
                KeyCode::Right => handler.handle_right_arrow()?,
                KeyCode::Left => handler.handle_left_arrow()?,
                KeyCode::Up => handler.handle_up_arrow(),
                KeyCode::Down => handler.handle_down_arrow(),
                KeyCode::Tab => handler.do_autofill(),
                _ => match handler.recurring_tab {
                    TxTab::Date => handler.handle_date(),
                    TxTab::Details => handler.handle_details(),
                    TxTab::FromMethod | TxTab::ToMethod => handler.handle_tx_method()?,
                    TxTab::Amount => handler.handle_amount()?,
                    TxTab::TxType => handler.handle_tx_type()?,
                    TxTab::Tags => handler.handle_tags(),
                    TxTab::Frequency
                    | TxTab::RecurInterval
                    | TxTab::RecurValue
                    | TxTab::RecurMonth
                    | TxTab::EndDate => handler.handle_recurring_field()?,
                    TxTab::Nothing => {}
                },
            },
        },
        _ => return popup_keys(handler),
    }

    Ok(None)
}
