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

// new release targets
// [x] Transfer balance between methods
// [x] yearly chart showing balance changes
// [x] maintain the ID num of the db when editing transaction
// [x] konsole error.txt being created
// [x] do simple calculation on amount field
// [x] move popup to an enum
// [x] move key checker to a different file
// [x] move popup texts to a different file
// [ ] add tests for the structs and trait
// [x] Find how to add transaction for transfer
// [x] find how to delete transaction for transfer
// [x] Allow editing for transfer tx
// [ ] move tests to tests folder
// [x] slight bug on total amount field and total balance calculation
