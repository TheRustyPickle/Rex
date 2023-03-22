use crate::home_page::{CurrentUi, PopupState, SelectedTab, TableData, TimeData, TransactionData};
use crate::transfer_page::TransferData;
use crate::tx_page::AddTxData;
use crossterm::event::{KeyCode, KeyEvent};
use rusqlite::Connection;
use std::error::Error;

pub fn home_keys(
    key: KeyEvent,
    cu_page: &mut CurrentUi,
    cu_popup: &mut PopupState,
    data_for_tx: &mut AddTxData,
    data_for_transfer: &mut TransferData,
    all_data: &mut TransactionData,
    table: &mut TableData,
    selected_tab: &mut SelectedTab,
    cu_table_index: Option<usize>,
    cu_month_index: usize,
    cu_year_index: usize,
    months: &mut TimeData,
    years: &mut TimeData,
    conn: &Connection,
) -> Result<String, Box<dyn Error>> {
    match cu_popup {
        PopupState::Nothing => {
            match key.code {
                KeyCode::Char('q') => return Ok("".to_string()),
                KeyCode::Char('a') => *cu_page = CurrentUi::AddTx,
                KeyCode::Char('t') => *cu_page = CurrentUi::Transfer,
                KeyCode::Char('r') => *cu_page = CurrentUi::Chart,
                KeyCode::Char('j') => return Ok("Change".to_string()),
                KeyCode::Char('h') => *cu_popup = PopupState::Helper,
                KeyCode::Char('z') => *cu_page = CurrentUi::Summary,
                KeyCode::Char('e') => {
                    if let Some(a) = cu_table_index {
                        let target_data = &all_data.get_txs()[a];
                        let target_id_num = all_data.get_id_num(a);
                        let tx_type = &target_data[4];

                        // based on what kind of transaction is selected, passes the tx data to the struct
                        // and changes the current interface
                        if tx_type != "Transfer" {
                            *data_for_tx = AddTxData::custom(
                                &target_data[0],
                                &target_data[1],
                                &target_data[2],
                                &target_data[3],
                                &target_data[4],
                                &target_data[5],
                                target_id_num,
                            );
                            *cu_page = CurrentUi::AddTx;
                        } else {
                            let from_to = target_data[2].split(" to ").collect::<Vec<&str>>();
                            let from_method = from_to[0];
                            let to_method = from_to[1];

                            *data_for_transfer = TransferData::custom(
                                &target_data[0],
                                &target_data[1],
                                from_method,
                                to_method,
                                &target_data[3],
                                &target_data[5],
                                target_id_num,
                            );
                            *cu_page = CurrentUi::Transfer;
                        }
                    }
                }
                KeyCode::Char('d') => {
                    if table.state.selected().is_some() {
                        let status = all_data.del_tx(table.state.selected().unwrap());
                        match status {
                            Ok(_) => {
                                // transaction deleted so reload the data again
                                *all_data =
                                    TransactionData::new(conn, cu_month_index, cu_year_index);
                                *table = TableData::new(all_data.get_txs());
                                table.state.select(None);
                                *selected_tab = SelectedTab::Months;
                            }
                            Err(err) => {
                                *cu_popup = PopupState::DeleteFailed(err.to_string());
                            }
                        }
                    }
                }
                KeyCode::Right => match &selected_tab {
                    SelectedTab::Months => months.next(),
                    SelectedTab::Years => {
                        years.next();
                        months.index = 0;
                    }
                    _ => {}
                },
                KeyCode::Left => match &selected_tab {
                    SelectedTab::Months => months.previous(),
                    SelectedTab::Years => {
                        years.previous();
                        months.index = 0;
                    }
                    _ => {}
                },
                KeyCode::Up => {
                    match &selected_tab {
                        SelectedTab::Table => {
                            // Do not select any table rows in the table section If
                            // there is no transaction
                            if all_data.all_tx.is_empty() {
                                *selected_tab = selected_tab.change_tab_up();
                            }
                            // executes when going from first table row to month widget
                            else if table.state.selected() == Some(0) {
                                *selected_tab = SelectedTab::Months;
                                table.state.select(None);
                            } else if !all_data.all_tx.is_empty() {
                                table.previous();
                            }
                        }
                        SelectedTab::Years => {
                            // Do not select any table rows in the table section If
                            // there is no transaction
                            if all_data.all_tx.is_empty() {
                                *selected_tab = selected_tab.change_tab_up();
                            } else {
                                // Move to the selected value on table/Transaction widget
                                // to the last row if pressed up on Year section
                                table.state.select(Some(table.items.len() - 1));
                                *selected_tab = selected_tab.change_tab_up();
                                if all_data.all_tx.is_empty() {
                                    *selected_tab = selected_tab.change_tab_up();
                                }
                            }
                        }
                        _ => *selected_tab = selected_tab.change_tab_up(),
                    }
                }
                KeyCode::Down => {
                    match &selected_tab {
                        SelectedTab::Table => {
                            // Do not proceed to the table section If
                            // there is no transaction
                            if all_data.all_tx.is_empty() {
                                *selected_tab = selected_tab.change_tab_down();
                            }
                            // executes when pressed on last row of the table
                            // moves to the year widget
                            else if table.state.selected() == Some(table.items.len() - 1) {
                                *selected_tab = SelectedTab::Years;
                                table.state.select(None);
                            } else if !all_data.all_tx.is_empty() {
                                table.next();
                            }
                        }
                        SelectedTab::Months => {
                            if all_data.all_tx.is_empty() {
                                *selected_tab = selected_tab.change_tab_up();
                            } else {
                                *selected_tab = selected_tab.change_tab_down();
                                table.state.select(Some(0));
                            };
                        }
                        _ => *selected_tab = selected_tab.change_tab_down(),
                    }
                }
                _ => {}
            }
        }
        _ => *cu_popup = PopupState::Nothing,
    }
    Ok("0".to_string())
}
