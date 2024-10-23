extern crate rex_tui;
use rex_tui::db::create_db;
use rex_tui::tx_handler::*;
use rex_tui::utility::*;
use rusqlite::Connection;
use std::fs;

fn create_test_db(file_name: &str) -> Connection {
    if let Ok(metadata) = fs::metadata(file_name) {
        if metadata.is_file() {
            fs::remove_file(file_name).expect("Failed to delete existing file");
        }
    }

    let mut conn = Connection::open(file_name).unwrap();
    create_db(&["test1".to_string(), "test 2".to_string()], &mut conn).unwrap();
    conn
}

#[test]
fn check_empty_changes() {
    let file_name = "empty_changes.sqlite";
    let conn = create_test_db(file_name);
    let data = get_empty_changes(&conn);
    conn.close().unwrap();

    fs::remove_file(file_name).unwrap();

    assert_eq!(
        data,
        vec![
            "Changes".to_string(),
            "0.00".to_string(),
            "0.00".to_string()
        ]
    );
}

#[test]
fn check_getting_all_changes() {
    let file_name = "getting_changes_1.sqlite";
    let conn = create_test_db(file_name);
    let data = get_all_changes(5, 6, &conn);
    let empty_data: Vec<Vec<String>> = Vec::new();

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();

    assert_eq!(data, empty_data);
}

#[test]
fn check_getting_all_changes_2() {
    let file_name = "getting_changes_2.sqlite";
    let mut conn = create_test_db(file_name);

    add_tx(
        "2022-07-19",
        "Testing transaction",
        "test1",
        "159.00",
        "Expense",
        "Unknown",
        None,
        &mut conn,
    )
    .unwrap();

    add_tx(
        "2022-07-19",
        "Testing transaction",
        "test 2",
        "159.00",
        "Expense",
        "Unknown",
        None,
        &mut conn,
    )
    .unwrap();

    add_tx(
        "2022-05-01",
        "Testing transaction",
        "test 2",
        "753.00",
        "Expense",
        "Unknown",
        None,
        &mut conn,
    )
    .unwrap();

    // This is the index of the interface. year 0 = 2022, month 0 = January
    let data_1 = get_all_changes(6, 0, &conn);
    let expected_data_1: Vec<Vec<String>> = vec![
        vec!["↓159.00".to_string(), "0.00".to_string()],
        vec!["0.00".to_string(), "↓159.00".to_string()],
    ];

    let another_data = get_all_changes(4, 0, &conn);

    let another_expected = vec![vec!["0.00".to_string(), "↓753.00".to_string()]];

    delete_tx(2, &mut conn).unwrap();

    let data_2 = get_all_changes(6, 0, &conn);
    let expected_data_2: Vec<Vec<String>> = vec![vec!["↓159.00".to_string(), "0.00".to_string()]];

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();

    assert_eq!(data_1, expected_data_1);
    assert_eq!(data_2, expected_data_2);
    assert_eq!(another_data, another_expected);
}
