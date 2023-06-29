use dirs::data_local_dir;
use rex_tui::page_handler::initialize_app;
use std::env::{current_dir, set_current_dir};
use std::fs;

fn main() {
    if let Some(dir) = data_local_dir() {
        let current_dir = current_dir().unwrap().display().to_string();
        let mut verifying_path = "./data.sqlite";

        // OS based path where data will be stored
        let working_path = format!("{}/Rex/", dir.display());

        if cfg!(target_os = "windows") {
            verifying_path = r#".\data.sqlite"#;
        }
        // Create folder if non-existing then move the current working directory
        // to the OS data directory
        fs::create_dir_all(&working_path).unwrap();
        set_current_dir(working_path).unwrap();
        if initialize_app(verifying_path, &current_dir).is_err() {
            std::process::exit(1);
        }
    } else {
        println!("Could not find local data directory. Exiting program...");
    }
}
