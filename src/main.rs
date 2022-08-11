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
// [ ] Transfer balance between methods and add transaction with amount 0 showing changes
// [ ] yearly chart showing balance changes
// [ ] maintain the ID num of the db when editing transaction
// [ ] konsole error.txt being created
// [ ] do simple calculation on amount field
// [ ] move popup to an enum and then cu_tab(enum) to check
// [ ] move key checker to a different file