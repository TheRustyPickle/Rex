mod home_page;
mod tx_page;
mod db;
mod initial_page;

use db::create_db;
use tx_page::AddTxData;
use tx_page::tx_ui;
use home_page::{TableData, TimeData, SelectedTab, CurrentUi, TxTab};
use db::{get_all_tx_methods, get_empty_changes};
use home_page::TransactionData;
use home_page::ui;
use initial_page::starter_ui;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use crossterm::{event::{poll}};
use std::time::Duration;
use rusqlite::{Connection, Result};
use std::fs;
use std::{error::Error, io};
use tui::layout::Constraint;
use tui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};


// [x] Check current path for the db, create new db if necessary
// [x] create add transaction ui + editing box with inputs
// [x] func for saving & deleting txs
// [x] add creating tx button
// [x] add removing tx button
// [ ] create a popup ui on Home window for commands list or if a new version is available
// [x] simple ui at the start of the program highlighting button
// [ ] allow adding tx methods
// [ ] change color scheme?
// [x] change balances to f32?
// [x] add date column to all_balance & all_changes
// [x] verify db cascade method working or not
// [ ] add more panic handling
// [ ] add save points for db commits
// [x] latest balance empty = all 0
// [x] limit add tx date between the available years
// [x] add status on add tx page
// [x] add monthly expense & income on home page
// [x] add more comments
// [x] check for empty fields if S is pressed
// [x] do not return to home if add tx is failed and show error on status section
// [x] check amount that it is not negative
// [ ] write tests
// [x] initial ui
// [ ] change database location
// [ ] Need to update hotkey for the popup ui

/// The main function is designed for 3 things. 
/// - checks if the local database named data.sqlite is found or create the database
/// - Calls lib function that makes the tui magic work such as moving to an alternate screen
/// - Passes a few terminal state and if there is an error, quits the application 

fn main() -> Result<(), Box<dyn Error>> {
    // checks the local folder and searches for data.sqlite
    let paths = fs::read_dir(".").unwrap();
    let mut db_found = false;
    for path in paths {
        let path = path.unwrap().path().display().to_string();
        if path == "./data.sqlite" {
            db_found = true;
        }
    }
    // TODO if db not found take user input for tx method during the initialization
    // create a new db if not found. If there is an error, delete the failed data.sqlite file
    if db_found != true {
        println!("Creating New Database. It may take some time...");
        let status = create_db();
        match status {
            Ok(_) => {}
            Err(_) => {
                println!("Database creation failed. Try again.");
                fs::remove_file("data.sqlite").expect("Error while deleting database");
            }
        }
    }
    // TUI magic functions starts here with multiple calls
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let months = TimeData::new(vec![
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ]);
    let years = TimeData::new(vec!["2022", "2023", "2024", "2025"]);

    // pass a few data to the main function and loop forever or until quit/faced with an error
    let res = run_app(&mut terminal, months, years);
    
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }
    Ok(())
}

/// run_app is the core part that makes the entire program run. It basically loops
/// incredibly fast to refresh the terminal and passes the provided data to ui modules to draw them.
/// While the loop is running, the program executes, gets the data from the db and key presses to
/// To keep on providing new data to the UI. 
/// What it does:
/// - Sends relevant data to the UI creating modules
/// - Stores the current UI data and states
/// - Detects all key presses and directs the UI accordingly
/// - Modifies UI data as needed
 
fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut months: TimeData,
    mut years: TimeData,
) -> io::Result<()> {

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
    let mut cu_tx_page = TxTab::Nothing;
    let mut data_for_tx = AddTxData::new();
    let mut total_income = vec![];
    let mut total_expense = vec![];
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
            total_income = all_data.get_total_income(&conn);
            total_expense = all_data.get_total_expense(&conn);
            last_month_index = cu_month_index;
            last_year_index = cu_year_index;
        };

        // balance variable contains all the 'rows' of the first/Balance widget in the home page.
        // So each line is inside a vector. "" represents empty placeholder.
        let mut balance: Vec<Vec<String>> = vec![vec!["".to_string()]];
        balance[0].extend(get_all_tx_methods(&conn));
        balance[0].extend(vec!["Total".to_string()]);

        // save the % of space each column should take in the Balance section based on the total 
        // transaction methods/columns available
        // Need to do a + 1 because there is a Total column & to make the gap tighter
        let width_percent = 100 / balance[0].len() as u16 + 1;
        let mut width_data = vec![];
        for _i in 0..balance[0].len()+1 {
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
            None => {
                balance.push(all_data.get_last_balance(&conn));
                balance.push(get_empty_changes());
            }
        }

        // total_income & total_expense data changes on each month/year index change. So push it now
        // to the balance vector to align with the rows.
        balance.push(total_income.clone());
        balance.push(total_expense.clone());

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
                )
            })?,
            CurrentUi::AddTx => terminal.draw(|f| {
                tx_ui(
                    f,
                    data_for_tx.get_all_texts(),
                    &cu_tx_page,
                    &data_for_tx.tx_status,
                )
            })?,
            CurrentUi::Initial => terminal.draw(|f| {
                starter_ui(f, starter_index);
                starter_index += 1;
                if starter_index > 28 { starter_index = 0;}
            })?
        };

        // This is where the keyboard press tracking starts
        if poll(Duration::from_millis(40))? {
            if let Event::Key(key) = event::read()? {
                match cu_page {
                    CurrentUi::Home => match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('a') => cu_page = CurrentUi::AddTx,
                        KeyCode::Char('d') => {
                            if table.state.selected() != None {
                                let status = all_data.del_tx(&conn, table.state.selected().unwrap());
                                match status {
                                    Ok(_) => {
                                        // transaction deleted so reload the data again
                                        all_data =
                                            TransactionData::new(&conn, cu_month_index, cu_year_index);
                                        table = TableData::new(all_data.get_txs());
                                        table.state.select(None);
                                        selected_tab = SelectedTab::Months;
                                    }

                                    Err(_) => {}
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
                                    else if table.state.selected() == Some(table.items.len() - 1) {
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
                    },
                    CurrentUi::AddTx => match cu_tx_page {
                        // start matching key pressed based on which widget is selected.
                        // current state tracked with enums
                        TxTab::Nothing => match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Char('h') => {
                                cu_page = CurrentUi::Home;
                                cu_tx_page = TxTab::Nothing;
                                data_for_tx = AddTxData::new();
                            }
                            KeyCode::Char('s') => {
                                let status = data_for_tx.add_tx(&conn);
                                if status == "".to_string() {
                                    cu_page = CurrentUi::Home;
                                    data_for_tx = AddTxData::new();
                                    
                                } else {
                                    data_for_tx.add_tx_status(&status);
                                }
                                
                            }
                            KeyCode::Char('1') => cu_tx_page = TxTab::Date,
                            KeyCode::Char('2') => cu_tx_page = TxTab::Details,
                            KeyCode::Char('3') => cu_tx_page = TxTab::TxMethod,
                            KeyCode::Char('4') => cu_tx_page = TxTab::Amount,
                            KeyCode::Char('5') => cu_tx_page = TxTab::TxType,
                            KeyCode::Enter => cu_tx_page = TxTab::Nothing,
                            KeyCode::Esc => cu_tx_page = TxTab::Nothing,
                            _ => {}
                        },

                        TxTab::Date => match key.code {
                            KeyCode::Enter => {
                                let status = data_for_tx.check_date();
                                match status {
                                    Ok(a) => {
                                        data_for_tx.add_tx_status(&a);
                                        if a.contains("Accepted") || a.contains("Nothing") {
                                            cu_tx_page = TxTab::Nothing
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
                                        if a.contains("Accepted") {
                                            cu_tx_page = TxTab::Nothing
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

                        TxTab::Details => match key.code {
                            KeyCode::Enter => cu_tx_page = TxTab::Nothing,
                            KeyCode::Esc => cu_tx_page = TxTab::Nothing,
                            KeyCode::Backspace => data_for_tx.edit_details('a', true),
                            KeyCode::Char(a) => data_for_tx.edit_details(a, false),
                            _ => {}
                        },

                        TxTab::TxMethod => match key.code {
                            KeyCode::Enter => {
                                let status = data_for_tx.check_tx_method(&conn);
                                data_for_tx.add_tx_status(&status);
                                if status.contains("Accepted") || status.contains("Nothing") {
                                    cu_tx_page = TxTab::Nothing
                                }
                            }
                            KeyCode::Esc => {
                                let status = data_for_tx.check_tx_method(&conn);
                                data_for_tx.add_tx_status(&status);
                                if status.contains("Accepted") {
                                    cu_tx_page = TxTab::Nothing
                                }
                            }
                            KeyCode::Backspace => data_for_tx.edit_tx_method('a', true),
                            KeyCode::Char(a) => data_for_tx.edit_tx_method(a, false),
                            _ => {}
                        },

                        TxTab::Amount => match key.code {
                            KeyCode::Enter => {
                                let status = data_for_tx.check_amount();
                                match status {
                                    Ok(a) => {
                                        data_for_tx.add_tx_status(&a);
                                        if a.contains("zero"){
                                        }
                                        else {
                                            cu_tx_page = TxTab::Nothing;
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
                                        if a.contains("zero"){
                                        }
                                        else {
                                            cu_tx_page = TxTab::Nothing;
                                        }
                                    }
                                    Err(_) => data_for_tx.add_tx_status("Amount: Invalid Amount found"),
                                }
                            }
                            KeyCode::Backspace => data_for_tx.edit_amount('a', true),
                            KeyCode::Char(a) => data_for_tx.edit_amount(a, false),
                            _ => {}
                        },

                        TxTab::TxType => match key.code {
                            KeyCode::Enter => {
                                let status = data_for_tx.check_tx_type();
                                data_for_tx.add_tx_status(&status);
                                if status.contains("Accepted") || status.contains("Nothing") {
                                    cu_tx_page = TxTab::Nothing
                                }
                            }
                            KeyCode::Esc => cu_tx_page = TxTab::Nothing,
                            KeyCode::Backspace => data_for_tx.edit_tx_type('a', true),
                            KeyCode::Char(a) => data_for_tx.edit_tx_type(a, false),
                            _ => {}
                        },
                    },
                    CurrentUi::Initial => match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        _ => cu_page = CurrentUi::Home,
                    }
                }
            };
        }
        else {}
    }
}
