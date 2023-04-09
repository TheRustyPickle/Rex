use dirs::data_local_dir;
use rex::page_handler::initialize_app;
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
// [x] update string calculation.
// [x]: Update REX logo algorithm to something more readable
// ? use a separate library for string calculation?
// TODO: Allow pasting
// TODO: On date selection, arrow key up down will change the date by 1, also for amount
// TODO: Add left/right arrow key to navigate strings
// ? Create a trait for easy implementation?
// [x]: Accept tx method in lower case
// TODO: Trait for method checker, input inserter
// TODO: Autocomplete for tx method
// TODO: Tx method autocomplete take consideration of the char position
// TODO: Handle repeating keys in a separate function
// [x]: instead of passing too many args to key checkers, turn all of them into a struct and pass that
// [x]: Move stuff from lib.rs
// TODO: replace all cu_something to current_something
// TODO: Handle tags checking with comma.
// [x]: Merge AddTxData and TransferData into one struct?
// TODO: Better logic when taking input for new tx methods
// TODO: better sql construction
// TODO: Allow hiding year selection on chart page
// TODO: Allow some kind of chart animation. Use the poll, create var that will track how many days it needs to render. Pass the var => render 1 day => return
// TODO load all data from db to summary and chart struct. Only reload on new tx

fn main() {
    if let Some(dir) = data_local_dir() {
        let current_dir = current_dir().unwrap().display().to_string();
        let mut verifying_path = "./data.sqlite";

        // * OS based path where data will be stored
        let working_path = format!("{}/Rex/", dir.display());

        if cfg!(target_os = "windows") {
            verifying_path = r#".\data.sqlite"#;
        }
        // * Create folder if non-existing then move the current working directory
        // * to the OS data directory
        fs::create_dir_all(&working_path).unwrap();
        set_current_dir(working_path).unwrap();
        if initialize_app(verifying_path, &current_dir).is_err() {
            std::process::exit(1);
        }
    } else {
        println!("Could not find local data directory. Exiting program...");
    }
}
