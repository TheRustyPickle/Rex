extern crate rex_tui;
use chrono::prelude::Local;
use rex_tui::db::create_db;
use rex_tui::outputs::{AType, CheckingError, NAType, TxType, VerifyingOutput};
use rex_tui::page_handler::{DateType, TxTab};
use rex_tui::tx_handler::TxData;
use rusqlite::Connection;
use std::fs;

fn create_test_db(file_name: &str) -> Connection {
    if let Ok(metadata) = fs::metadata(file_name) {
        if metadata.is_file() {
            fs::remove_file(file_name).expect("Failed to delete existing file");
        }
    }

    let mut conn = Connection::open(file_name).unwrap();
    create_db(vec!["test1".to_string(), "test 2".to_string()], &mut conn).unwrap();
    conn
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
    assert_eq!(tx_data.check_all_empty(), true);

    tx_data.add_tx_status("Some status".to_string());

    assert_eq!(tx_data.get_tx_status(), &vec!["Some status".to_string()]);

    let tx_data = TxData::custom(
        "2024-06-15",
        "details",
        "test1",
        "test 2",
        "100",
        "Transfer",
        "tags",
        0,
    );

    assert_eq!(tx_data.get_tx_type(), TxType::Transfer);
    assert_eq!(tx_data.get_tx_method(), "test1 to test 2".to_string());

    let mut tx_data = TxData::custom(
        "2023-07-19",
        "details",
        "test1",
        "",
        "100",
        "Expense",
        "tags",
        0,
    );

    assert_eq!(tx_data.get_tx_type(), TxType::IncomeExpense);
    assert_eq!(tx_data.get_tx_method(), "test1".to_string());
    assert!(tx_data.check_all_fields().is_none());
    assert_eq!(tx_data.check_all_empty(), false);

    let current_index = tx_data.get_current_index();

    tx_data.move_index_right(&TxTab::Date);
    tx_data.move_index_right(&TxTab::Date);

    assert_eq!(tx_data.get_current_index(), current_index + 2);

    tx_data.move_index_left(&TxTab::Date);

    assert_eq!(tx_data.get_current_index(), current_index + 1);

    let mut tx_data = TxData::custom("", "details", "test1", "", "100", "Expense", "tags", 0);
    assert_eq!(
        tx_data.check_all_fields().unwrap(),
        CheckingError::EmptyDate
    );

    let mut tx_data = TxData::custom(
        "2023-07-19",
        "details",
        "test1",
        "test1",
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
        "test1",
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
        "test1",
        "test1",
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
        "test1",
        "test1",
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
        "test1",
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
}

#[test]
fn test_tx_data_verifier() {
    let file_name = "tx_data_verifier.sqlite";
    let conn = create_test_db(&file_name);

    let mut tx_data = TxData::custom(
        "15-06-2023",
        "details",
        "test1",
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

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();
}
