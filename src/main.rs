use dirs::data_local_dir;
use std::env::{current_dir, set_current_dir};
use std::fs;

// TODO: add a page for searching txs. Support method: tag: type: detail: date: :amount> :amount< :amount= or :amount
// TODO: Update summary page. Include 2 tables instead of texts at the top, divided vertically. 1 will show the current summary. another will show year + income + expense
// TODO: Update total year to 2030
// TODO: Autofill tag fields. Show autofill in blue color
// TODO: Allow moving pointer on input fields
// TODO: allow removing chars from anywhere from input fields
// TODO: Allow changing year in the chart page. All | year | year
// TODO: Allow year selection of the summary page. All | year | year
// TODO: Check if I can center point on transfer page from the position of the latest char
// TODO update string calculation.
// TODO: Update REX logo algorithm to something more readable
// ? use a separate library for string calculation?
// TODO: Allow pasting
// TODO: On date selection, arrow key up down will change the date by 1
// TODO: Add left/right arrow key to navigate strings
// ? Create a trait for easy implementation?

fn main() {
    if let Some(dir) = data_local_dir() {
        let mut is_windows = false;
        let current_dir = current_dir().unwrap().display().to_string();
        let mut verifying_path = "./data.sqlite";

        // * OS based path where data will be stored
        let working_path = format!("{}/Rex/", dir.display());

        if cfg!(target_os = "windows") {
            is_windows = true;
            verifying_path = r#".\data.sqlite"#;
        }
        // * Create folder if non-existing then move the current working directory
        // * to the OS data directory
        fs::create_dir_all(&working_path).unwrap();
        set_current_dir(working_path).unwrap();
        rex::initializer(is_windows, verifying_path, &current_dir).unwrap();
    } else {
        println!("Could not find local data directory. Exiting program...");
    }
}
