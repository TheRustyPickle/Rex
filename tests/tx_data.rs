extern crate rex_tui;
use chrono::prelude::Local;
use rex_tui::outputs::{AType, CheckingError, NAType, TxType, VerifyingOutput};
use rex_tui::page_handler::{DateType, TxTab};
use rex_tui::tx_handler::{add_tx, TxData};
use rusqlite::Connection;
use std::fs;

mod common;

use crate::common::create_test_db;

fn add_dummy_tx(conn: &mut Connection) {
    add_tx(
        "2022-08-19",
        "Testing transaction",
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
        "Testing transaction",
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
        "Testing transaction",
        "Super Special Bank to Cash Cow",
        "200.00",
        "Transfer",
        "Food",
        None,
        conn,
    )
    .unwrap();
}

#[test]
fn test_tx_data_1() {
    let mut tx_data = TxData::new();

    let local_time = Local::now().to_string();
    let expected_data = vec![&local_time[0..10], "", "", "", "", "", "", ""];
    assert_eq!(tx_data.get_all_texts(), expected_data);

    tx_data.clear_date();

    let expected_data = vec!["", "", "", "", "", "", "", ""];
    assert_eq!(tx_data.get_all_texts(), expected_data);
    assert_eq!(tx_data.get_tx_status(), &Vec::<String>::new());
    assert!(tx_data.check_all_empty());

    tx_data.add_tx_status("Some status".to_string());

    assert_eq!(tx_data.get_tx_status(), &vec!["Some status".to_string()]);

    let tx_data = TxData::custom(
        "2024-06-15",
        "details",
        "Super Special Bank",
        "Cash Cow",
        "100",
        "Transfer",
        "tags",
        0,
    );

    assert_eq!(tx_data.get_tx_type(), TxType::Transfer);
    assert_eq!(
        tx_data.get_tx_method(),
        "Super Special Bank to Cash Cow".to_string()
    );

    let mut tx_data = TxData::custom(
        "2023-07-19",
        "details",
        "Super Special Bank",
        "",
        "100",
        "Expense",
        "tags",
        0,
    );

    assert_eq!(tx_data.get_tx_type(), TxType::IncomeExpense);
    assert_eq!(tx_data.get_tx_method(), "Super Special Bank".to_string());
    assert!(tx_data.check_all_fields().is_none());
    assert!(!tx_data.check_all_empty());

    let current_index = tx_data.get_current_index();

    tx_data.move_index_right(&TxTab::Date);
    tx_data.move_index_right(&TxTab::Date);

    assert_eq!(tx_data.get_current_index(), current_index + 2);

    tx_data.move_index_left(&TxTab::Date);

    assert_eq!(tx_data.get_current_index(), current_index + 1);

    tx_data.move_index_left(&TxTab::Date);
    tx_data.move_index_left(&TxTab::Date);
    tx_data.move_index_left(&TxTab::Date);
    tx_data.move_index_left(&TxTab::Date);

    assert_eq!(tx_data.get_current_index(), current_index);

    tx_data.go_current_index(&TxTab::Date);

    assert_eq!(tx_data.get_current_index(), 10);

    let mut tx_data = TxData::custom(
        "",
        "details",
        "Super Special Bank",
        "",
        "100",
        "Expense",
        "tags",
        0,
    );
    assert_eq!(
        tx_data.check_all_fields().unwrap(),
        CheckingError::EmptyDate
    );

    let mut tx_data = TxData::custom(
        "2023-07-19",
        "details",
        "Super Special Bank",
        "Super Special Bank",
        "100",
        "Transfer",
        "tags",
        0,
    );
    assert_eq!(
        tx_data.check_all_fields().unwrap(),
        CheckingError::SameTxMethod
    );

    let mut tx_data = TxData::custom(
        "2023-07-19",
        "details",
        "",
        "Super Special Bank",
        "100",
        "Expense",
        "tags",
        0,
    );
    assert_eq!(
        tx_data.check_all_fields().unwrap(),
        CheckingError::EmptyMethod
    );

    let mut tx_data = TxData::custom(
        "2023-07-19",
        "details",
        "Super Special Bank",
        "Super Special Bank",
        "",
        "Expense",
        "tags",
        0,
    );
    assert_eq!(
        tx_data.check_all_fields().unwrap(),
        CheckingError::EmptyAmount
    );

    let mut tx_data = TxData::custom(
        "2023-07-19",
        "details",
        "Super Special Bank",
        "Super Special Bank",
        "100",
        "",
        "tags",
        0,
    );
    assert_eq!(
        tx_data.check_all_fields().unwrap(),
        CheckingError::EmptyTxType
    );

    let mut tx_data = TxData::custom(
        "2023-07-19",
        "details",
        "Super Special Bank",
        "",
        "100",
        "Transfer",
        "tags",
        0,
    );
    assert_eq!(
        tx_data.check_all_fields().unwrap(),
        CheckingError::EmptyMethod
    );

    let mut tx_data = TxData::custom(
        "2023-07-19",
        "details",
        "Super Special Bank",
        "Cash Cow",
        "100",
        "Transfer",
        "",
        0,
    );
    assert!(tx_data.check_all_fields().is_none());
}

#[test]
fn test_tx_data_verifier() {
    let file_name = "tx_data_verifier.sqlite";
    let conn = create_test_db(file_name);

    let mut tx_data = TxData::custom(
        "15-06-2023",
        "details",
        "Super Special Bank",
        "Nope",
        "b+100",
        "Transfer",
        "tags, asdf",
        0,
    );

    let date_status = tx_data.check_date(&DateType::Exact);
    let from_method_verifier = tx_data.check_from_method(&conn);
    let to_method_verifier = tx_data.check_to_method(&conn);
    let tx_type_status = tx_data.check_tx_type();
    let tags_status = tx_data.check_tags_forced(&conn);
    let amount_status = tx_data.check_amount(false, &conn);

    assert_eq!(date_status, VerifyingOutput::Accepted(AType::Date));
    assert_eq!(
        from_method_verifier,
        VerifyingOutput::Accepted(AType::TxMethod)
    );
    assert_eq!(
        to_method_verifier,
        VerifyingOutput::NotAccepted(NAType::InvalidTxMethod)
    );
    assert_eq!(tx_type_status, VerifyingOutput::Accepted(AType::TxType));
    assert_eq!(
        tags_status,
        VerifyingOutput::NotAccepted(NAType::NonExistingTag)
    );
    assert_eq!(amount_status, VerifyingOutput::Accepted(AType::Amount));

    let mut tx_data = TxData::custom(
        "15-06-2023",
        "details",
        "Super Special Bank",
        "Nope",
        "b+100",
        "Transfer",
        "tags, tags",
        0,
    );

    tx_data.check_tags();

    assert_eq!(tx_data.get_all_texts()[6], "tags");

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();
}

#[test]
fn test_tx_data_stepper() {
    let file_name = "tx_data_stepper.sqlite";
    let conn = create_test_db(file_name);

    let mut tx_data = TxData::custom(
        "15-06-2023",
        "details",
        "Super Special Bank",
        "Nope",
        "b+100",
        "Transfer",
        "tags, asdf",
        0,
    );

    let date_step_up = tx_data.do_date_up(&DateType::Exact);
    let date_step_down = tx_data.do_date_down(&DateType::Exact);

    let amount_step_up = tx_data.do_amount_up(false, &conn);
    let amount_step_down = tx_data.do_amount_down(false, &conn);

    let tx_method_up = tx_data.do_from_method_up(&conn);
    let tx_method_down = tx_data.do_from_method_down(&conn);

    let tx_to_up = tx_data.do_to_method_up(&conn);
    let tx_to_down = tx_data.do_to_method_down(&conn);

    let tx_type_up = tx_data.do_tx_type_up();
    let tx_type_down = tx_data.do_tx_type_down();

    let tags_up = tx_data.do_tags_up(&conn);
    let tags_down = tx_data.do_tags_down(&conn);

    assert!(date_step_up.is_ok());
    assert!(date_step_down.is_ok());
    assert!(amount_step_up.is_ok());
    assert!(amount_step_down.is_ok());
    assert!(tx_method_up.is_ok());
    assert!(tx_method_down.is_ok());
    assert!(tx_to_up.is_err());
    assert!(tx_to_down.is_ok());
    assert!(tx_type_up.is_ok());
    assert!(tx_type_down.is_ok());
    assert!(tags_up.is_err());
    assert!(tags_down.is_ok());

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();
}

#[test]
fn tx_data_searching() {
    let file_name = "tx_data_searching_test.sqlite";
    let mut conn = create_test_db(file_name);
    let tx_data = TxData::new_empty();

    let data = tx_data.get_search_tx(&DateType::Exact, &conn);
    assert_eq!(data, (Vec::new(), Vec::new()));

    add_dummy_tx(&mut conn);

    assert!(tx_data.check_all_empty());

    let tx_data = TxData::custom("19-07-2023", "", "", "", "", "", "", 0);

    let data = tx_data.get_search_tx(&DateType::Exact, &conn);
    assert_eq!(data.0.len(), 1);

    let tx_data = TxData::custom("19-07-2023", "", "", "", "", "", "", 0);

    let data = tx_data.get_search_tx(&DateType::Monthly, &conn);
    assert_eq!(data.0.len(), 2);

    let mut tx_data = TxData::new_empty();

    for i in ['2', '0', '2', '3'] {
        tx_data.edit_date(Some(i));
    }

    let data = tx_data.get_search_tx(&DateType::Yearly, &conn);
    assert_eq!(data.0.len(), 2);

    let tx_data = TxData::custom("", "Testing transaction", "", "", "", "", "", 0);

    let data = tx_data.get_search_tx(&DateType::Exact, &conn);
    assert_eq!(data.0.len(), 3);

    let tx_data = TxData::custom("", "", "Super Special Bank", "", "", "", "", 0);

    let data = tx_data.get_search_tx(&DateType::Exact, &conn);
    assert_eq!(data.0.len(), 1);

    let tx_data = TxData::custom(
        "",
        "",
        "Super Special Bank",
        "Cash Cow",
        "",
        "Transfer",
        "",
        0,
    );

    let data = tx_data.get_search_tx(&DateType::Exact, &conn);
    assert_eq!(data.0.len(), 1);

    let tx_data = TxData::custom("", "", "", "", "100.00", "", "", 0);

    let data = tx_data.get_search_tx(&DateType::Exact, &conn);
    assert_eq!(data.0.len(), 2);

    let tx_data = TxData::custom("", "", "", "", ">100.00", "", "", 0);

    let data = tx_data.get_search_tx(&DateType::Exact, &conn);
    assert_eq!(data.0.len(), 1);

    let tx_data = TxData::custom("", "", "", "", "<200.00", "", "", 0);

    let data = tx_data.get_search_tx(&DateType::Exact, &conn);
    assert_eq!(data.0.len(), 2);

    let tx_data = TxData::custom("", "", "", "", ">=100.00", "", "", 0);

    let data = tx_data.get_search_tx(&DateType::Exact, &conn);
    assert_eq!(data.0.len(), 3);

    let tx_data = TxData::custom("", "", "", "", "<=100.00", "", "", 0);

    let data = tx_data.get_search_tx(&DateType::Exact, &conn);
    assert_eq!(data.0.len(), 2);

    let tx_data = TxData::custom("", "", "", "", "", "Transfer", "", 0);

    let data = tx_data.get_search_tx(&DateType::Exact, &conn);
    assert_eq!(data.0.len(), 1);

    let tx_data = TxData::custom("", "", "", "", "", "", "Food, Car", 0);

    let data = tx_data.get_search_tx(&DateType::Exact, &conn);
    assert_eq!(data.0.len(), 3);

    let tx_data = TxData::custom(
        "19-07-2023",
        "Testing transaction",
        "Cash Cow",
        "",
        "100.00",
        "Expense",
        "Food",
        0,
    );

    let data = tx_data.get_search_tx(&DateType::Exact, &conn);
    assert_eq!(data.0.len(), 1);

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();
}

#[test]
fn tx_data_editing() {
    let mut tx_data = TxData::new_empty();

    for i in ['2', '0', '2', '3', '-', '0', '7', '-', '0', '5'] {
        tx_data.edit_date(Some(i));
    }

    tx_data.go_current_index(&TxTab::Details);

    for i in ['S', 'o', 'm', 'e'] {
        tx_data.edit_details(Some(i));
    }

    tx_data.go_current_index(&TxTab::FromMethod);

    for i in [
        'S', 'u', 'p', 'e', 'r', ' ', 'S', 'p', 'e', 'c', 'i', 'a', 'l', ' ', 'B', 'a', 'n', 'k',
    ] {
        tx_data.edit_from_method(Some(i));
    }

    tx_data.go_current_index(&TxTab::Amount);

    for i in ['2', '0', '2', '3'] {
        tx_data.edit_amount(Some(i));
    }

    tx_data.go_current_index(&TxTab::TxType);

    let i = 'E';
    tx_data.edit_tx_type(Some(i));

    tx_data.go_current_index(&TxTab::ToMethod);

    for i in ['C', 'a', 's', 'h', ' ', 'C', 'o', 'w'] {
        tx_data.edit_to_method(Some(i));
    }

    tx_data.go_current_index(&TxTab::Tags);

    for i in ['T', 'a', 'g'] {
        tx_data.edit_tags(Some(i));
    }

    let expected_data: Vec<String> = vec![
        "2023-07-05",
        "Some",
        "Super Special Bank",
        "Cash Cow",
        "2023",
        "E",
        "Tag",
        "",
    ]
    .into_iter()
    .map(ToString::to_string)
    .collect();

    assert_eq!(tx_data.get_all_texts(), expected_data);

    tx_data.go_current_index(&TxTab::Date);

    for _ in ['2', '0', '2', '3', '-', '0', '7', '-', '0', '5'] {
        tx_data.edit_date(None);
    }

    tx_data.go_current_index(&TxTab::Details);

    for _ in ['S', 'o', 'm', 'e'] {
        tx_data.edit_details(None);
    }

    tx_data.go_current_index(&TxTab::FromMethod);

    for _ in [
        'S', 'u', 'p', 'e', 'r', ' ', 'S', 'p', 'e', 'c', 'i', 'a', 'l', ' ', 'B', 'a', 'n', 'k',
    ] {
        tx_data.edit_from_method(None);
    }

    tx_data.go_current_index(&TxTab::Amount);

    for _ in ['2', '0', '2', '3'] {
        tx_data.edit_amount(None);
    }

    tx_data.go_current_index(&TxTab::TxType);

    let _ = 'E';
    tx_data.edit_tx_type(None);

    tx_data.go_current_index(&TxTab::ToMethod);

    for _ in ['C', 'a', 's', 'h', ' ', 'C', 'o', 'w'] {
        tx_data.edit_to_method(None);
    }

    tx_data.go_current_index(&TxTab::Tags);

    for _ in ['T', 'a', 'g'] {
        tx_data.edit_tags(None);
    }

    let expected_data: Vec<String> = vec!["", "", "", "", "", "", "", ""]
        .into_iter()
        .map(ToString::to_string)
        .collect();

    assert_eq!(tx_data.get_all_texts(), expected_data);
}

#[test]
fn tx_data_suffix() {
    let file_name = "tx_data_suffix.sqlite";
    let conn = create_test_db(file_name);

    let mut tx_data = TxData::new_empty();

    tx_data.amount = String::from("1k");
    tx_data.check_amount(false, &conn);

    assert_eq!(tx_data.amount, String::from("1000.00"));

    tx_data.amount = String::from("1m");
    tx_data.check_amount(false, &conn);

    assert_eq!(tx_data.amount, String::from("1000000.00"));

    tx_data.amount = String::from("1km");
    tx_data.check_amount(false, &conn);

    assert_eq!(tx_data.amount, String::from("1000000000.00"));

    tx_data.amount = String::from("1k1");
    tx_data.check_amount(false, &conn);

    assert_eq!(tx_data.amount, String::from("11.00"));

    tx_data.amount = String::from("5m12");
    tx_data.check_amount(false, &conn);

    assert_eq!(tx_data.amount, String::from("512.00"));

    tx_data.amount = String::from("100 + 5k");
    tx_data.check_amount(false, &conn);

    assert_eq!(tx_data.amount, String::from("5100.00"));

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();
}
