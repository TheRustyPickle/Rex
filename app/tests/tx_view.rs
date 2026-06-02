use chrono::NaiveDate;
use rex_app::conn::FetchNature;
use rex_app::views::PartialTx;
use std::fs;

use crate::common::{add_tx, create_test_db};

mod common;

fn get_col(rows: &[Vec<String>], row: usize, col: usize) -> &str {
    &rows[row][col]
}

#[test]
fn balance_array_with_index_shows_running_balance() {
    let file_name = "test_balance_array.sqlite";
    let mut db_conn = create_test_db(file_name);

    add_tx(
        &mut db_conn,
        "2024-06-01",
        "Income",
        "Cash",
        "",
        "1000.00",
        "Income",
        "Salary",
    );
    add_tx(
        &mut db_conn,
        "2024-06-15",
        "Expense",
        "Cash",
        "",
        "200.00",
        "Expense",
        "Food",
    );

    let date = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
    let tx_view = db_conn
        .fetch_txs_with_date(date, FetchNature::Monthly)
        .unwrap();

    // At index 0: running balance after first tx (Income +1000)
    let arr = tx_view.balance_array(Some(0), &mut db_conn).unwrap();
    // Header row: ["", "Cash", "Bank", "Other", "Total"]
    assert_eq!(get_col(&arr, 0, 1), "Cash");
    assert_eq!(get_col(&arr, 0, 4), "Total");
    // Balance row: ["Balance", "1000.00", "0.00", "0.00", "1000.00"]
    assert_eq!(get_col(&arr, 1, 0), "Balance");
    assert_eq!(get_col(&arr, 1, 1), "1000.00"); // Cash
    assert_eq!(get_col(&arr, 1, 4), "1000.00"); // Total
    // Changes row for this tx: "↑1000.00"
    assert_eq!(get_col(&arr, 2, 0), "Changes");
    assert!(get_col(&arr, 2, 1).contains("1000.00"));
    // Income row: cumulative income up to this tx
    assert_eq!(get_col(&arr, 3, 0), "Income");
    assert_eq!(get_col(&arr, 3, 1), "1000.00");
    // Expense row: cumulative expense up to this tx
    assert_eq!(get_col(&arr, 4, 0), "Expense");
    assert_eq!(get_col(&arr, 4, 1), "0.00");

    // At index 1: after Expense -200, running balance = 800
    let arr = tx_view.balance_array(Some(1), &mut db_conn).unwrap();
    assert_eq!(get_col(&arr, 1, 1), "800.00"); // Cash balance
    assert_eq!(get_col(&arr, 1, 4), "800.00"); // Total

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn balance_array_with_none_shows_final_balance() {
    let file_name = "test_balance_array_none.sqlite";
    let mut db_conn = create_test_db(file_name);

    add_tx(
        &mut db_conn,
        "2024-07-01",
        "Salary",
        "Cash",
        "",
        "500.00",
        "Income",
        "Work",
    );
    add_tx(
        &mut db_conn,
        "2024-07-01",
        "Transfer",
        "Cash",
        "Bank",
        "200.00",
        "Transfer",
        "Move",
    );

    let date = NaiveDate::from_ymd_opt(2024, 7, 1).unwrap();
    let tx_view = db_conn
        .fetch_txs_with_date(date, FetchNature::Monthly)
        .unwrap();

    // None → final balance (from Balance::get_final_balance)
    let arr = tx_view.balance_array(None, &mut db_conn).unwrap();
    // Cash: 500 - 200 = 300, Bank: 200
    assert_eq!(get_col(&arr, 1, 1), "300.00"); // Cash
    assert_eq!(get_col(&arr, 1, 2), "200.00"); // Bank
    assert_eq!(get_col(&arr, 1, 4), "500.00"); // Total

    // Changes should all be 0.00 (no specific tx)
    assert_eq!(get_col(&arr, 2, 1), "0.00");

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn balance_array_daily_income_expense() {
    let file_name = "test_balance_daily.sqlite";
    let mut db_conn = create_test_db(file_name);

    // Same date: two incomes
    add_tx(
        &mut db_conn,
        "2024-08-01",
        "Morning",
        "Cash",
        "",
        "100.00",
        "Income",
        "A",
    );
    add_tx(
        &mut db_conn,
        "2024-08-01",
        "Afternoon",
        "Cash",
        "",
        "50.00",
        "Income",
        "B",
    );
    add_tx(
        &mut db_conn,
        "2024-08-02",
        "Next day",
        "Cash",
        "",
        "200.00",
        "Income",
        "C",
    );

    let date = NaiveDate::from_ymd_opt(2024, 8, 1).unwrap();
    let tx_view = db_conn
        .fetch_txs_with_date(date, FetchNature::Monthly)
        .unwrap();

    // At index 1 (second tx of Aug 1): daily income = 100+50 = 150
    let arr = tx_view.balance_array(Some(1), &mut db_conn).unwrap();
    assert_eq!(get_col(&arr, 5, 0), "Daily Income");
    assert_eq!(get_col(&arr, 5, 1), "150.00"); // 100+50 from same date

    // At index 2 (Aug 2): daily income = only the Aug 2 tx = 200
    let arr = tx_view.balance_array(Some(2), &mut db_conn).unwrap();
    assert_eq!(get_col(&arr, 5, 1), "200.00"); // only the Aug 2 tx

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn add_tx_balance_array_projecting_new_tx() {
    let file_name = "test_add_balance_array.sqlite";
    let mut db_conn = create_test_db(file_name);

    add_tx(
        &mut db_conn,
        "2024-09-01",
        "Existing",
        "Cash",
        "",
        "500.00",
        "Income",
        "Salary",
    );

    let date = NaiveDate::from_ymd_opt(2024, 9, 1).unwrap();
    let tx_view = db_conn
        .fetch_txs_with_date(date, FetchNature::Monthly)
        .unwrap();

    // Project what adding a new expense of $100 would look like at index 1
    let partial = Some(PartialTx {
        from_method: "Cash",
        to_method: "",
        tx_type: "Expense",
        amount: "100.00",
    });
    let arr = tx_view
        .add_tx_balance_array(Some(1), partial, &mut db_conn)
        .unwrap();

    // Balance row: existing 500 - projected 100 = 400
    assert_eq!(get_col(&arr, 1, 0), "Balance");
    assert_eq!(get_col(&arr, 1, 1), "400.00"); // Cash

    // Changes row: should show projected change
    assert_eq!(get_col(&arr, 2, 0), "Changes");
    assert!(get_col(&arr, 2, 1).contains("100.00"));

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn add_tx_balance_array_projection_transfer() {
    let file_name = "test_add_balance_transfer.sqlite";
    let mut db_conn = create_test_db(file_name);

    add_tx(
        &mut db_conn,
        "2024-10-01",
        "Income",
        "Cash",
        "",
        "1000.00",
        "Income",
        "Salary",
    );

    let date = NaiveDate::from_ymd_opt(2024, 10, 1).unwrap();
    let tx_view = db_conn
        .fetch_txs_with_date(date, FetchNature::Monthly)
        .unwrap();

    // Project adding a transfer of $300 from Cash to Bank
    let partial = Some(PartialTx {
        from_method: "Cash",
        to_method: "Bank",
        tx_type: "Transfer",
        amount: "300.00",
    });
    let arr = tx_view
        .add_tx_balance_array(Some(1), partial, &mut db_conn)
        .unwrap();

    // Cash: 1000 - 300 = 700, Bank: 0 + 300 = 300
    assert_eq!(get_col(&arr, 1, 1), "700.00"); // Cash
    assert_eq!(get_col(&arr, 1, 2), "300.00"); // Bank

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}
