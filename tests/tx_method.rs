extern crate rex;
use rex::db::*;
use rex::utility::get_all_tx_methods;
use rusqlite::Connection;
use std::fs;
//use std::collections::HashMap;

fn create_test_db(file_name: &str) -> Connection {
    create_db(file_name, vec!["test1".to_string(), "test 2".to_string()]).unwrap();
    Connection::open(file_name).unwrap()
}

#[test]
fn check_getting_tx_methods_1() {
    let file_name = "getting_tx_methods_1.sqlite";
    let conn = create_test_db(file_name);
    let data = get_all_tx_methods(&conn);
    conn.close().unwrap();

    fs::remove_file(file_name).unwrap();

    assert_eq!(data, vec!["test1".to_string(), "test 2".to_string()]);
}

#[test]
fn check_getting_tx_methods_2() {
    let file_name = "getting_tx_methods_2.sqlite";
    let conn = create_test_db(file_name);

    add_new_tx_methods(
        file_name,
        vec!["new method 1".to_string(), "testing methods".to_string()],
    )
    .unwrap();

    let data = get_all_tx_methods(&conn);
    conn.close().unwrap();

    fs::remove_file(file_name).unwrap();

    assert_eq!(
        data,
        vec![
            "test1".to_string(),
            "test 2".to_string(),
            "new method 1".to_string(),
            "testing methods".to_string()
        ]
    );
}
