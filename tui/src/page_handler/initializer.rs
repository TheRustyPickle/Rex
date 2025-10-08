use anyhow::Result;
use app::conn::get_conn;
use atty::Stream;
use std::env::set_current_dir;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process;

use crate::config::{Config, migrate_config};
use crate::outputs::HandlingOutput;
use crate::page_handler::{ResetType, UserInputType, start_app};
use crate::utility::{
    check_version, enter_tui_interface, exit_tui_interface, migrate_to_new_schema,
    start_taking_input, start_terminal, start_timer,
};

/// Initialize the TUI loop
pub fn initialize_app(
    old_db_path: &PathBuf,
    migrated_db_path: &Path,
    original_dir: &PathBuf,
) -> Result<()> {
    // If is not terminal, try to start a terminal otherwise create an error.txt file with the error message
    if !atty::is(Stream::Stdout) && !start_terminal(original_dir.to_str().unwrap()) {
        let mut error_location = PathBuf::from(&original_dir);
        error_location.push("Error.txt");

        let mut open = File::create(error_location)?;
        let to_write =
            "Failed to start a terminal. Please open one manually by executing the binary"
                .to_string();
        open.write_all(to_write.as_bytes())?;
        process::exit(1);
    }

    let result =
        migrate_to_new_schema(old_db_path, migrated_db_path.display().to_string().as_str());

    match result {
        Ok(result) => {
            if result {
                println!("Restart required after migration. Exiting");
                process::exit(0);
            }
        }
        Err(e) => {
            println!("Failed to migrate to new schema. Error: {e:?}");
            fs::remove_file(migrated_db_path)?;
            process::exit(1);
        }
    }

    let new_version = check_version()?;

    if let Err(e) = migrate_config(old_db_path) {
        println!("Failed to migrate config. Error: {e:?}");
        process::exit(1);
    }

    let mut config = Config::get_config(&migrated_db_path.to_path_buf())?;

    let new_db_path = if let Some(mut location) = config.new_location.clone() {
        let result = set_current_dir(&location);

        if let Err(e) = result {
            println!(
                "Failed to set the new path. Does the path exists? Exiting program. Error: {e}"
            );
            process::exit(1);
        } else {
            location.push("rex.sqlite");
            location.clone()
        }
    } else {
        migrated_db_path.to_path_buf()
    };

    let mut migrated_conn = get_conn(new_db_path.display().to_string().as_str());

    loop {
        let mut terminal = enter_tui_interface()?;
        let result = start_app(&mut terminal, &new_version, &mut config, &mut migrated_conn);
        exit_tui_interface()?;

        match result {
            Ok(output) => match output {
                HandlingOutput::TakeUserInput => match start_taking_input(&mut migrated_conn) {
                    UserInputType::AddNewTxMethod(tx_methods) => {
                        let status = migrated_conn.add_new_methods(&tx_methods);

                        match status {
                            Ok(()) => start_timer("Added Transaction Methods Successfully."),
                            Err(e) => {
                                println!(
                                    "Error while adding new Transaction Methods. Error: {e:?}."
                                );
                                start_timer("");
                            }
                        }
                    }
                    UserInputType::RenameTxMethod(rename_data) => {
                        let old_name = &rename_data[0];
                        let new_name = &rename_data[1];

                        let status = migrated_conn.rename_tx_method(old_name, new_name);

                        match status {
                            Ok(()) => start_timer("Tx Method renamed successfully."),
                            Err(e) => {
                                println!("Error while renaming tx method. Error: {e:?}.");
                                start_timer("");
                            }
                        }
                    }
                    UserInputType::RepositionTxMethod(tx_methods) => {
                        let status = migrated_conn.set_new_tx_method_positions(&tx_methods);

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
                        if let Err(e) = config.set_new_location(target_path.clone()) {
                            println!("Error while setting new location. Error: {e:?}");
                            start_timer("");
                            continue;
                        }

                        target_path.push("data.sqlite");
                        let file_copy_status = fs::copy(&new_db_path, target_path);

                        match file_copy_status {
                            Ok(_) => {
                                start_timer(
                                    "New location set successfully. The app must be restarted for it to take effect. It will exit after this.",
                                );
                                process::exit(0)
                            }
                            Err(e) => {
                                println!("Error while trying to copy app data. Error: {e:?}");
                                start_timer("");
                            }
                        }
                    }
                    UserInputType::BackupDBPath(paths) => {
                        if let Err(e) = config.set_backup_db_path(paths) {
                            println!("Error while setting backup DB path. Error: {e:?}");
                            start_timer("");
                            continue;
                        }

                        start_timer("Backup DB path locations set successfully.");
                    }
                    UserInputType::ResetData(reset_type) => match reset_type {
                        ResetType::NewLocation => match config.reset_new_location() {
                            Ok(()) => {
                                start_timer(
                                    "New location data removed successfully. The app must be restarted for it to take effect. It will exit after this.",
                                );
                                process::exit(0)
                            }
                            Err(e) => {
                                println!(
                                    "Error while trying to delete saved location data. Error: {e:?}"
                                );
                                start_timer("");
                            }
                        },
                        ResetType::BackupDB => match config.reset_backup_db_path() {
                            Ok(()) => start_timer("Backup DB Path removed successfully."),
                            Err(e) => {
                                println!(
                                    "Error while trying to delete saved backup location data. Error: {e:?}"
                                );
                                start_timer("");
                            }
                        },
                    },
                    UserInputType::InvalidInput => unreachable!(),
                },
                HandlingOutput::QuitUi => {
                    drop(migrated_conn);
                    config.save_backup(&new_db_path.clone());
                    break;
                }
                HandlingOutput::PrintNewUpdate => println!(
                    "Could not open browser.\n\nLatest Version Link: https://github.com/TheRustyPickle/Rex/releases/latest"
                ),
            },
            Err(error) => {
                println!("{error}");
                process::exit(1);
            }
        }
    }

    Ok(())
}
