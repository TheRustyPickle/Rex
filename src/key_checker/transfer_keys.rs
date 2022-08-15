use crate::home_page::{
    CurrentUi, PopupState, SelectedTab, TableData, TransactionData, TransferTab,
};
use crate::tx_page::AddTxData;
use crossterm::event::{KeyCode, KeyEvent};
use rusqlite::Connection;
use std::error::Error;

pub fn transfer_checker(
    key: KeyEvent,
    cu_page: &mut CurrentUi,
    cu_popup: &mut PopupState,
    cu_tx_page: &mut TransferTab,
    data_for_tx: &mut AddTxData,
    all_data: &mut TransactionData,
    table: &mut TableData,
    selected_tab: &mut SelectedTab,
    cu_month_index: usize,
    cu_year_index: usize,
    conn: &Connection,
) -> Result<String, Box<dyn Error>> {
    match cu_popup {
        PopupState::Nothing => {
            match cu_tx_page {
                // start matching key pressed based on which widget is selected.
                // current state tracked with enums
                TransferTab::Nothing => match key.code {
                    KeyCode::Char('q') => return Ok("".to_string()),
                    KeyCode::Char('f') => {
                        *cu_page = CurrentUi::Home;
                        *cu_tx_page = TransferTab::Nothing;
                        *data_for_tx = AddTxData::new();
                    }
                    KeyCode::Char('h') => *cu_popup = PopupState::Helper,
                    KeyCode::Char('s') => {
                        let status = data_for_tx.add_tx();
                        if status == "".to_string() {
                            // reload home page and switch UI
                            *selected_tab = SelectedTab::Months;
                            *data_for_tx = AddTxData::new();
                            *all_data = TransactionData::new(&conn, cu_month_index, cu_year_index);
                            *table = TableData::new(all_data.get_txs());
                            *cu_page = CurrentUi::Home;
                        } else {
                            data_for_tx.add_tx_status(&status);
                        }
                    }
                    KeyCode::Char('1') => *cu_tx_page = TransferTab::Date,
                    KeyCode::Char('2') => *cu_tx_page = TransferTab::Details,
                    KeyCode::Char('3') => *cu_tx_page = TransferTab::To,
                    KeyCode::Char('4') => *cu_tx_page = TransferTab::Amount,
                    KeyCode::Char('5') => *cu_tx_page = TransferTab::From,
                    KeyCode::Enter => *cu_tx_page = TransferTab::Nothing,
                    KeyCode::Esc => *cu_tx_page = TransferTab::Nothing,
                    _ => {}
                },

                TransferTab::Date => match key.code {
                    KeyCode::Enter => {
                        let status = data_for_tx.check_date();
                        match status {
                            Ok(a) => {
                                data_for_tx.add_tx_status(&a);
                                if a.contains("Accepted") || a.contains("Nothing") {
                                    *cu_tx_page = TransferTab::Details
                                }
                            }
                            Err(_) => data_for_tx
                                .add_tx_status("Date: Error acquired or Date not acceptable."),
                        }
                    }
                    KeyCode::Esc => {
                        let status = data_for_tx.check_date();
                        match status {
                            Ok(a) => {
                                data_for_tx.add_tx_status(&a);
                                if a.contains("Accepted") || a.contains("Nothing") {
                                    *cu_tx_page = TransferTab::Nothing
                                }
                            }
                            Err(_) => data_for_tx
                                .add_tx_status("Date: Error acquired or Date not acceptable."),
                        }
                    }
                    KeyCode::Backspace => data_for_tx.edit_date('a', true),
                    KeyCode::Char(a) => data_for_tx.edit_date(a, false),
                    _ => {}
                },

                TransferTab::Details => match key.code {
                    KeyCode::Enter => *cu_tx_page = TransferTab::From,
                    KeyCode::Esc => *cu_tx_page = TransferTab::Nothing,
                    KeyCode::Backspace => data_for_tx.edit_details('a', true),
                    KeyCode::Char(a) => data_for_tx.edit_details(a, false),
                    _ => {}
                },

                TransferTab::From => match key.code {
                    KeyCode::Enter => {
                        let status = data_for_tx.check_tx_method(&conn);

                        match status {
                            Ok(a) => {
                                data_for_tx.add_tx_status(&a);
                                if a.contains("Accepted") || a.contains("Nothing") {
                                    *cu_tx_page = TransferTab::Amount
                                }
                            }
                            Err(_) => data_for_tx
                                .add_tx_status("TX Method: Error acquired while checking."),
                        }
                    }
                    KeyCode::Esc => {
                        let status = data_for_tx.check_tx_method(&conn);

                        match status {
                            Ok(a) => {
                                data_for_tx.add_tx_status(&a);
                                if a.contains("Accepted") || a.contains("Nothing") {
                                    *cu_tx_page = TransferTab::Nothing
                                }
                            }
                            Err(_) => data_for_tx
                                .add_tx_status("TX Method: Error acquired while checking."),
                        }
                    }
                    KeyCode::Backspace => data_for_tx.edit_tx_method('a', true),
                    KeyCode::Char(a) => data_for_tx.edit_tx_method(a, false),
                    _ => {}
                },

                TransferTab::Amount => match key.code {
                    KeyCode::Enter => {
                        let status = data_for_tx.check_amount();
                        match status {
                            Ok(a) => {
                                data_for_tx.add_tx_status(&a);
                                if a.contains("zero") {
                                } else {
                                    *cu_tx_page = TransferTab::From;
                                }
                            }
                            Err(_) => data_for_tx.add_tx_status("Amount: Invalid Amount found"),
                        }
                    }
                    KeyCode::Esc => {
                        let status = data_for_tx.check_amount();
                        match status {
                            Ok(a) => {
                                data_for_tx.add_tx_status(&a);
                                if a.contains("zero") {
                                } else {
                                    *cu_tx_page = TransferTab::Nothing;
                                }
                            }
                            Err(_) => data_for_tx.add_tx_status("Amount: Invalid Amount found"),
                        }
                    }
                    KeyCode::Backspace => data_for_tx.edit_amount('a', true),
                    KeyCode::Char(a) => data_for_tx.edit_amount(a, false),
                    _ => {}
                },

                TransferTab::To => {
                    match key.code {
                        KeyCode::Enter => {
                            let status = data_for_tx.check_tx_type();
                            match status {
                                Ok(a) => {
                                    data_for_tx.add_tx_status(&a);
                                    if a.contains("Accepted") || a.contains("Nothing") {
                                        *cu_tx_page = TransferTab::Nothing
                                    }
                                }
                                Err(_) => data_for_tx
                                    .add_tx_status("TX Type: Invalid Transaction Type Found"),
                            }
                        }
                        KeyCode::Esc => {
                            let status = data_for_tx.check_tx_type();
                            match status {
                                Ok(a) => {
                                    data_for_tx.add_tx_status(&a);
                                    if a.contains("Accepted") || a.contains("Nothing") {
                                        *cu_tx_page = TransferTab::Nothing
                                    }
                                }
                                Err(_) => data_for_tx
                                    .add_tx_status("TX Type: Invalid Transaction Type Found"),
                            }
                        }
                        KeyCode::Backspace => data_for_tx.edit_tx_type('a', true),
                        KeyCode::Char(a) => data_for_tx.edit_tx_type(a, false),
                        _ => {}
                    }
                }
            }
        }
        _ => match key.code {
            _ => *cu_popup = PopupState::Nothing,
        },
    }

    Ok("0".to_string())
}
