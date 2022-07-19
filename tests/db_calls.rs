extern crate rex;
use rex::db::*;
use rusqlite::Connection;
use std::fs;

fn create_test_db(file_name: &str) -> Connection {

    create_db(file_name, vec!["test1".to_string(),
                        "test 2".to_string()]).unwrap();
    return Connection::open(file_name).unwrap();
}

#[test]
fn check_getting_tx_methods_1() {
    let file_name = "getting_tx_methods_1.sqlite";
    let conn = create_test_db(file_name);
    let data = get_all_tx_methods(&conn);
    fs::remove_file(file_name).unwrap();

    assert_eq!(data, vec!["test1".to_string(),
                "test 2".to_string()]);
}

#[test]
fn check_getting_tx_methods_2() {
    let file_name = "getting_tx_methods_2.sqlite";
    let conn = create_test_db(file_name);
    
    add_new_tx_methods(file_name, vec![
        "new method 1".to_string(),
        "testing methods".to_string()
    ]).unwrap();

    let data = get_all_tx_methods(&conn);
    fs::remove_file(file_name).unwrap();

    assert_eq!(data, vec!["test1".to_string(),
                "test 2".to_string(),
                "new method 1".to_string(),
            "testing methods".to_string()]);
}

#[test]
fn check_empty_changes() {
    let file_name = "empty_changes.sqlite";
    let conn = create_test_db(file_name);
    let data = get_empty_changes(&conn);
    fs::remove_file(file_name).unwrap();

    assert_eq!(data, vec![
        "Changes".to_string(),
        "0.00".to_string(),
        "0.00".to_string()
    ]);
}

#[test]
fn check_getting_all_changes_1() {
    let file_name = "getting_changes_1.sql";
    let conn = create_test_db(file_name);
    let data = get_all_changes(&conn, 5, 6);
    let empty_data: Vec<Vec<String>> = vec![vec![]];
    fs::remove_file(file_name).unwrap();

    assert_eq!(data, empty_data);
}

#[test]
fn check_getting_all_changes_2() {
    let file_name = "getting_changes_2.sql".to_string();
    let conn = create_test_db(&file_name);

    add_new_tx(
        "2022-07-19",
        "Testing transaction",
        "test1",
        "159.00",
        "Expense",
        file_name).unwrap();
    
    // This is the index of the interface. year 0 = 2022, month 0 = January
    let data = get_all_changes(&conn, 6, 0);
    let expected_data: Vec<Vec<String>> = vec![vec!["↓159.00".to_string(), "0.00".to_string()]];
    fs::remove_file("getting_changes_2.sql").unwrap();

    assert_eq!(data, expected_data);
}