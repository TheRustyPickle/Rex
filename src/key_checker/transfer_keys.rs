use crate::key_checker::InputKeyHandler;
use crate::outputs::HandlingOutput;
use crate::page_handler::{PopupState, TransferTab};
use crossterm::event::KeyCode;

pub fn transfer_keys(handler: &mut InputKeyHandler) -> Option<HandlingOutput> {
    match handler.popup {
        PopupState::Nothing => match handler.transfer_tab {
            // start matching key pressed based on which widget is selected.
            // current state tracked with enums
            TransferTab::Nothing => match handler.key.code {
                KeyCode::Char('a') => handler.go_add_tx(),
                KeyCode::Char('q') => return Some(HandlingOutput::QuitUi),
                KeyCode::Char('f') => handler.go_home(),
                KeyCode::Char('h') => handler.do_help_popup(),
                KeyCode::Char('s') => handler.add_transfer_tx(),
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
                _ => match handler.transfer_tab {
                    TransferTab::Date => handler.handle_date(),
                    TransferTab::Details => handler.handle_details(),
                    TransferTab::From => handler.handle_tx_method(),
                    TransferTab::To => handler.handle_tx_method(),
                    TransferTab::Amount => handler.handle_amount(),
                    TransferTab::Tags => handler.handle_tags(),
                    _ => {}
                },
            },
        },

        _ => handler.do_empty_popup(),
    }

    None
}
