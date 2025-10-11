use anyhow::Result;
use atty::Stream;
use rex_app::conn::get_conn;
use std::env::set_current_dir;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process;

use crate::config::{Config, migrate_config};
use crate::outputs::HandlingOutput;
use crate::page_handler::start_app;
use crate::utility::{
    check_version, enter_tui_interface, exit_tui_interface, migrate_to_new_schema, start_terminal,
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
