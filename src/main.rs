mod db;
mod home_page;
mod initial_page;
mod tx_page;
mod popup_page;

use atty::Stream;
use std::process::Command;
use open;
use crossterm::event::poll;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use db::{create_db, add_new_tx_methods};
use db::{get_all_tx_methods, get_empty_changes, get_user_tx_methods};
use home_page::ui;
use home_page::TransactionData;
use home_page::{CurrentUi, SelectedTab, TableData, TimeData, TxTab};
use initial_page::starter_ui;
use rusqlite::{Connection, Result};
use popup_page::create_popup;
use std::fs;
use std::{time::Duration, error::Error, thread, io, process};
use tui::layout::Constraint;
use tui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use tx_page::tx_ui;
use tx_page::AddTxData;
use std::fs::File;
use std::io::prelude::*;

// [x] Check current path for the db, create new db if necessary
// [x] create add transaction ui + editing box with inputs
// [x] func for saving & deleting txs
// [x] add creating tx button
// [x] add removing tx button
// [x] create a popup ui on Home window for commands list or if a new version is available
// [x] simple ui at the start of the program highlighting button
// [x] allow adding tx methods
// [x] change color scheme?
// [x] change balances to f32?
// [x] add date column to all_balance & all_changes
// [x] verify db cascade method working or not
// [x] add more panic handling
// [x] add save points for db commits
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
// [x] change database location (nothing to do for now)
// [x] Need to update hotkey for the popup ui
// [x] run on terminal when using the binary
// [x] allow cancelling adding transaction method 

/// The starting function checks for the local database location and creates a new database
/// if not existing. Also checks if the user is trying to open the app via a terminal or the binary.
/// If trying to open using the binary, tries open the relevant terminal to execute the app.
/// Lastly, starts a loop that keeps the interface running until exit command is given.
fn main() -> Result<(), Box<dyn Error>> {

    let mut is_windows = false;
    let mut verifying_path = "./data.sqlite";
    // change details if running on windows
    if cfg!(target_os = "windows") {
        is_windows = true;
        verifying_path = r#".\data.sqlite"#;
    }

    // atty verifies whether a terminal is being used or not.
    if atty::is(Stream::Stdout) {} else {
        let cu_directory = std::env::current_dir()?.display().to_string();
        let output = if is_windows {
            // NOTE f*** windows. Unknown errors everywhere. 
            Command::new("cmd.exe")
                    .arg("start")
                    .arg("rex")
                    .output()?
        } else {
            let linux_dir = format!("--working-directory={}", cu_directory);
            Command::new("gnome-terminal")
                    .arg(linux_dir)
                    .arg("--")
                    .arg("./rex")
                    .output()?
        };
        // TODO add checking for common and most used terminal among different O
        // Windows cmd, Konsole, other to be found out.
        if output.stderr != vec![] {
            let full_text = format!("Error while trying to run console/terminal. Output: \n\n{:?}", output);
            let mut open = File::create("info.txt")?;
            open.write_all(full_text.as_bytes())?;
        }
        return Ok(());
        
    }
    // checks the local folder and searches for data.sqlite
    let paths = fs::read_dir(".")?;
    let mut db_found = false;
    for path in paths {
        let path = path?.path().display().to_string();
        if path == verifying_path {
            db_found = true;
        }
    }
    // create a new db if not found. If there is an error, delete the failed data.sqlite file
    if db_found != true {
        let db_tx_methods = get_user_tx_methods(false);
        println!("Creating New Database. It may take some time...");
        let status = create_db(db_tx_methods);
        match status {
            Ok(_) => {}
            Err(e) => {
                println!("Database creation failed. Try again. Error: {}", e);
                fs::remove_file("data.sqlite").expect("Error while deleting database");
                process::exit(1);
            }
        }
        
    }
    loop { 
        // Continue to loop to the main interface until the ending command or "break" is given
        let status = check_app(start_run_app());
        if &status == "break" {
            break;
        }
    }
    Ok(())
}

/// The function to start run_app along with executing commands for switching to an alternate screen,
/// mouse capturing and passing months and year data to the function and starts the interface
fn start_run_app() -> Result<String, Box<dyn Error>> {
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
    let res = run_app(&mut terminal, months, years)?;

    Ok(res)
}

/// The function is used to exit out of the interface 
fn exit_tui_interface() -> Result<(), Box<dyn Error>> {
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    disable_raw_mode()?;
    Ok(())
}

/// The function is used to check the output which caused the tui interface to end. This
/// is used for quitting the app or do something outside of the main tui interface.
fn check_app(res: Result<String, Box<dyn Error>>) -> String {
    exit_tui_interface().expect("Error exiting the interface");

    match res {
        Err(e) => {
            println!("Error: {:?}", e);
        },
        Ok(a) => {
            // the string is gotten from run_app to process the data here.
            if &a == "Change" {
                let db_data = get_user_tx_methods(true);
                if db_data == vec!["".to_string()] {  
                    println!("Operation Cancelled. Restarting in 5 seconds");
                    thread::sleep(Duration::from_millis(5000));
                }
                else {
                    let status = add_new_tx_methods(db_data);
                    match status {
                        Ok(_) => {
                            println!("Added Transaction Methods Successfully. The app will restart in 5 seconds");
                            thread::sleep(Duration::from_millis(5000));
                        },
                        Err(e) => {
                            println!("Error while adding new transaction methods. Error: {e:?}");
                            thread::sleep(Duration::from_millis(5000));
                        }
                    }
                }
                
            }
            else if &a == "Link" {
                println!("Could not open new version link.\n\nLink: https://github.com/WaffleMixer/Rex");
                return "break".to_string();
            }
            else {
                return "break".to_string();
            }
        }
    }
    return "".to_string();
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
) -> io::Result<String> {
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
    // starter_index : to keep track of the loop on each iteration on the initial page's animation.
    // update_found : The variable to check if there is a new version of the app.
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

    // TODO create a separate function in a different thread to check for latest release and update this variable
    // The next three variable will determine whether a certain pop is turned on or off.
    // Based on this, the popup is created/passed
    let mut update_popup_on = false;
    let mut help_popup_on = false;
    let mut delete_popup_on = false;

    // The next three vectors will store the lines that will be displayed on popup
    // The Vec contains popup title, text, x axis and y axis size.
    let mut popup_data_delete_failed = vec![];

    let mut popup_data_new_update = vec![];
    popup_data_new_update.push("New Update".to_string());
    popup_data_new_update.push("There is a new version available\n
'Enter' : Redirect to the new version\n\nPress Any Key to dismiss".to_string());
    popup_data_new_update.push("50".to_string());
    popup_data_new_update.push("30".to_string());

    let mut popup_data_help = vec![];
    popup_data_help.push("Help".to_string());
    popup_data_help.push("'Arrow Key' : Navigate
'A' : Add Transaction Page
'F' : Home Page
'D' : Delete selected Transaction (Home Page)
'J' : Add new Transaction Methods (Home Page)
'H' : Open Hotkey Help
'Q' : Quit

Add Transaction Page:
'1': Edit Date          '4': Edit Amount
'2': Edit TX details    '3': Edit TX Method
'5': Edit TX Type
'S' : Save the data as a Transaction
'Enter' or 'Esc': Submit/Stop editing field\n
Press Any Key to dismiss".to_string());
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
        let width_percent = 100 / balance[0].len() as u16 ;
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
            None => {
                balance.push(all_data.get_last_balance(&conn));
                balance.push(get_empty_changes(&conn));
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
                );
                // based on the bool variable, start a new popup window
                if help_popup_on == true {
                    create_popup(f, &popup_data_help)
                }
                else if delete_popup_on == true {
                    create_popup(f, &popup_data_delete_failed)
                }
            })?,
            CurrentUi::AddTx => terminal.draw(|f| {
                tx_ui(
                    f,
                    data_for_tx.get_all_texts(),
                    &cu_tx_page,
                    &data_for_tx.tx_status,
                );
                // based on the bool variable, start a new popup window
                if help_popup_on == true {
                    create_popup(f, &popup_data_help)
                }
            })?,
            CurrentUi::Initial => terminal.draw(|f| {                
                starter_ui(f, starter_index);
                starter_index += 1;
                if starter_index > 28 {
                    starter_index = 0;
                }
                // based on the bool variable, start a new popup window
                if update_popup_on == true {
                    create_popup(f, &popup_data_new_update);
                }
            })?,
        };

        // This is where the keyboard press tracking starts
        // What is poll? This is something from crossterm lib.
        // There are two options, event or timer. Timer keeps the loop unblocked. Loops for
        // event checking each 40 milliseconds
        if poll(Duration::from_millis(40))? {
            if let Event::Key(key) = event::read()? {
                match cu_page {
                    CurrentUi::Home => {
                        // we don't want to move the main UI while the popup is on
                        // so we verify that nothing is on and then check with the key presses.
                        if update_popup_on == true {
                            // Check if any popup is turned on and if yes, match key for the popup,
                            // otherwise match key for the main interface
                            match key.code {
                                KeyCode::Enter => {
                                    match open::that("https://github.com/WaffleMixer/Rex") {
                                        Ok(_) => update_popup_on = false,
                                        // if it fails for any reason, break interface and print the link
                                        Err(_) => return Ok("Link".to_string())
                                    }
                                },
                                _ => update_popup_on = false, 
                            }
                        } else if help_popup_on == true {
                            match key.code {
                                _ => help_popup_on = false, 
                            }
                        } else if delete_popup_on == true {
                            match key.code {
                                _ => delete_popup_on = false, 
                            }
                        } else {
                            match key.code {
                                KeyCode::Char('q') => return Ok("".to_string()),
                                KeyCode::Char('a') => cu_page = CurrentUi::AddTx,
                                KeyCode::Char('j') => return Ok("Change".to_string()),
                                KeyCode::Char('h') => help_popup_on = true,
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
                                                popup_data_delete_failed.push("Delete Error".to_string());
                                                popup_data_delete_failed.push(format!("Error while deleting transaction \
                                                \nError: {e:?}\n\nPress Any Key to dismiss"));
                                                popup_data_delete_failed.push("50".to_string());
                                                popup_data_delete_failed.push("30".to_string());
                                                delete_popup_on = true;

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
                                            else if table.state.selected() == Some(table.items.len() - 1)
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
                    },
                    CurrentUi::AddTx => {
                        // Check if any popup is turned on and if yes, match key for the popup,
                        // otherwise match key for the main interface
                        if help_popup_on == true {
                            match key.code {
                                _ => help_popup_on = false, 
                            }
                        }
                        else {
                            match cu_tx_page {
                                // start matching key pressed based on which widget is selected.
                                // current state tracked with enums
                                TxTab::Nothing => match key.code {
                                    KeyCode::Char('q') => return Ok("".to_string()),
                                    KeyCode::Char('f') => {
                                        cu_page = CurrentUi::Home;
                                        cu_tx_page = TxTab::Nothing;
                                        data_for_tx = AddTxData::new();
                                    }
                                    KeyCode::Char('h') => help_popup_on = true,
                                    KeyCode::Char('s') => {
                                        let status = data_for_tx.add_tx();
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
                                            Err(_) => data_for_tx.add_tx_status(
                                                "Date: Error acquired or Date not acceptable.",
                                            ),
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
                                            Err(_) => data_for_tx.add_tx_status(
                                                "Date: Error acquired or Date not acceptable.",
                                            ),
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
                                                if a.contains("zero") {
                                                } else {
                                                    cu_tx_page = TxTab::Nothing;
                                                }
                                            }
                                            Err(_) => data_for_tx
                                                .add_tx_status("Amount: Invalid Amount found"),
                                        }
                                    }
                                    KeyCode::Esc => {
                                        let status = data_for_tx.check_amount();
                                        match status {
                                            Ok(a) => {
                                                data_for_tx.add_tx_status(&a);
                                                if a.contains("zero") {
                                                } else {
                                                    cu_tx_page = TxTab::Nothing;
                                                }
                                            }
                                            Err(_) => data_for_tx
                                                .add_tx_status("Amount: Invalid Amount found"),
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
                            }
                        }
                    }
                    CurrentUi::Initial => {
                        // Check if any popup is turned on and if yes, match key for the popup,
                        // otherwise match key for the main interface
                        if update_popup_on == true {
                            match key.code {
                                KeyCode::Enter => {
                                    match open::that("https://github.com/WaffleMixer/Rex") {
                                        Ok(_) => update_popup_on = false,
                                        // if it fails for any reason, break interface and print the link
                                        Err(_) => return Ok("Link".to_string())
                                    }
                                },
                                _ => update_popup_on = false, 
                            }
                        }
                        else {
                            match key.code {
                                KeyCode::Char('q') => return Ok("".to_string()),
                                _ => cu_page = CurrentUi::Home,
                            }
                        }
                    },
                }
            };
        } else {}
    }
}
