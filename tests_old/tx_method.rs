extern crate rex_tui;
use rex_tui::db::*;
use rex_tui::utility::get_all_tx_methods;
use std::fs;

mod common;

use crate::common::create_test_db;

#[test]
fn check_getting_tx_methods_1() {
    let file_name = "getting_tx_methods_1.sqlite";
    let conn = create_test_db(file_name);
    let data = get_all_tx_methods(&conn);
    conn.close().unwrap();

    fs::remove_file(file_name).unwrap();

    assert_eq!(
        data,
        vec!["Super Special Bank".to_string(), "Cash Cow".to_string()]
    );
}

#[test]
fn check_getting_tx_methods_2() {
    let file_name = "getting_tx_methods_2.sqlite";
    let mut conn = create_test_db(file_name);

    add_new_tx_methods(
        &["new method 1".to_string(), "testing methods".to_string()],
        &mut conn,
    )
    .unwrap();

    let data = get_all_tx_methods(&conn);
    conn.close().unwrap();

    fs::remove_file(file_name).unwrap();

    assert_eq!(
        data,
        vec![
            "Super Special Bank".to_string(),
            "Cash Cow".to_string(),
            "new method 1".to_string(),
            "testing methods".to_string()
        ]
    );
}
