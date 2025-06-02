use atty::Stream;
use rusqlite::Connection;
use std::env::set_current_dir;
use std::error::Error;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::PathBuf;
use std::process;

use crate::db::{add_new_tx_methods, rename_column, reposition_column};
use crate::initial_page::check_version;
use crate::outputs::HandlingOutput;
use crate::page_handler::{start_app, ResetType, UserInputType};
use crate::utility::{
    check_n_create_db, check_old_sql, create_backup_location_file, create_change_location_file,
    delete_backup_db, delete_location_change, enter_tui_interface, exit_tui_interface,
    is_location_changed, save_backup_db, start_taking_input, start_terminal, start_timer,
};

/// Initialize the TUI loop
#[cfg(not(tarpaulin_include))]
pub fn initialize_app(
    original_db_path: &PathBuf,
    original_dir: &PathBuf,
) -> Result<(), Box<dyn Error>> {
    let new_version = check_version();

    let new_version_available = new_version.unwrap_or_default();
    // If is not terminal, try to start a terminal otherwise create an error.txt file with the error message
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

    // If the location was changed/json file found, change the db directory.
    let db_path = if let Some(mut location) = is_location_changed(original_db_path) {
        let Ok(_) = set_current_dir(&location) else {
            println!("Failed to set the new path. Exiting program...");
            std::process::exit(1);
        };
        location.push("data.sqlite");
        location
    } else {
        original_db_path.clone()
    };

    // Create a new db if not found. If there is an error, delete the failed data.sqlite file and exit
    check_n_create_db(&db_path)?;

    let mut conn = Connection::open(&db_path)?;

    // Initiates migration if old database is detected.
    check_old_sql(&mut conn);

    loop {
        let mut terminal = enter_tui_interface()?;
        let result = start_app(&mut terminal, &new_version_available, &mut conn);
        exit_tui_interface()?;

        match result {
            Ok(output) => match output {
                HandlingOutput::TakeUserInput => match start_taking_input(&conn) {
                    UserInputType::AddNewTxMethod(tx_methods) => {
                        let status = add_new_tx_methods(&tx_methods, &mut conn);
                        match status {
                            Ok(()) => start_timer("Added Transaction Methods Successfully."),
                            Err(e) => {
                                println!("Error while adding new Transaction Methods. Error: {e:?}.");
                                start_timer("");}
                        }
                    }
                    UserInputType::RenameTxMethod(rename_data) => {
                        let old_name = &rename_data[0];
                        let new_name = &rename_data[1];

                        let status = rename_column(old_name, new_name, &mut conn);

                        match status {
                            Ok(()) => start_timer("Tx Method renamed successfully."),
                            Err(e) => {
                                println!("Error while renaming tx method. Error: {e:?}.");
                                start_timer("");
                            }
                        }
                    }
                    UserInputType::RepositionTxMethod(tx_methods) => {
                        let status = reposition_column(&tx_methods, &mut conn);

                        match status {
                            Ok(()) => start_timer("Transaction Method repositioned successfully."),
                            Err(e) => {
                                println!("Error while repositioning tx method. Error: {e:?}");
                                start_timer("");
                            }
                        }
                    }
                    UserInputType::CancelledOperation => {
                        start_timer("Operation Cancelled.");
                    }
                    UserInputType::SetNewLocation(mut target_path) => {
                        create_change_location_file(&db_path, &target_path);

                        target_path.push("data.sqlite");
                        let file_copy_status = fs::copy(&db_path, target_path);

                        match file_copy_status {
                            Ok(_) => {
                                start_timer("New location set successfully. The app must be restarted for it to take effect. It will exit after this.");
                                process::exit(0)
                            }
                            Err(e) => {
                                println!("Error while trying to copy app data. Error: {e:?}");
                                start_timer("");
                            }
                        }
                    }
                    UserInputType::BackupDBPath(paths) => {
                        create_backup_location_file(original_db_path, paths);

                        start_timer("Backup DB path locations set successfully.");
                    }
                    UserInputType::ResetData(reset_type) => {

                        match reset_type {
                            ResetType::NewLocation => {
                                match delete_location_change(original_db_path) {
                                    Ok(()) => {
                                        start_timer("New location data removed successfully. The app must be restarted for it to take effect. It will exit after this.");
                                        process::exit(0)
                                    }
                                    Err(e) => {
                                        println!("Error while trying to delete saved location data. Error: {e:?}");
                                        start_timer("");
                                    }
                                }
                            }
                            ResetType::BackupDB => {
                                match delete_backup_db(original_db_path) {
                                    Ok(()) => start_timer("Backup DB Path removed successfully."),
                                    Err(e) => {
                                        println!("Error while trying to delete saved backup location data. Error: {e:?}");
                                        start_timer("");
                                    }
                                }
                            }
                        }
                    }
                    UserInputType::InvalidInput => unreachable!()
                },
                HandlingOutput::QuitUi => {
                    save_backup_db(&db_path, original_db_path);
                    break;
                },
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
