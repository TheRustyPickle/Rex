use crate::home_page::{CurrentUi, PopupState, TxTab};
use crate::tx_page::AddTxData;
use crossterm::event::{KeyCode, KeyEvent};
use std::error::Error;

pub fn chart_keys(
    key: KeyEvent,
    cu_page: &mut CurrentUi,
    cu_popup: &mut PopupState,
    cu_tx_page: &mut TxTab,
    data_for_tx: &mut AddTxData,
) -> Result<String, Box<dyn Error>> {
    match cu_popup {
        PopupState::Nothing => match key.code {
            KeyCode::Char('q') => return Ok("".to_string()),
            KeyCode::Char('f') => {
                // returns to home page and reloads data
                *cu_page = CurrentUi::Home;
                *cu_tx_page = TxTab::Nothing;
                *data_for_tx = AddTxData::new();
            }
            KeyCode::Char('h') => *cu_popup = PopupState::Helper,
            _ => {}
        },
        _ => match key.code {
            _ => *cu_popup = PopupState::Nothing,
        },
    }
    Ok("0".to_string())
}
