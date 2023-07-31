use crate::key_checker::InputKeyHandler;
use crate::outputs::HandlingOutput;
use crate::page_handler::{PopupState, TxTab};
use crossterm::event::KeyCode;

/// Tracks the keys of the Add Tx page and calls relevant function based on it
#[cfg(not(tarpaulin_include))]
pub fn search_keys(handler: &mut InputKeyHandler) -> Option<HandlingOutput> {
    match handler.popup {
        // we don't want to move this interface while the popup is on
        PopupState::Nothing => match handler.search_tab {
            TxTab::Nothing => match handler.key.code {
                KeyCode::Char('a') => handler.go_add_tx(),
                KeyCode::Char('r') => handler.go_chart(),
                KeyCode::Char('z') => handler.go_summary(),
                KeyCode::Char('q') => return Some(HandlingOutput::QuitUi),
                KeyCode::Char('f') => handler.go_home(),
                KeyCode::Char('h') => handler.do_help_popup(),
                KeyCode::Char('s') => handler.search_tx(),
                KeyCode::Char('c') => handler.clear_input(),
                KeyCode::Char('x') => handler.change_search_date_type(),
                KeyCode::Up => handler.handle_up_arrow(),
                KeyCode::Down => handler.handle_down_arrow(),
                KeyCode::Enter => handler.select_date_field(),
                KeyCode::Char(c) => {
                    if c.is_numeric() {
                        handler.handle_number_press()
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
                _ => match handler.search_tab {
                    TxTab::Date => handler.handle_date(),
                    TxTab::Details => handler.handle_details(),
                    TxTab::FromMethod => handler.handle_tx_method(),
                    TxTab::ToMethod => handler.handle_tx_method(),
                    TxTab::Amount => handler.handle_amount(),
                    TxTab::TxType => handler.handle_tx_type(),
                    TxTab::Tags => handler.handle_tags(),
                    _ => {}
                },
            },
        },
        _ => handler.do_empty_popup(),
    }

    None
}
