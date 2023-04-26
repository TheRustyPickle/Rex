extern crate rex_tui;
use rex_tui::db::{add_new_tx_methods, create_db};
use rusqlite::Connection;
use std::fs;

#[test]
fn check_db_creation() {
    let file_name = "test_data_1.sqlite";

    if let Ok(metadata) = fs::metadata(file_name) {
        if metadata.is_file() {
            fs::remove_file(file_name).expect("Failed to delete existing file");
        }
    }
    let mut conn = Connection::open(file_name).unwrap();

    create_db(vec!["test1".to_string(), "test 2".to_string()], &mut conn).unwrap();
    let paths = fs::read_dir(".").unwrap();
    let mut db_found = false;
    for path in paths {
        let path = path.unwrap().path().display().to_string();
        if path.contains(file_name) {
            db_found = true;
        }
    }
    fs::remove_file(file_name).unwrap();

    assert!(db_found)
}

#[test]
fn check_adding_new_tx_method() {
    let file_name = "test_data_2.sqlite";
    if let Ok(metadata) = fs::metadata(file_name) {
        if metadata.is_file() {
            fs::remove_file(file_name).expect("Failed to delete existing file");
        }
    }

    let mut conn = Connection::open(file_name).unwrap();
    create_db(vec!["test1".to_string(), "test 2".to_string()], &mut conn).unwrap();

    let status = add_new_tx_methods(vec!["test3".to_string(), "test 4".to_string()], &mut conn);
    fs::remove_file(file_name).unwrap();

    assert_eq!(status, Ok(()))
}
