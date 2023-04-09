use crate::add_tx_page::add_tx_ui;
use crate::chart_page::{chart_ui, ChartData};
use crate::home_page::home_ui;
use crate::home_page::TransactionData;
use crate::initial_page::initial_ui;
use crate::key_checker::{
    add_tx_keys, chart_keys, home_keys, initial_keys, summary_keys, transfer_keys, InputKeyHandler,
};
use crate::outputs::{HandlingOutput, UiHandlingError};
use crate::page_handler::{
    AddTxTab, ChartTab, CurrentUi, HomeTab, IndexedData, PopupState, SummaryTab, TableData,
    TransferTab,
};
use crate::popup_page::add_popup;
use crate::summary_page::{summary_ui, SummaryData};
use crate::transfer_page::transfer_ui;
use crate::tx_handler::TxData;
use crate::utility::{get_all_tx_methods, get_empty_changes};
use crossterm::event::poll;
use crossterm::event::{self, Event};
use rusqlite::Connection;
use std::time::Duration;
use tui::backend::Backend;
use tui::layout::Constraint;
use tui::Terminal;

/// The core part that makes the entire program run. It loops
/// incredibly fast to refresh the terminal and passes the provided data to ui modules to draw them.
pub fn start_app<B: Backend>(
    terminal: &mut Terminal<B>,
    new_version_available: bool,
) -> Result<HandlingOutput, UiHandlingError> {
    // Setting up some default values. Let's go through all of them

    // contains the home page month list that is indexed
    let mut add_tx_months = IndexedData::new_monthly();
    // contains the home page year list that is indexed
    let mut add_tx_years = IndexedData::new_yearly();
    // contains the chart page month list that is indexed
    let mut chart_months = IndexedData::new_monthly();
    // contains the chart page year list that is indexed
    let mut chart_years = IndexedData::new_yearly();
    // contains the chart page mode selection list that is indexed
    let mut chart_modes = IndexedData::new(vec![
        "Monthly".to_string(),
        "Yearly".to_string(),
        "All Time".to_string(),
    ]);
    // contains the summary page month list that is indexed
    let mut summary_months = IndexedData::new_monthly();
    // contains the summary page year list that is indexed
    let mut summary_years = IndexedData::new_yearly();
    // contains the summary page mode selection list that is indexed
    let mut summary_modes = IndexedData::new(vec![
        "Monthly".to_string(),
        "Yearly".to_string(),
        "All Time".to_string(),
    ]);

    // We only want to open the version checker popup once, turns true once the checking is done
    let mut version_checked = false;

    // the selected widget on the Home Page. Default set to the month selection
    let mut home_tab = HomeTab::Months;

    // Helps keep track if the Home Page's month or the year index was changed
    let mut last_month_index = 99;
    let mut last_year_index = 99;

    // the database connection and the path of the db
    let path = "data.sqlite";
    let conn = Connection::open(path).expect("Could not connect to database");
    conn.execute("PRAGMA foreign_keys = ON", [])
        .expect("Could not enable foreign keys");

    // Stores all data relevant for home page such as balance, changes and txs
    let mut all_tx_data = TransactionData::new(&conn, 0, 0);
    // data for the Home Page's tx table
    let mut table = TableData::new(all_tx_data.get_txs());

    // The page which is currently selected. Default is the initial page
    let mut page = CurrentUi::Initial;
    // stores current popup status
    let mut popup = PopupState::Nothing;
    // Stores the current selected widget on Add Transaction page
    let mut tx_tab = AddTxTab::Nothing;
    // Store the current selected widget on Add Transfer page
    let mut transfer_tab = TransferTab::Nothing;
    // Store the current selected widget on Chart page
    let mut chart_tab = ChartTab::ModeSelection;
    // Store the current selected widget on Summary page
    let mut summary_tab = SummaryTab::ModeSelection;

    // Holds the data that will be/are inserted into the Add Tx page's input fields
    let mut add_tx_data = TxData::new();
    // Holds the data that will be/are inserted into the Transfer Tx page's input fields
    let mut transfer_data = TxData::new_transfer();
    // Holds the data that will be/are inserted into the Summary Page
    let mut summary_data = SummaryData::new(&conn);

    // Stores how many unique tags that stores. Used as an index for Summary Page table
    let mut total_tags = summary_data.get_table_data().len();
    // data for the Summary Page's table
    let mut summary_table = TableData::new(summary_data.get_table_data());
    // Texts and numbers of the details shown at top of summary page
    let mut summary_texts = summary_data.get_tx_data();
    // summary data generation is a heavy task. As it's a loop, the data is loaded once the page is selected.
    // If true, the data is no longer generated even if the summary page is on
    let mut summary_reloaded = false;

    // the initial page REX loading index
    let mut starter_index = 0;

    // The loop begins at this point and before the loop starts, multiple variables are initiated
    // with the default values which will quickly be changing once the loop starts.
    loop {
        // after each refresh this will check the current selected month, year and if a table/spreadsheet row is selected in the ui.
        let cu_month_index = add_tx_months.index;
        let cu_year_index = add_tx_years.index;
        let cu_table_index = table.state.selected();

        // reload the data saved in memory each time the month or the year changes
        if cu_month_index != last_month_index || cu_year_index != last_year_index {
            all_tx_data = TransactionData::new(&conn, cu_month_index, cu_year_index);
            table = TableData::new(all_tx_data.get_txs());
            last_month_index = cu_month_index;
            last_year_index = cu_year_index;
        };

        // total income and expense value for the home page
        let total_income = all_tx_data.get_total_income(&conn, cu_table_index);
        let total_expense = all_tx_data.get_total_expense(&conn, cu_table_index);

        // balance variable contains all the 'rows' of the Balance widget in the home page.
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

        // cu_table_index is the Home Page table widget index. If a row is selected,
        // get the balance there was once that transaction happened + the changes it did
        // otherwise, get the absolute final balance after all transaction happened + no changes.

        match cu_table_index {
            // pass out the current index to get the necessary balance & changes data
            Some(a) => {
                balance.push(all_tx_data.get_balance(a));
                balance.push(all_tx_data.get_changes(a));
            }
            // if none selected, get empty changes + the absolute final balance
            None => {
                balance.push(all_tx_data.get_last_balance(&conn));
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
                popup = PopupState::NewUpdate
            }
            version_checked = true;
        }

        // passing out relevant data to the ui function

        terminal
            .draw(|f| {
                match page {
                    CurrentUi::Home => {
                        home_ui(
                            f,
                            &add_tx_months,
                            &add_tx_years,
                            &mut table,
                            &mut balance,
                            &home_tab,
                            &mut width_data,
                        );
                        summary_reloaded = false;
                    }
                    CurrentUi::AddTx => {
                        add_tx_ui(
                            f,
                            add_tx_data.get_all_texts(),
                            &tx_tab,
                            &add_tx_data.tx_status,
                        );
                        summary_reloaded = false;
                    }
                    CurrentUi::Initial => {
                        initial_ui(f, starter_index);
                        starter_index += 1;
                        if starter_index > 27 {
                            starter_index = 0;
                        }
                        summary_reloaded = false;
                    }
                    CurrentUi::Transfer => {
                        transfer_ui(
                            f,
                            transfer_data.get_all_texts(),
                            &transfer_tab,
                            &transfer_data.tx_status,
                        );
                        summary_reloaded = false;
                    }
                    CurrentUi::Chart => {
                        let chart_data = ChartData::set(cu_year_index);
                        chart_ui(
                            f,
                            &chart_months,
                            &chart_years,
                            &chart_modes,
                            chart_data,
                            &chart_tab,
                        );
                        summary_reloaded = false;
                    }
                    CurrentUi::Summary => {
                        if !summary_reloaded {
                            summary_data = SummaryData::new(&conn);
                            total_tags = summary_data.get_table_data().len();
                            summary_table = TableData::new(summary_data.get_table_data());
                            summary_texts = summary_data.get_tx_data();
                            if total_tags > 0 {
                                summary_table.state.select(Some(0));
                            }
                        }
                        summary_reloaded = true;
                        summary_ui(
                            f,
                            &summary_months,
                            &summary_years,
                            &summary_modes,
                            &mut summary_table,
                            &summary_texts,
                            &summary_tab,
                        );
                    }
                }
                add_popup(f, &popup);
            })
            .map_err(UiHandlingError::DrawingError)?;

        if let CurrentUi::Initial = page {
            if !poll(Duration::from_millis(40)).map_err(UiHandlingError::PollingError)? {
                continue;
            }
        }

        if let Event::Key(key) = event::read().map_err(UiHandlingError::PollingError)? {
            let mut handler = InputKeyHandler::new(
                key,
                &mut page,
                &mut popup,
                &mut tx_tab,
                &mut transfer_tab,
                &mut chart_tab,
                &mut summary_tab,
                &mut home_tab,
                &mut add_tx_data,
                &mut transfer_data,
                &mut all_tx_data,
                &mut table,
                &mut summary_table,
                &mut add_tx_months,
                &mut add_tx_years,
                &mut chart_months,
                &mut chart_years,
                &mut chart_modes,
                &mut summary_months,
                &mut summary_years,
                &mut summary_modes,
                cu_month_index,
                cu_year_index,
                cu_table_index,
                total_tags,
                &conn,
            );

            let status = match handler.page {
                CurrentUi::Initial => initial_keys(&mut handler),
                CurrentUi::Home => home_keys(&mut handler),
                CurrentUi::AddTx => add_tx_keys(&mut handler),
                CurrentUi::Transfer => transfer_keys(&mut handler),
                CurrentUi::Chart => chart_keys(&mut handler),
                CurrentUi::Summary => summary_keys(&mut handler),
            };

            if let Some(output) = status {
                return Ok(output);
            }
        }
    }
}
