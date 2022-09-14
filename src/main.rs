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
