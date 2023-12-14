use crate::db::{add_new_tx_methods, rename_column, reposition_column};
use crate::initial_page::check_version;
use crate::outputs::HandlingOutput;
use crate::page_handler::start_app;
use crate::utility::{
    check_n_create_db, check_old_sql, enter_tui_interface, exit_tui_interface, start_taking_input,
    start_terminal, start_timer,
};
use atty::Stream;
use rusqlite::Connection;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process;

use super::UserInputType;

#[cfg(not(tarpaulin_include))]
pub fn initialize_app(working_dir: PathBuf, original_dir: PathBuf) -> Result<(), Box<dyn Error>> {
    let new_version_available = check_version()?;
    if !atty::is(Stream::Stdout) {
        if let Err(err) = start_terminal(original_dir.to_str().unwrap()) {
            let mut error_location = PathBuf::from(&original_dir);
            error_location.push("Error.txt");

            let mut open = File::create(error_location)?;
            let to_write = format!("{}\n{}", original_dir.to_str().unwrap(), err);
            open.write_all(to_write.as_bytes())?;
            process::exit(1);
        }
    }

    // create a new db if not found. If there is an error, delete the failed data.sqlite file and exit
    check_n_create_db(&working_dir)?;

    let mut conn = Connection::open(&working_dir)?;

    // initiates migration if old database is detected.
    check_old_sql(&mut conn);

    loop {
        let mut terminal = enter_tui_interface()?;
        let result = start_app(&mut terminal, &new_version_available, &mut conn);
        exit_tui_interface()?;

        match result {
            Ok(output) => match output {
                HandlingOutput::TakeUserInput => match start_taking_input( &conn) {
                    UserInputType::AddNewTxMethod(tx_methods) => {
                        let status = add_new_tx_methods(tx_methods, &mut conn);
                        match status {
                            Ok(_) => start_timer("Added Transaction Methods Successfully."),
                            Err(e) => {
                                println!("Error while adding new Transaction Methods. Error: {e:?}.");
                                start_timer("")}
                        }
                    }
                    UserInputType::RenameTxMethod(rename_data) => {
                        let old_name = &rename_data[0];
                        let new_name = &rename_data[1];

                        let status = rename_column(old_name, new_name, &mut conn);

                        match status {
                            Ok(_) => start_timer("Tx Method renamed successfully."),
                            Err(e) => {
                                println!("Error while renaming tx method. Error: {e:?}.");
                                start_timer("")
                            }
                        }
                    }
                    UserInputType::RepositionTxMethod(tx_methods) => {
                        let status = reposition_column(tx_methods, &mut conn);

                        match status {
                            Ok(_) => start_timer("Transaction Method repositioned successfully."),
                            Err(e) => {
                                println!("Error while repositioning tx method. Error: {e:?}");
                                start_timer("");
                            }
                        }
                    }
                    UserInputType::CancelledOperation => {
                        start_timer("Operation Cancelled.")
                    }
                    _ => {}
                },
                HandlingOutput::QuitUi => break,
                HandlingOutput::PrintNewUpdate => println!("Could not open browser.\n\nLatest Version Link: https://github.com/TheRustyPickle/Rex/releases/latest")
            },
            Err(error) => {
                println!("{error}");
                process::exit(1);
            }
        }
    }

    Ok(())
}
