use crate::chart_page::{chart_ui, ChartData};
use crate::db::{get_all_tx_methods, get_empty_changes};
use crate::home_page::ui;
use crate::home_page::TransactionData;
use crate::home_page::{
    CurrentUi, PopupState, SelectedTab, TableData, TimeData, TransferTab, TxTab,
};
use crate::initial_page::starter_ui;
use crate::key_checker::{
    add_tx_keys, chart_keys, home_keys, initial_keys, summary_keys, transfer_keys,
};
use crate::popup_page::add_popup;
use crate::summary_page::{summary_ui, SummaryData};
use crate::transfer_page::{transfer_ui, TransferData};
use crate::tx_page::tx_ui;
use crate::tx_page::AddTxData;
use crossterm::event::poll;
use crossterm::event::{self, Event};
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
    // cu_tx_page : To make sure random key presses are not registered for the Transaction UI, selected as Nothing
    // cu_transfer_page : To make sure random key presses are not registered for the Transfer Transaction UI, selected as Nothing
    // cu_popup : Contains the current state of popups. Defaults as Nothing
    //
    // data_for_tx : This is the struct for storing data which is to be stored as the transaction in the database.
    // It also contains all the texts for the Status widget in the transaction adding ui. For each key presses when
    // selected adds a character to the relevant struct field.
    //
    // data_for_transfer: Contains data to create a Transfer Transaction, handles key presses and saving the transaction.
    //
    // summary_data: Creates a struct that contains all information to create the Summary UI
    // total_tags: Highlights how many tags the DB contains
    // summary_table: Contains the vector to create Summary UI Table section
    // summary_texts: Contains relevant texts inside a vector to create the upper section of the Summary UI
    //
    // summary_reloaded: The interface is a loop so we don't want to keep reloading same data over and over again
    // which can be expensive. Makes sure it iters only one time but keeps the loop running.
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
    let mut cu_transfer_page = TransferTab::Nothing;
    let mut data_for_tx = AddTxData::new();
    let mut data_for_transfer = TransferData::new();
    let mut summary_data = SummaryData::new(&conn);
    let mut total_tags = summary_data.get_table_data().len();
    let mut summary_table = TableData::new(summary_data.get_table_data());
    let mut summary_texts = summary_data.get_tx_data();
    let mut summary_reloaded = false;
    let mut starter_index = 0;

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
        if !version_checked {
            if new_version_available {
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
                summary_reloaded = false;
                match cu_popup {
                    PopupState::Helper => add_popup(f, 1),
                    PopupState::DeleteFailed => add_popup(f, 2),
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
                summary_reloaded = false;
                if let PopupState::Helper = cu_popup {
                    add_popup(f, 1)
                }
            })?,
            CurrentUi::Initial => terminal.draw(|f| {
                starter_ui(f, starter_index);
                starter_index += 1;
                if starter_index > 28 {
                    starter_index = 0;
                }
                summary_reloaded = false;
                if let PopupState::NewUpdate = cu_popup {
                    add_popup(f, 0)
                }
            })?,

            CurrentUi::Transfer => terminal.draw(|f| {
                transfer_ui(
                    f,
                    data_for_transfer.get_all_texts(),
                    &cu_transfer_page,
                    &data_for_transfer.tx_status,
                );
                summary_reloaded = false;
                if let PopupState::Helper = cu_popup {
                    add_popup(f, 1)
                }
            })?,
            CurrentUi::Chart => {
                let data_for_chart = ChartData::set(cu_year_index);
                summary_reloaded = false;
                terminal.draw(|f| {
                    chart_ui(f, data_for_chart);

                    if let PopupState::Helper = cu_popup {
                        add_popup(f, 1)
                    }
                })?
            }
            CurrentUi::Summary => {
                if summary_reloaded == false {
                    summary_data = SummaryData::new(&conn);
                    total_tags = summary_data.get_table_data().len();
                    summary_table = TableData::new(summary_data.get_table_data());
                    summary_texts = summary_data.get_tx_data();
                    if total_tags > 0 {
                        summary_table.state.select(Some(0));
                    }

                    summary_reloaded = true;
                }
                terminal.draw(|f| {
                    summary_ui(f, &mut summary_table, &summary_texts);

                    if let PopupState::Helper = cu_popup {
                        add_popup(f, 1)
                    }
                })?
            }
        };

        // This is where the keyboard press tracking starts
        // There are two options, event or timer. Timer keeps the loop unblocked. Loops for
        // event checking each 40 milliseconds
        if poll(Duration::from_millis(40))? {
            if let Event::Key(key) = event::read()? {
                match cu_page {
                    CurrentUi::Home => {
                        let status = home_keys(
                            key,
                            &mut cu_page,
                            &mut cu_popup,
                            &mut data_for_tx,
                            &mut data_for_transfer,
                            &mut all_data,
                            &mut table,
                            &mut selected_tab,
                            cu_table_index,
                            cu_month_index,
                            cu_year_index,
                            &mut months,
                            &mut years,
                            &conn,
                        )?;
                        if status != "0" {
                            return Ok(status);
                        }
                    }
                    CurrentUi::AddTx => {
                        let status = add_tx_keys(
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
                        let status = initial_keys(key, &mut cu_page, &mut cu_popup)?;
                        if status != "0" {
                            return Ok(status);
                        }
                    }
                    CurrentUi::Transfer => {
                        let status = transfer_keys(
                            key,
                            &mut cu_page,
                            &mut cu_popup,
                            &mut cu_transfer_page,
                            &mut data_for_transfer,
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
                    CurrentUi::Chart => {
                        let status = chart_keys(
                            key,
                            &mut cu_page,
                            &mut cu_popup,
                            &mut cu_tx_page,
                            &mut data_for_tx,
                        )?;
                        if status != "0" {
                            return Ok(status);
                        }
                    }
                    CurrentUi::Summary => {
                        let status = summary_keys(
                            key,
                            &mut cu_page,
                            &mut cu_popup,
                            &mut cu_tx_page,
                            &mut data_for_tx,
                            &mut summary_table,
                            total_tags,
                        )?;
                        if status != "0" {
                            return Ok(status);
                        }
                    }
                }
            };
        }
    }
}
