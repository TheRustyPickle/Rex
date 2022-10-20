use dirs::data_local_dir;
use std::env::set_current_dir;
use std::fs;
fn main() {
    if let Some(dir) = data_local_dir() {
        let mut is_windows = false;
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
        rex::initializer(is_windows, &verifying_path).unwrap();
    } else {
        println!("Could not find local data directory. Exiting program...");
    }
}

// [x] consider b as current balance when doing calculation in the UI
// [x] create a separate page that collects all tags and shows total incomes or expenses
// [x] Auto upgrade DB if using the old version
// [x] update UI with the tag input field
// [x] Update functions that breaks
// [x] pass existing tests and add new ones if necessary
// [x] check if version checker can be done in thread not worth it
// [x] edit initial page, popup
// [x] edit add tx and transfer page help text
// [x] add this ALTER TABLE tx_all ADD tags TEXT DEFAULT Unknown;
// [x] maintain tx method order during db creation
// [x] find how to use cargo install. dirs crate
