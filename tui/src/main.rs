mod config;
mod key_checker;
mod outputs;
mod page_handler;
mod pages;
mod theme;
mod tx_handler;
mod utility;

use dirs::data_local_dir;
use page_handler::initialize_app;
use std::env::{current_dir, set_current_dir};
use std::fs;

fn main() {
    if let Some(dir) = data_local_dir() {
        // The path where the application was opened initially
        let original_dir = current_dir().unwrap();

        // The OS based path where data will be stored
        let mut working_path = dir;
        working_path.push("Rex");
        // Create folder if non-existing then move the current working directory
        // to the OS data directory
        let Ok(()) = fs::create_dir_all(&working_path) else {
            println!("Failed to work with the working path. Exiting program...");
            return;
        };

        let Ok(()) = set_current_dir(&working_path) else {
            println!("Failed to set the working path. Exiting program...");
            return;
        };

        let mut migrated_path = working_path.clone();
        migrated_path.push("rex.sqlite");

        working_path.push("data.sqlite");
        if let Err(e) = initialize_app(&working_path, &migrated_path, &original_dir) {
            println!("Failed to initialize app. Error: {e:?}");
            std::process::exit(1);
        }
    } else {
        println!("Could not find local data directory. Exiting program...");
    }
}
