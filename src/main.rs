use dirs::data_local_dir;
use rex_tui::page_handler::initialize_app;
use std::env::{current_dir, set_current_dir};
use std::fs;

fn main() {
    if let Some(dir) = data_local_dir() {
        // The path where the application was opened initially
        let original_dir = current_dir().unwrap();

        // OS based path where data will be stored
        let mut working_path = dir;
        working_path.push("Rex");
        // Create folder if non-existing then move the current working directory
        // to the OS data directory
        fs::create_dir_all(&working_path).unwrap();
        set_current_dir(&working_path).unwrap();

        working_path.push("data.sqlite");
        if initialize_app(working_path, original_dir).is_err() {
            std::process::exit(1);
        }
    } else {
        println!("Could not find local data directory. Exiting program...");
    }
}
