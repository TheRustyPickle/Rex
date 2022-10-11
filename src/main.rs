fn main() {
    let mut is_windows = false;
    let mut verifying_path = "./data.sqlite";
    // change details if running on windows
    if cfg!(target_os = "windows") {
        is_windows = true;
        verifying_path = r#".\data.sqlite"#;
    }
    rex::initializer(is_windows, verifying_path).unwrap();
}

// TODO consider b as current balance when doing calculation in the UI
// TODO create a separate page that collects all tags and shows total incomes or expenses
// TODO Auto upgrade DB if using the old version
// TODO update UI with the tag input field
// TODO Update functions that breaks
// TODO pass exiting tests and add new ones if necessary
// TODO check if version checker can be done in thread
// TODO edit initial page, popup
// TODO edit add tx and transfer page help text
// [ ] add this ALTER TABLE tx_all ADD tags TEXT DEFAULT Unknown;