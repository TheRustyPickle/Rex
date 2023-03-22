use crate::home_page::{CurrentUi, PopupState, TableData, TxTab};
use crate::tx_page::AddTxData;
use crossterm::event::{KeyCode, KeyEvent};
use std::error::Error;

pub fn summary_keys(
    key: KeyEvent,
    cu_page: &mut CurrentUi,
    cu_popup: &mut PopupState,
    cu_tx_page: &mut TxTab,
    data_for_tx: &mut AddTxData,
    summary_table: &mut TableData,
    total_index: usize,
) -> Result<String, Box<dyn Error>> {
    match cu_popup {
        PopupState::Nothing => match key.code {
            KeyCode::Char('q') => return Ok("".to_string()),
            KeyCode::Char('f') => {
                // * returns to home page and reloads data
                *cu_page = CurrentUi::Home;
                *cu_tx_page = TxTab::Nothing;
                *data_for_tx = AddTxData::new();
            }
            KeyCode::Char('h') => *cu_popup = PopupState::Helper,
            KeyCode::Up => {
                if total_index > 0 {
                    if summary_table.state.selected() == Some(0) {
                        summary_table.state.select(Some(total_index - 1));
                    } else {
                        summary_table.previous();
                    }
                } else {
                    summary_table.state.select(None)
                }
            }
            KeyCode::Down => {
                if total_index > 0 {
                    if summary_table.state.selected() == Some(total_index - 1) {
                        summary_table.state.select(Some(0));
                    } else {
                        summary_table.next();
                    }
                } else {
                    summary_table.state.select(None)
                }
            }
            _ => {}
        },
        _ => *cu_popup = PopupState::Nothing,
    }

    Ok("0".to_string())
}
