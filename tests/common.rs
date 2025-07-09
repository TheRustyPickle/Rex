use rex_tui::db::create_db;
use rusqlite::Connection;
use std::fs;

pub fn create_test_db(file_name: &str) -> Connection {
    if let Ok(metadata) = fs::metadata(file_name) {
        if metadata.is_file() {
            fs::remove_file(file_name).expect("Failed to delete existing file");
        }
    }

    let mut conn = Connection::open(file_name).unwrap();
    create_db(
        &["Super Special Bank".to_string(), "Cash Cow".to_string()],
        &mut conn,
    )
    .unwrap();
    conn
}
