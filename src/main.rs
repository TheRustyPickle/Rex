mod transaction_ui;
mod data_struct;
mod sub_func;
mod create_initial_db;
mod add_tx_ui;

use rusqlite::{Connection, Result};
use add_tx_ui::ui as tx_ui;
use transaction_ui::ui;
use sub_func::*;
use data_struct::{TimeData, TableData, SelectedTab, TransactionData};
use std::{error::Error, io};
use tui::{backend::{Backend, CrosstermBackend}, Terminal,};
use tui::layout::Constraint;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

// [ ] Check current path for the db, create new db if necessary
// [ ] create add transaction ui + editing box with inputs
// [ ] create initial ui asking for tx methods
// [ ] add creating tx button
// [ ] add remvoing tx button
// [ ] create a popup ui on tx data window for commands list
// [ ] allow manually changing tx methods balances
// [ ] change color scheme?

fn main() -> Result<(), Box<dyn Error>>{
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let months = TimeData::new(vec!["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"]);
    let years = TimeData::new(vec!["2021", "2022", "2023", "2024", "2025", "2026"]);
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
    let mut all_data = TransactionData::new(&conn, 0, 0);
    let mut table = TableData::new(all_data.get_txs());
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
            // Do a +1 because the index starts at 0
            Some(a) => {
                balance.push(all_data.get_balance(a as i32 + 1));
                balance.push(all_data.get_changes(a as i32 + 1));
            },
            None => {
                balance.push(all_data.get_last_balance());
                balance.push(get_empty_changes());
            },
        }

        //terminal.draw(|f| tx_ui(f,))?;
        terminal.draw(|f| ui(f, &months, &years, &mut table, &mut balance, &selected_tab, &mut width_data))?; 
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
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
                            // Do not proceed to the table section If
                            // there is no transaction
                            if all_data.all_tx.len() < 1 {
                                selected_tab = selected_tab.change_tab_up();
                            }
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
                            if all_data.all_tx.len() < 1 {
                                selected_tab = selected_tab.change_tab_up();
                            }
                            else {
                                // Move to the last value on table if pressed up on Year section
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
            };
        };
    }
}