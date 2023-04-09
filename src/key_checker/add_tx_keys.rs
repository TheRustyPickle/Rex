use crate::key_checker::InputKeyHandler;
use crate::outputs::HandlingOutput;
use crate::page_handler::{AddTxTab, PopupState};
use crossterm::event::KeyCode;

/// Tracks the keys once interacting with the Add Transaction interface. Based on the key pressed,
/// calls functions and passes them to a struct
pub fn add_tx_keys(handler: &mut InputKeyHandler) -> Option<HandlingOutput> {
    match handler.popup {
        // we don't want to move this interface while the popup is on
        PopupState::Nothing => match handler.tx_tab {
            AddTxTab::Nothing => match handler.key.code {
                KeyCode::Char('t') => handler.go_transfer(),
                KeyCode::Char('q') => return Some(HandlingOutput::QuitUi),
                KeyCode::Char('f') => handler.go_home(),
                KeyCode::Char('h') => handler.do_help_popup(),
                KeyCode::Char('s') => handler.add_tx(),
                KeyCode::Char(c) => {
                    if c.is_numeric() {
                        handler.handle_number_press()
                    }
                }
                _ => {}
            },
            AddTxTab::Date => handler.handle_date(),
            AddTxTab::Details => handler.handle_details(),
            AddTxTab::TxMethod => handler.handle_tx_method(),
            AddTxTab::Amount => handler.handle_amount(),
            AddTxTab::TxType => handler.handle_tx_type(),
            AddTxTab::Tags => handler.handle_tags(),
        },
        _ => handler.do_empty_popup(),
    }

    None
}
