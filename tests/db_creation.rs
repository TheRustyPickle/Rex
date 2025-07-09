extern crate rex_tui;
use rex_tui::db::{add_new_tx_methods, rename_column, reposition_column};
use rex_tui::tx_handler::add_tx;
use rex_tui::utility::{get_all_tx_methods, get_last_balances};
use std::fs;

mod common;

use crate::common::create_test_db;

#[test]
fn check_db_creation() {
    let file_name = "test_db_1.sqlite";
    let conn = create_test_db(file_name);

    let paths = fs::read_dir(".").unwrap();
    let mut db_found = false;
    for path in paths {
        let path = path.unwrap().path().display().to_string();
        if path.contains(file_name) {
            db_found = true;
        }
    }
    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();

    assert!(db_found);
}

#[test]
fn check_adding_new_tx_method() {
    let file_name = "test_db_2.sqlite";
    let mut conn = create_test_db(file_name);

    let status = add_new_tx_methods(&["test3".to_string(), "test 4".to_string()], &mut conn);

    let tx_methods = get_all_tx_methods(&conn);
    let expected_tx_methods = vec![
        "Super Special Bank".to_string(),
        "Cash Cow".to_string(),
        "test3".to_string(),
        "test 4".to_string(),
    ];
    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();

    assert_eq!(status, Ok(()));
    assert_eq!(expected_tx_methods, tx_methods);
}

#[test]
fn check_renaming_columns() {
    let file_name = "test_db_3.sqlite";
    let mut conn = create_test_db(file_name);

    let status = rename_column("Cash Cow", "testing", &mut conn);
    let tx_methods = get_all_tx_methods(&conn);
    let expected_tx_methods = vec!["Super Special Bank".to_string(), "testing".to_string()];

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();

    assert_eq!(status, Ok(()));
    assert_eq!(expected_tx_methods, tx_methods);
}

#[test]
fn check_repositioning_columns() {
    let file_name = "test_db_4.sqlite";
    let mut conn = create_test_db(file_name);

    add_tx(
        "2022-07-19",
        "Testing transaction",
        "Super Special Bank",
        "159.00",
        "Income",
        "Unknown",
        None,
        &mut conn,
    )
    .unwrap();

    let old_last_balances = get_last_balances(&conn);

    let status = reposition_column(
        &["Cash Cow".to_string(), "Super Special Bank".to_string()],
        &mut conn,
    );
    let tx_methods = get_all_tx_methods(&conn);
    let expected_tx_methods = vec!["Cash Cow".to_string(), "Super Special Bank".to_string()];

    let last_balances = get_last_balances(&conn);

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();

    assert_eq!(status, Ok(()));
    assert_eq!(expected_tx_methods, tx_methods);
    assert_eq!(old_last_balances, vec!["159", "0"]);
    assert_eq!(last_balances, vec!["0", "159"]);
}
