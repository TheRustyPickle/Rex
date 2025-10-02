use crossterm::event::KeyCode;

use crate::key_checker::InputKeyHandler;
use crate::outputs::HandlingOutput;
use crate::page_handler::TxTab;
use crate::pages::PopupType;

/// Tracks the keys of the Add Tx page and calls relevant function based on it
pub fn add_tx_keys(handler: &mut InputKeyHandler) -> Option<HandlingOutput> {
    match handler.popup_status {
        // We don't want to move this interface while the popup is on
        PopupType::Nothing => match handler.add_tx_tab {
            TxTab::Nothing => match handler.key.code {
                KeyCode::Char('q') => return Some(HandlingOutput::QuitUi),
                KeyCode::Char('f') => handler.go_home(),
                KeyCode::Char('r') => handler.go_chart(),
                KeyCode::Char('z') => handler.go_summary(),
                KeyCode::Char('h') => handler.do_help_popup(),
                KeyCode::Char('s') => handler.add_tx(),
                KeyCode::Char('w') => handler.go_search(),
                KeyCode::Char('c') => handler.clear_input(),
                KeyCode::Char('y') => handler.go_activity(),
                KeyCode::Enter => handler.select_date_field(),
                KeyCode::Char(c) => {
                    if c.is_numeric() {
                        handler.handle_number_press();
                    }
                }
                _ => {}
            },
            _ => match handler.key.code {
                KeyCode::Right => handler.handle_right_arrow(),
                KeyCode::Left => handler.handle_left_arrow(),
                KeyCode::Up => handler.handle_up_arrow(),
                KeyCode::Down => handler.handle_down_arrow(),
                KeyCode::Tab => handler.do_autofill(),
                _ => match handler.add_tx_tab {
                    TxTab::Date => handler.handle_date(),
                    TxTab::Details => handler.handle_details(),
                    TxTab::FromMethod | TxTab::ToMethod => handler.handle_tx_method(),
                    TxTab::Amount => handler.handle_amount(),
                    TxTab::TxType => handler.handle_tx_type(),
                    TxTab::Tags => handler.handle_tags(),
                    TxTab::Nothing => {}
                },
            },
        },
        PopupType::Info(_) => match handler.key.code {
            KeyCode::Up => handler.popup_up(),
            KeyCode::Down => handler.popup_down(),
            _ => handler.do_empty_popup(),
        },
        _ => handler.do_empty_popup(),
    }

    None
}
