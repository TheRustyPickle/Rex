use crate::db::{get_all_tx_methods, get_empty_changes};
use crate::home_page::ui;
use crate::home_page::TransactionData;
use crate::home_page::{CurrentUi, PopupState, SelectedTab, TableData, TimeData, TxTab};
use crate::initial_page::starter_ui;
use crate::key_checker::{add_tx_checker, home_checker, initial_checker};
use crate::popup_page::create_popup;
use crate::transfer_page::transfer_ui_func;
use crate::tx_page::tx_ui;
use crate::tx_page::AddTxData;
use crossterm::event::poll;
use crossterm::event::{self, Event, KeyCode};
use rusqlite::Connection;
use std::{error::Error, time::Duration};
use tui::layout::Constraint;
use tui::{backend::Backend, Terminal};

/// run_app is the core part that makes the entire program run. It basically loops
/// incredibly fast to refresh the terminal and passes the provided data to ui modules to draw them.
/// While the loop is running, the program executes, gets the data from the db and key presses to
/// To keep on providing new data to the UI.
pub fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut months: TimeData,
    mut years: TimeData,
    new_version_available: bool,
) -> Result<String, Box<dyn Error>> {
    // Setting up some default values. Let's go through all of them
    // selected_tab : Basically the current selected widget/field. Default set to the month selection/3rd widget
    //
    // last_month_index & last_year_index : The current selected index of the 2nd and 3rd or month and year selection widget.
    // This is important because using the index we will be moving the cursor on arrow key presses by passing it to the home page ui.
    //
    // path & conn : The connection status and the path of the database
    //
    // all_data : This is a struct that fetches and stores the home page data based on the current month and year index.
    // It contains the selected month and year's all transaction, all ↑ and ↓ which is stored in the database,
    // monthly balance data and the database id numbers. The data is parsed in various functions to only select the
    // relevant content and pass to the UI. Operates in the Home page UI.
    //
    // table : I am calling the spreadsheet like widget the table. It contains the selected month and year's all transactions
    // cu_page : Opening UI page which is selected as the Home page
    // cu_tx_page : To make sure random key presses are not registered, selected as Nothing
    //
    // data_for_tx : This is the struct for storing data which is to be stored as the transaction in the database.
    // It also contains all the texts for the Status widget in the transaction adding ui. For each key presses when
    // selected adds a character to the relevant struct field.
    //
    // total_income & total_expense : Contains the data of all incomes and expenses of the selected month and year,
    // calculated from the transaction saved in the database, it is needed for the Income and Expense section in the Home page.
    // Why is it a vector? Because the entire row has to be saved inside this to put in the UI.
    //
    // starter_index : to keep track of the loop on each iteration on the initial page's animation.
    // version_checked : during the loop of the app, this variable is tracked so we don't keep opening the popup multiple times

    let mut version_checked = false;
    let mut selected_tab = SelectedTab::Months;
    let mut last_month_index = 99;
    let mut last_year_index = 99;
    let path = "data.sqlite";
    let conn = Connection::open(path).expect("Could not connect to database");
    conn.execute("PRAGMA foreign_keys = ON", [])
        .expect("Could not enable foreign keys");
    let mut all_data = TransactionData::new(&conn, 0, 0);
    let mut table = TableData::new(all_data.get_txs());
    let mut cu_page = CurrentUi::Initial;
    let mut cu_popup = PopupState::Nothing;
    let mut cu_tx_page = TxTab::Nothing;
    let mut data_for_tx = AddTxData::new();
    let mut starter_index = 0;
    // The next three variable will determine whether a certain pop is turned on or off.
    // Based on this, the popup is created/passed
    // This one wil likely remain unused until one is messing with the database

    // The next three vectors will store the lines that will be displayed on popup
    // The Vec contains popup title, text, x axis and y axis size.
    let mut popup_data_delete_failed = vec![];

    let mut popup_data_new_update = vec![];
    popup_data_new_update.push("New Update".to_string());
    popup_data_new_update.push(
        "There is a new version available\n
'Enter' : Redirect to the new version\n\nPress Any Key to dismiss"
            .to_string(),
    );
    popup_data_new_update.push("50".to_string());
    popup_data_new_update.push("30".to_string());

    let mut popup_data_help = vec![];
    popup_data_help.push("Help".to_string());
    popup_data_help.push(
        "'Arrow Key' : Navigate
'A' : Add Transaction Page
'F' : Home Page
'D' : Delete selected Transaction (Home Page)
'J' : Add new Transaction Methods (Home Page)
'E' : Edit Selected Transaction (Home Page)
'H' : Open Hotkey Help
'Q' : Quit

Add Transaction Page:
'1' : Edit Date          '4' : Edit Amount
'2' : Edit TX details    '3' : Edit TX Method
'5' : Edit TX Type
'S' : Save the data as a Transaction
'Enter' : Submit field and continue
'Esc' : Stop editing filed\n
Press Any Key to dismiss"
            .to_string(),
    );
    popup_data_help.push("50".to_string());
    popup_data_help.push("65".to_string());

    // The loop begins at this point and before the loop starts, multiple variables are initiated
    // with the default values which will quickly be changing once the loop starts.
    loop {
        // after each refresh this will check the current selected month, year and if a table/spreadsheet row is selected in the ui.
        let cu_month_index = months.index;
        let cu_year_index = years.index;
        let cu_table_index = table.state.selected();

        // reload the data saved in memory each time the month or the year changes
        if cu_month_index != last_month_index || cu_year_index != last_year_index {
            all_data = TransactionData::new(&conn, cu_month_index, cu_year_index);
            table = TableData::new(all_data.get_txs());
            last_month_index = cu_month_index;
            last_year_index = cu_year_index;
        };

        let total_income = all_data.get_total_income(&conn, cu_table_index);
        let total_expense = all_data.get_total_expense(&conn, cu_table_index);

        // balance variable contains all the 'rows' of the first/Balance widget in the home page.
        // So each line is inside a vector. "" represents empty placeholder.
        let mut balance: Vec<Vec<String>> = vec![vec!["".to_string()]];
        balance[0].extend(get_all_tx_methods(&conn));
        balance[0].extend(vec!["Total".to_string()]);

        // save the % of space each column should take in the Balance section based on the total
        // transaction methods/columns available
        let width_percent = 100 / balance[0].len() as u16;
        let mut width_data = vec![];
        for _i in 0..balance[0].len() {
            width_data.push(Constraint::Percentage(width_percent));
        }

        // cu_table_index is the spreadsheet/Transaction widget index. If a row is selected,
        // get the balance there was once that transaction happened + the changes it did
        // otherwise, get the absolute final balance after all transaction happened + no changes.

        match cu_table_index {
            // pass out the current index to get the necessary balance & changes data
            Some(a) => {
                balance.push(all_data.get_balance(a));
                balance.push(all_data.get_changes(a));
            }
            // if none selected, get empty changes + the absolute final balance
            None => {
                balance.push(all_data.get_last_balance(&conn));
                balance.push(get_empty_changes(&conn));
            }
        }

        // total_income & total_expense data changes on each month/year index change. So push it now
        // to the balance vector to align with the rows.
        balance.push(total_income.clone());
        balance.push(total_expense.clone());

        // check the version of current TUI and based on that, turn on the popup
        if version_checked == false {
            if new_version_available == true {
                cu_popup = PopupState::NewUpdate
            }
            version_checked = true;
        }

        // passing out relevant data to the ui function
        match cu_page {
            CurrentUi::Home => terminal.draw(|f| {
                ui(
                    f,
                    &months,
                    &years,
                    &mut table,
                    &mut balance,
                    &selected_tab,
                    &mut width_data,
                );

                match cu_popup {
                    PopupState::Helper => create_popup(f, &popup_data_help),
                    PopupState::DeleteFailed => create_popup(f, &popup_data_delete_failed),
                    _ => {}
                }
            })?,
            CurrentUi::AddTx => terminal.draw(|f| {
                tx_ui(
                    f,
                    data_for_tx.get_all_texts(),
                    &cu_tx_page,
                    &data_for_tx.tx_status,
                );

                match cu_popup {
                    PopupState::Helper => create_popup(f, &popup_data_help),
                    _ => {}
                }
            })?,
            CurrentUi::Initial => terminal.draw(|f| {
                starter_ui(f, starter_index);
                starter_index += 1;
                if starter_index > 28 {
                    starter_index = 0;
                }

                match cu_popup {
                    PopupState::NewUpdate => create_popup(f, &popup_data_new_update),
                    _ => {}
                }
            })?,

            // TODO change data here for transfer page
            CurrentUi::Transfer => terminal.draw(|f| {
                transfer_ui_func(
                    f,
                    data_for_tx.get_all_texts(),
                    &cu_tx_page,
                    &data_for_tx.tx_status,
                );

                match cu_popup {
                    PopupState::Helper => create_popup(f, &popup_data_help),
                    _ => {}
                }
            })?,
        };

        // This is where the keyboard press tracking starts
        // There are two options, event or timer. Timer keeps the loop unblocked. Loops for
        // event checking each 40 milliseconds
        if poll(Duration::from_millis(40))? {
            if let Event::Key(key) = event::read()? {
                match cu_page {
                    CurrentUi::Home => {
                        match cu_popup {
                            PopupState::Nothing => {
                                match key.code {
                                    KeyCode::Char('q') => return Ok("".to_string()),
                                    KeyCode::Char('a') => cu_page = CurrentUi::AddTx,
                                    KeyCode::Char('j') => return Ok("Change".to_string()),
                                    KeyCode::Char('h') => cu_popup = PopupState::Helper,
                                    KeyCode::Char('e') => {
                                        if let Some(a) = cu_table_index {
                                            let target_data = &all_data.get_txs()[a];
                                            let target_id_num = all_data.get_id_num(a);
                                            data_for_tx = AddTxData::custom(
                                                &target_data[0],
                                                &target_data[1],
                                                &target_data[2],
                                                &target_data[3],
                                                &target_data[4],
                                                target_id_num,
                                            );
                                            cu_page = CurrentUi::AddTx;
                                        }
                                    }
                                    KeyCode::Char('d') => {
                                        if table.state.selected() != None {
                                            let status =
                                                all_data.del_tx(table.state.selected().unwrap());
                                            match status {
                                                Ok(_) => {
                                                    // transaction deleted so reload the data again
                                                    all_data = TransactionData::new(
                                                        &conn,
                                                        cu_month_index,
                                                        cu_year_index,
                                                    );
                                                    table = TableData::new(all_data.get_txs());
                                                    table.state.select(None);
                                                    selected_tab = SelectedTab::Months;
                                                }
                                                Err(e) => {
                                                    popup_data_delete_failed = vec![];
                                                    popup_data_delete_failed
                                                        .push("Delete Error".to_string());
                                                    popup_data_delete_failed.push(format!(
                                                        "Error while deleting transaction \
                                                    \nError: {e:?}\n\nPress Any Key to dismiss"
                                                    ));
                                                    popup_data_delete_failed.push("50".to_string());
                                                    popup_data_delete_failed.push("30".to_string());
                                                    cu_popup = PopupState::DeleteFailed;
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
                                                if all_data.all_tx.len() < 1 {
                                                    selected_tab = selected_tab.change_tab_up();
                                                }
                                                // executes when going from first table row to month widget
                                                else if table.state.selected() == Some(0) {
                                                    selected_tab = SelectedTab::Months;
                                                    table.state.select(None);
                                                } else {
                                                    if all_data.all_tx.len() > 0 {
                                                        table.previous();
                                                    }
                                                }
                                            }
                                            SelectedTab::Years => {
                                                // Do not select any table rows in the table section If
                                                // there is no transaction
                                                if all_data.all_tx.len() < 1 {
                                                    selected_tab = selected_tab.change_tab_up();
                                                } else {
                                                    // Move to the selected value on table/Transaction widget
                                                    // to the last row if pressed up on Year section
                                                    table.state.select(Some(table.items.len() - 1));
                                                    selected_tab = selected_tab.change_tab_up();
                                                    if all_data.all_tx.len() < 1 {
                                                        selected_tab = selected_tab.change_tab_up();
                                                    }
                                                }
                                            }
                                            _ => selected_tab = selected_tab.change_tab_up(),
                                        }
                                    }
                                    KeyCode::Down => {
                                        match &selected_tab {
                                            SelectedTab::Table => {
                                                // Do not proceed to the table section If
                                                // there is no transaction
                                                if all_data.all_tx.len() < 1 {
                                                    selected_tab = selected_tab.change_tab_down();
                                                }
                                                // executes when pressed on last row of the table
                                                // moves to the year widget
                                                else if table.state.selected()
                                                    == Some(table.items.len() - 1)
                                                {
                                                    selected_tab = SelectedTab::Years;
                                                    table.state.select(None);
                                                } else {
                                                    if all_data.all_tx.len() > 0 {
                                                        table.next();
                                                    }
                                                }
                                            }
                                            _ => selected_tab = selected_tab.change_tab_down(),
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            _ => match key.code {
                                _ => cu_popup = PopupState::Nothing,
                            },
                        }
                    }
                    CurrentUi::AddTx => {
                        let status = add_tx_checker(
                            key,
                            &mut cu_page,
                            &mut cu_popup,
                            &mut cu_tx_page,
                            &mut data_for_tx,
                            &mut all_data,
                            &mut table,
                            &mut selected_tab,
                            cu_month_index,
                            cu_year_index,
                            &conn,
                        )?;
                        if status != "0" {
                            return Ok(status);
                        }
                    }
                    CurrentUi::Initial => {
                        let status = initial_checker(key, &mut cu_page, &mut cu_popup)?;
                        if status != "0" {
                            return Ok(status);
                        }
                    }
                    // TODO change data here for transfer
                    CurrentUi::Transfer => {}
                }
            };
        }
    }
}
