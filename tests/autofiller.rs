extern crate rex_tui;
use rex_tui::page_handler::TxTab;
use rex_tui::tx_handler::*;
use rex_tui::utility::traits::AutoFiller;
use rusqlite::Connection;
use std::fs;

use crate::common::create_test_db;

mod common;

struct Testing {
    data: Vec<String>,
    expected: Vec<String>,
}
impl AutoFiller for Testing {}

fn add_dummy_tx(conn: &mut Connection) {
    add_tx(
        "2022-08-19",
        "Car expense",
        "Super Special Bank",
        "100.00",
        "Expense",
        "Car",
        None,
        conn,
    )
    .unwrap();

    add_tx(
        "2023-07-19",
        "Food cost",
        "Cash Cow",
        "100.00",
        "Expense",
        "Food",
        None,
        conn,
    )
    .unwrap();

    add_tx(
        "2023-07-25",
        "Selling goods",
        "Super Special Bank",
        "200.00",
        "Income",
        "Goods",
        None,
        conn,
    )
    .unwrap();
}

#[test]
fn autofiller_test() {
    let file_name = "autofiller_test.sqlite";
    let mut conn = create_test_db(file_name);

    let data = vec![
        "car",
        "expense",
        "sll",
        "f o o d",
        "Selling goods",
        "coast",
        "r ex",
        "",
    ]
    .into_iter()
    .map(std::string::ToString::to_string)
    .collect::<Vec<String>>();
    let expected = vec![
        "Car expense",
        "Car expense",
        "Selling goods",
        "Food cost",
        "",
        "Food cost",
        "Car expense",
        "",
    ]
    .into_iter()
    .map(std::string::ToString::to_string)
    .collect::<Vec<String>>();

    let test_data = Testing { data, expected };

    // Nothing to take suggestion from so empty string as autofiller
    for i in 0..test_data.data.len() {
        let result = test_data.autofill_details(&test_data.data[i], &conn);
        assert_eq!(result, String::new());
    }

    add_dummy_tx(&mut conn);

    for i in 0..test_data.data.len() {
        let result = test_data.autofill_details(&test_data.data[i], &conn);
        assert_eq!(result, test_data.expected[i]);
    }

    let data = vec![
        "sup", "cash", "CoW", "Cis", "sup", "bank", "CaSh CoW", "Cash Cow", "",
    ]
    .into_iter()
    .map(std::string::ToString::to_string)
    .collect::<Vec<String>>();
    let expected = vec![
        "Super Special Bank",
        "Cash Cow",
        "Cash Cow",
        "Cash Cow",
        "Super Special Bank",
        "Super Special Bank",
        "Cash Cow",
        "",
        "",
    ]
    .into_iter()
    .map(std::string::ToString::to_string)
    .collect::<Vec<String>>();

    let test_data = Testing { data, expected };

    for i in 0..test_data.data.len() {
        let result = test_data.autofill_tx_method(&test_data.data[i], &conn);
        assert_eq!(result, test_data.expected[i]);
    }

    let data = vec![
        "foo", "goo", "fod", "gid", "rac", "Car", "", "Food,", "Food, Go",
    ]
    .into_iter()
    .map(std::string::ToString::to_string)
    .collect::<Vec<String>>();
    let expected = vec!["Food", "Goods", "Food", "Goods", "Car", "", "", "", "Goods"]
        .into_iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<String>>();

    let test_data = Testing { data, expected };

    for i in 0..test_data.data.len() {
        let result = test_data.autofill_tags(&test_data.data[i], &conn);
        assert_eq!(result, test_data.expected[i]);
    }

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();
}

#[test]
fn tx_data_autofiller() {
    let file_name = "autofiller_test_2.sqlite";
    let mut conn = create_test_db(file_name);
    add_dummy_tx(&mut conn);

    let mut tx_data = TxData::custom("", "Food", "Super", "Cash", "", "", "Car, fo", 1);

    tx_data.check_autofill(&TxTab::Details, &conn);
    assert_eq!(tx_data.autofill, "Food cost");
    tx_data.accept_autofill(&TxTab::Details);
    assert_eq!(tx_data.details, "Food cost");

    tx_data.check_autofill(&TxTab::FromMethod, &conn);
    assert_eq!(tx_data.autofill, "Super Special Bank");
    tx_data.accept_autofill(&TxTab::FromMethod);
    assert_eq!(tx_data.from_method, "Super Special Bank");

    tx_data.check_autofill(&TxTab::ToMethod, &conn);
    assert_eq!(tx_data.autofill, "Cash Cow");
    tx_data.accept_autofill(&TxTab::ToMethod);
    assert_eq!(tx_data.to_method, "Cash Cow");

    tx_data.check_autofill(&TxTab::Tags, &conn);
    assert_eq!(tx_data.autofill, "Food");
    tx_data.accept_autofill(&TxTab::Tags);
    assert_eq!(tx_data.tags, "Car, Food");

    tx_data.check_autofill(&TxTab::Amount, &conn);
    assert_eq!(tx_data.autofill, "");
    tx_data.accept_autofill(&TxTab::Details);
    assert_eq!(tx_data.amount, "");

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();
}
