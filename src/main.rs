mod table_ui;
mod ui_data_state;
mod sub_func;
mod create_initial_db;
mod add_tx_ui;
mod table_data;
mod add_tx_data;

use std::fs;
use create_initial_db::create_db;
use rusqlite::{Connection, Result};
use add_tx_ui::tx_ui;
use table_ui::ui;
use sub_func::{get_all_tx_methods, get_empty_changes};
use table_data::TransactionData;
use add_tx_data::AddTxData;
use ui_data_state::*;
use std::{error::Error, io};
use tui::{backend::{Backend, CrosstermBackend}, Terminal,};
use tui::layout::Constraint;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

// [x] Check current path for the db, create new db if necessary
// [x] create add transaction ui + editing box with inputs
// [x] func for saving & deleting txs
// [ ] create initial ui asking for tx methods
// [x] add creating tx button
// [x] add remvoing tx button
// [ ] create a popup ui on Home window for commands list
// [ ] allow adding/removing tx methods(will require renaming columns)
// [ ] change color scheme?
// [x] change balances to f32?
// [x] add date column to all_balance & all_changes
// [x] verify db cascade method working or not
// [ ] add more panic handling
// [ ] add save points for db commits
// [x] latest balance empty = all 0
// [x] limit add tx date between the available years
// [x] add status on add tx page
// [ ] add average expense on home page
// [ ] add more comments

fn main() -> Result<(), Box<dyn Error>>{
    let paths = fs::read_dir(".").unwrap();
    let mut db_found = false;
    for path in paths {
        let path = path.unwrap().path().display().to_string();
        if path == "./data.sqlite"{
            db_found = true;
        }
    }
    // TODO if db not found take user input for tx method during the initialization
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
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let months = TimeData::new(vec!["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"]);
    let years = TimeData::new(vec!["2022", "2023", "2024", "2025"]);
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

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut months: TimeData, mut years: TimeData) -> io::Result<()> {
    let mut selected_tab = SelectedTab::Months;
    let mut last_month_index = 99;
    let mut last_year_index = 99;
    let path = "data.sqlite";
    let conn = Connection::open(path).expect("Could not connect to database");
    conn.execute("PRAGMA foreign_keys = ON", []).expect("Could not enable foreign keys");
    let mut all_data = TransactionData::new(&conn, 0, 0);
    let mut table = TableData::new(all_data.get_txs());
    let mut cu_page = CurrentUi::Home;
    let mut cu_tx_page = TxTab::Nothing;
    let mut data_for_tx = AddTxData::new();

    loop {
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

        let mut balance: Vec<Vec<String>> = vec![
            vec!["".to_string()]
        ];
        balance[0].extend(get_all_tx_methods(&conn));

        // save the % of space each column should take in the Balance section
        let width_percent = 100 / balance[0].len() as u16;
        let mut width_data = vec![];
        for _i in 0..balance[0].len()  {
            width_data.push(Constraint::Percentage(width_percent));
        }

        match cu_table_index {
            // pass out the current index to get the necessary balance & changes data
            Some(a) => {
                balance.push(all_data.get_balance(a));
                balance.push(all_data.get_changes(a));
            },
            None => {
                balance.push(all_data.get_last_balance(&conn));
                balance.push(get_empty_changes());
            },
        }

        match cu_page {
            //NOTE initial ui to be added here
            CurrentUi::Home => terminal.draw(|f| ui(f, &months, &years, &mut table, &mut balance, &selected_tab, &mut width_data))?,
            CurrentUi::AddTx => terminal.draw(|f| tx_ui(f, data_for_tx.get_all_texts(), &cu_tx_page, &data_for_tx.tx_status), )?,
        };
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
                                    all_data = TransactionData::new(&conn, cu_month_index, cu_year_index);
                                    table = TableData::new(all_data.get_txs());
                                    table.state.select(None);
                                    selected_tab = SelectedTab::Months;
                                },
                                    
                                Err(_) => {}
                            }
                        }
                    }
                    KeyCode::Right => {
                        match &selected_tab {
                            SelectedTab::Months => months.next(),
                            SelectedTab::Years => {
                                years.next();
                                months.index = 0;
                            },
                            _ => {}
                        }
                    },
                    KeyCode::Left => {
                        match &selected_tab {
                            SelectedTab::Months => months.previous(),
                            SelectedTab::Years => {
                                years.previous();
                                months.index = 0;
                            },
                            _ => {}
                        }
                    },
                    KeyCode::Up => {
                        match &selected_tab{
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
                                }
                                else {
                                    if all_data.all_tx.len() > 0 {
                                        table.previous();
                                    }
                                }
                            },
                            SelectedTab::Years => {
                                // Do not select any table rows in the table section If
                                // there is no transaction
                                if all_data.all_tx.len() < 1 {
                                    selected_tab = selected_tab.change_tab_up();
                                }
                                else {
                                    // Move to the selected value on table
                                    // to the last row if pressed up on Year section
                                    table.state.select(Some(table.items.len() - 1));
                                    selected_tab = selected_tab.change_tab_up();
                                    if all_data.all_tx.len() < 1 {
                                    selected_tab = selected_tab.change_tab_up();
                                    }
                                }
                                
                            }
                            _ => selected_tab = selected_tab.change_tab_up()
                        }
                    },
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
                                }
                                else {
                                    if all_data.all_tx.len() > 0 {
                                        table.next();
                                    }
                                }
                            }
                            _ => selected_tab = selected_tab.change_tab_down(),
                        }
                    },
                    _ => {}
                    }
                CurrentUi::AddTx => match cu_tx_page {
                    // start matching key pressed based on which widget is selected.
                    // current state tracked with enums
                    TxTab::Nothing => match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('h') => {
                            cu_page = CurrentUi::Home;
                            cu_tx_page = TxTab::Nothing;
                            data_for_tx = AddTxData::new();
                        },
                        KeyCode::Char('s') => {
                            let _status = data_for_tx.add_tx(&conn);
                            cu_page = CurrentUi::Home;
                            data_for_tx = AddTxData::new();
                        },
                        KeyCode::Char('1') => cu_tx_page = TxTab::Date,
                        KeyCode::Char('2') => cu_tx_page = TxTab::Details,
                        KeyCode::Char('3') => cu_tx_page = TxTab::TxMethod,
                        KeyCode::Char('4') => cu_tx_page = TxTab::Amount,
                        KeyCode::Char('5') => cu_tx_page = TxTab::TxType,
                        KeyCode::Enter => cu_tx_page = TxTab::Nothing,
                        KeyCode::Esc => cu_tx_page = TxTab::Nothing,
                        _ => {}
                    }

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
                                Err(_) => data_for_tx.add_tx_status("Date: Error acquired or Date not acceptable."),
                                
                            }
                        },
                        KeyCode::Esc => {
                            let status = data_for_tx.check_date();
                            match status {
                                Ok(a) => {
                                    data_for_tx.add_tx_status(&a);
                                    if a.contains("Accepted") {
                                        cu_tx_page = TxTab::Nothing
                                    }
                                    
                                }
                                Err(_) => data_for_tx.add_tx_status("Date: Error acquired or Date not acceptable."),
                                
                            }
                        },
                        KeyCode::Backspace => data_for_tx.edit_date('a', true),
                        KeyCode::Char(a) => data_for_tx.edit_date(a, false),
                        _ => {}
                    }

                    TxTab::Details => match key.code {
                        KeyCode::Enter => cu_tx_page = TxTab::Nothing,
                        KeyCode::Esc => cu_tx_page = TxTab::Nothing,
                        KeyCode::Backspace => data_for_tx.edit_details('a', true),
                        KeyCode::Char(a) => data_for_tx.edit_details(a, false),
                        _ => {}
                    }

                    TxTab::TxMethod => match key.code {
                        KeyCode::Enter => {
                            let status = data_for_tx.check_tx_method(&conn);
                            data_for_tx.add_tx_status(&status);
                            if status.contains("Accepted") || status.contains("Nothing") {
                                cu_tx_page = TxTab::Nothing
                            }
                        },
                        KeyCode::Esc => {
                            let status = data_for_tx.check_tx_method(&conn);
                            data_for_tx.add_tx_status(&status);
                            if status.contains("Accepted") {
                                cu_tx_page = TxTab::Nothing
                            }
                        },
                        KeyCode::Backspace => data_for_tx.edit_tx_method('a', true),
                        KeyCode::Char(a) => data_for_tx.edit_tx_method(a, false),
                        _ => {}
                    }

                    TxTab::Amount => match key.code {
                        KeyCode::Enter => {
                            let status = data_for_tx.check_amount();
                            match status {
                                Ok(a) => {
                                    data_for_tx.add_tx_status(&a);
                                    cu_tx_page = TxTab::Nothing;
                                },
                                Err(_) => data_for_tx.add_tx_status("Amount: Invalid Amount found")
                            }
                        },
                        KeyCode::Esc => {
                            let status = data_for_tx.check_amount();
                            match status {
                                Ok(a) => {
                                    data_for_tx.add_tx_status(&a);
                                    cu_tx_page = TxTab::Nothing;
                                },
                                Err(_) => data_for_tx.add_tx_status("Amount: Invalid Amount found")
                            }
                        },
                        KeyCode::Backspace => data_for_tx.edit_amount('a', true),
                        KeyCode::Char(a) => data_for_tx.edit_amount(a, false),
                        _ => {}
                    }

                    TxTab::TxType => match key.code {
                        KeyCode::Enter => {
                            let status = data_for_tx.check_tx_type();
                            data_for_tx.add_tx_status(&status);
                            if status.contains("Accepted") || status.contains("Nothing"){
                                cu_tx_page = TxTab::Nothing
                            }
                        },
                        KeyCode::Esc => cu_tx_page = TxTab::Nothing,
                        KeyCode::Backspace => data_for_tx.edit_tx_type('a', true),
                        KeyCode::Char(a) => data_for_tx.edit_tx_type(a, false),
                        _ => {}
                    }
                }
            }
        };
    }
}