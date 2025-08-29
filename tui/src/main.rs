use dirs::data_local_dir;
use log::LevelFilter;
use rex_tui::page_handler::initialize_app;
use simplelog::{Config, WriteLogger};
use std::env::{current_dir, set_current_dir};
use std::fs::{self, File};

fn main() {
    let _ = WriteLogger::init(
        LevelFilter::Info,
        Config::default(),
        File::create("logs.log").unwrap(),
    );

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

        working_path.push("data.sqlite");
        if initialize_app(&working_path, &original_dir).is_err() {
            std::process::exit(1);
        }
    } else {
        println!("Could not find local data directory. Exiting program...");
    }
}
