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
// TODO find how to use cargo install
