use app::conn::{DbConn, get_conn};
use std::fs;

#[must_use]
pub fn create_test_db(file_name: &str) -> DbConn {
    if let Ok(metadata) = fs::metadata(file_name)
        && metadata.is_file()
    {
        fs::remove_file(file_name).expect("Failed to delete existing file");
    }

    let mut conn = get_conn(file_name);
    let tx_methods = ["Cash", "Bank", "Other"]
        .iter()
        .map(|name| name.to_string())
        .collect::<Vec<String>>();

    conn.add_new_methods(&tx_methods).unwrap();

    conn
}
