use chrono::NaiveDate;
use rex_app::conn::FetchNature;
use rex_app::modifier::parse_tx_fields;
use rex_db::ConnCache;
use std::fs;

use crate::common::{add_tx, create_test_db};

mod common;

#[test]
fn edit_income_amount_updates_balance() {
    let file_name = "test_edit_income_amount.sqlite";

    let mut db_conn = create_test_db(file_name);
    let cash_id = db_conn.cache().get_method_id("Cash").unwrap();
    let date = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();

    let old_tx = add_tx(
        &mut db_conn,
        "2024-06-01",
        "Paycheck",
        "Cash",
        "",
        "500.00",
        "Income",
        "Salary",
    );

    let new_tx = parse_tx_fields(
        "2024-06-01",
        "Paycheck",
        "Cash",
        "",
        "700.00",
        "Income",
        &db_conn,
    )
    .unwrap();

    db_conn.edit_tx(&old_tx, new_tx, "Salary").unwrap();

    let tx_view = db_conn
        .fetch_txs_with_date(date, FetchNature::Monthly)
        .unwrap();
    assert_eq!(tx_view.len(), 1);
    assert_eq!(tx_view.get_tx_balance(0)[&cash_id].value(), 70000);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn edit_expense_amount_updates_balance() {
    let file_name = "test_edit_expense_amount.sqlite";

    let mut db_conn = create_test_db(file_name);
    let cash_id = db_conn.cache().get_method_id("Cash").unwrap();
    let date = NaiveDate::from_ymd_opt(2024, 7, 1).unwrap();

    add_tx(
        &mut db_conn,
        "2024-07-01",
        "Income",
        "Cash",
        "",
        "1000.00",
        "Income",
        "Salary",
    );

    let old_tx = add_tx(
        &mut db_conn,
        "2024-07-15",
        "Rent",
        "Cash",
        "",
        "200.00",
        "Expense",
        "Rent",
    );

    let tx_view = db_conn
        .fetch_txs_with_date(date, FetchNature::Monthly)
        .unwrap();
    assert_eq!(tx_view.get_tx_balance(1)[&cash_id].value(), 80000);

    let new_tx = parse_tx_fields(
        "2024-07-15",
        "Rent",
        "Cash",
        "",
        "300.00",
        "Expense",
        &db_conn,
    )
    .unwrap();

    db_conn.edit_tx(&old_tx, new_tx, "Rent").unwrap();

    let tx_view = db_conn
        .fetch_txs_with_date(date, FetchNature::Monthly)
        .unwrap();
    assert_eq!(tx_view.len(), 2);
    // 1000 - 300 = 700
    assert_eq!(tx_view.get_tx_balance(1)[&cash_id].value(), 70000);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn edit_income_to_expense_reverses_balance() {
    let file_name = "test_edit_income_to_expense.sqlite";

    let mut db_conn = create_test_db(file_name);
    let cash_id = db_conn.cache().get_method_id("Cash").unwrap();
    let date = NaiveDate::from_ymd_opt(2024, 8, 1).unwrap();

    let old_tx = add_tx(
        &mut db_conn,
        "2024-08-01",
        "Mistake",
        "Cash",
        "",
        "500.00",
        "Income",
        "Unknown",
    );

    let new_tx = parse_tx_fields(
        "2024-08-01",
        "Mistake",
        "Cash",
        "",
        "500.00",
        "Expense",
        &db_conn,
    )
    .unwrap();

    db_conn.edit_tx(&old_tx, new_tx, "Unknown").unwrap();

    let tx_view = db_conn
        .fetch_txs_with_date(date, FetchNature::Monthly)
        .unwrap();
    assert_eq!(tx_view.len(), 1);
    // Changed from +500 to -500
    assert_eq!(tx_view.get_tx_balance(0)[&cash_id].value(), -50000);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn edit_change_method_moves_balance() {
    let file_name = "test_edit_change_method.sqlite";

    let mut db_conn = create_test_db(file_name);
    let cash_id = db_conn.cache().get_method_id("Cash").unwrap();
    let bank_id = db_conn.cache().get_method_id("Bank").unwrap();
    let date = NaiveDate::from_ymd_opt(2024, 9, 1).unwrap();

    // Add income to Cash
    add_tx(
        &mut db_conn,
        "2024-09-01",
        "Income",
        "Cash",
        "",
        "1000.00",
        "Income",
        "Salary",
    );

    let old_tx = add_tx(
        &mut db_conn,
        "2024-09-10",
        "Coffee",
        "Cash",
        "",
        "50.00",
        "Expense",
        "Coffee",
    );

    let tx_view = db_conn
        .fetch_txs_with_date(date, FetchNature::Monthly)
        .unwrap();
    assert_eq!(tx_view.get_tx_balance(1)[&cash_id].value(), 95000);

    let new_tx = parse_tx_fields(
        "2024-09-10",
        "Coffee",
        "Bank",
        "",
        "50.00",
        "Expense",
        &db_conn,
    )
    .unwrap();

    db_conn.edit_tx(&old_tx, new_tx, "Coffee").unwrap();

    let tx_view = db_conn
        .fetch_txs_with_date(date, FetchNature::Monthly)
        .unwrap();
    // Cash: 1000 (unchanged), Bank: -50
    assert_eq!(tx_view.get_tx_balance(1)[&cash_id].value(), 100000);
    assert_eq!(tx_view.get_tx_balance(1)[&bank_id].value(), -5000);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn edit_details_preserves_balance() {
    let file_name = "test_edit_details.sqlite";

    let mut db_conn = create_test_db(file_name);
    let cash_id = db_conn.cache().get_method_id("Cash").unwrap();
    let date = NaiveDate::from_ymd_opt(2024, 10, 1).unwrap();

    let old_tx = add_tx(
        &mut db_conn,
        "2024-10-01",
        "Old name",
        "Cash",
        "",
        "300.00",
        "Income",
        "Salary",
    );

    let new_tx = parse_tx_fields(
        "2024-10-01",
        "New name",
        "Cash",
        "",
        "300.00",
        "Income",
        &db_conn,
    )
    .unwrap();

    db_conn.edit_tx(&old_tx, new_tx, "Salary").unwrap();

    let tx_view = db_conn
        .fetch_txs_with_date(date, FetchNature::Monthly)
        .unwrap();
    assert_eq!(tx_view.len(), 1);
    assert_eq!(tx_view.get_tx(0).details, Some("New name".to_string()));
    assert_eq!(tx_view.get_tx_balance(0)[&cash_id].value(), 30000);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn edit_tags_changes_only_tags() {
    let file_name = "test_edit_tags.sqlite";

    let mut db_conn = create_test_db(file_name);
    let cash_id = db_conn.cache().get_method_id("Cash").unwrap();
    let date = NaiveDate::from_ymd_opt(2024, 11, 1).unwrap();

    let old_tx = add_tx(
        &mut db_conn,
        "2024-11-01",
        "Shopping",
        "Cash",
        "",
        "200.00",
        "Expense",
        "OldTag",
    );

    let new_tx = parse_tx_fields(
        "2024-11-01",
        "Shopping",
        "Cash",
        "",
        "200.00",
        "Expense",
        &db_conn,
    )
    .unwrap();

    db_conn
        .edit_tx(&old_tx, new_tx, "NewTag, ExtraTag")
        .unwrap();

    let tx_view = db_conn
        .fetch_txs_with_date(date, FetchNature::Monthly)
        .unwrap();
    assert_eq!(tx_view.len(), 1);
    let tags: Vec<&str> = tx_view
        .get_tx(0)
        .tags
        .iter()
        .map(|t| t.name.as_str())
        .collect();
    assert_eq!(tags, vec!["NewTag", "ExtraTag"]);
    assert_eq!(tx_view.get_tx_balance(0)[&cash_id].value(), -20000);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}
