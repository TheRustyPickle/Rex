use chrono::NaiveDate;
use rex_app::conn::FetchNature;
use rex_db::ConnCache;
use std::fs;

use crate::common::{add_tx, create_test_db};

mod common;

#[test]
fn delete_income_reverses_balance() {
    let file_name = "test_delete_income.sqlite";

    let mut db_conn = create_test_db(file_name);
    let cash_id = db_conn.cache().get_method_id("Cash").unwrap();

    let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();

    let tx = add_tx(
        &mut db_conn,
        "2024-06-15",
        "Paycheck",
        "Cash",
        "",
        "500.00",
        "Income",
        "Salary",
    );

    let tx_view = db_conn
        .fetch_txs_with_date(date, FetchNature::Monthly)
        .unwrap();
    assert_eq!(tx_view.len(), 1);
    assert_eq!(tx_view.get_tx_balance(0)[&cash_id].value(), 50000);

    db_conn.delete_tx(&tx).unwrap();

    let tx_view = db_conn
        .fetch_txs_with_date(date, FetchNature::Monthly)
        .unwrap();
    assert_eq!(tx_view.len(), 0);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn delete_expense_reverses_balance() {
    let file_name = "test_delete_expense.sqlite";

    let mut db_conn = create_test_db(file_name);
    let cash_id = db_conn.cache().get_method_id("Cash").unwrap();

    // First add income so we have a balance to spend from
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

    // Add an expense of $200
    let tx = add_tx(
        &mut db_conn,
        "2024-07-15",
        "Rent",
        "Cash",
        "",
        "200.00",
        "Expense",
        "Rent",
    );

    let date = NaiveDate::from_ymd_opt(2024, 7, 1).unwrap();
    let tx_view = db_conn
        .fetch_txs_with_date(date, FetchNature::Monthly)
        .unwrap();
    assert_eq!(tx_view.len(), 2);
    // Balance after both txs: 1000 - 200 = 800
    assert_eq!(tx_view.get_tx_balance(1)[&cash_id].value(), 80000);

    // Delete the expense
    db_conn.delete_tx(&tx).unwrap();

    let tx_view = db_conn
        .fetch_txs_with_date(date, FetchNature::Monthly)
        .unwrap();
    assert_eq!(tx_view.len(), 1, "Only the income should remain");
    // Balance should revert to 1000
    assert_eq!(tx_view.get_tx_balance(0)[&cash_id].value(), 100000);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn delete_transfer_reverses_both_methods() {
    let file_name = "test_delete_transfer.sqlite";

    let mut db_conn = create_test_db(file_name);
    let cash_id = db_conn.cache().get_method_id("Cash").unwrap();
    let bank_id = db_conn.cache().get_method_id("Bank").unwrap();

    // Add income to Cash first
    add_tx(
        &mut db_conn,
        "2024-08-01",
        "Income",
        "Cash",
        "",
        "1000.00",
        "Income",
        "Salary",
    );

    let tx = add_tx(
        &mut db_conn,
        "2024-08-10",
        "Move to bank",
        "Cash",
        "Bank",
        "300.00",
        "Transfer",
        "Transfer",
    );

    let date = NaiveDate::from_ymd_opt(2024, 8, 1).unwrap();
    let tx_view = db_conn
        .fetch_txs_with_date(date, FetchNature::Monthly)
        .unwrap();
    assert_eq!(tx_view.len(), 2);
    // Cash: 1000 - 300 = 700, Bank: 0 + 300 = 300
    assert_eq!(tx_view.get_tx_balance(1)[&cash_id].value(), 70000);
    assert_eq!(tx_view.get_tx_balance(1)[&bank_id].value(), 30000);

    db_conn.delete_tx(&tx).unwrap();

    let tx_view = db_conn
        .fetch_txs_with_date(date, FetchNature::Monthly)
        .unwrap();
    assert_eq!(tx_view.len(), 1);
    // Cash back to 1000, Bank back to 0
    assert_eq!(tx_view.get_tx_balance(0)[&cash_id].value(), 100000);
    assert_eq!(tx_view.get_tx_balance(0)[&bank_id].value(), 0);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn delete_first_of_multiple_txs_maintains_other_balances() {
    let file_name = "test_delete_first_tx.sqlite";

    let mut db_conn = create_test_db(file_name);
    let cash_id = db_conn.cache().get_method_id("Cash").unwrap();

    let tx1 = add_tx(
        &mut db_conn,
        "2024-09-01",
        "Salary",
        "Cash",
        "",
        "500.00",
        "Income",
        "Salary",
    );
    add_tx(
        &mut db_conn,
        "2024-09-15",
        "Groceries",
        "Cash",
        "",
        "100.00",
        "Expense",
        "Groceries",
    );
    add_tx(
        &mut db_conn,
        "2024-09-20",
        "Coffee",
        "Cash",
        "",
        "50.00",
        "Expense",
        "Coffee",
    );

    let date = NaiveDate::from_ymd_opt(2024, 9, 1).unwrap();
    let tx_view = db_conn
        .fetch_txs_with_date(date, FetchNature::Monthly)
        .unwrap();
    assert_eq!(tx_view.len(), 3);
    // Balance: 500 - 100 - 50 = 350
    assert_eq!(tx_view.get_tx_balance(2)[&cash_id].value(), 35000);

    db_conn.delete_tx(&tx1).unwrap();

    let tx_view = db_conn
        .fetch_txs_with_date(date, FetchNature::Monthly)
        .unwrap();
    assert_eq!(tx_view.len(), 2);
    // Without the income: 0 - 100 - 50 = -150
    assert_eq!(tx_view.get_tx_balance(1)[&cash_id].value(), -15000);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn delete_middle_tx_maintains_remaining_order() {
    let file_name = "test_delete_middle_tx.sqlite";

    let mut db_conn = create_test_db(file_name);

    add_tx(
        &mut db_conn,
        "2024-10-01",
        "Salary",
        "Cash",
        "",
        "1000.00",
        "Income",
        "Salary",
    );
    let middle = add_tx(
        &mut db_conn,
        "2024-10-01",
        "Rent",
        "Cash",
        "",
        "500.00",
        "Expense",
        "Rent",
    );
    add_tx(
        &mut db_conn,
        "2024-10-01",
        "Food",
        "Cash",
        "",
        "100.00",
        "Expense",
        "Food",
    );

    let date = NaiveDate::from_ymd_opt(2024, 10, 1).unwrap();
    let tx_view = db_conn
        .fetch_txs_with_date(date, FetchNature::Monthly)
        .unwrap();
    assert_eq!(tx_view.len(), 3);

    db_conn.delete_tx(&middle).unwrap();

    let tx_view = db_conn
        .fetch_txs_with_date(date, FetchNature::Monthly)
        .unwrap();
    assert_eq!(tx_view.len(), 2);
    // Details should be Salary and Food (Rent removed)
    assert_eq!(tx_view.get_tx(0).details, Some("Salary".to_string()));
    assert_eq!(tx_view.get_tx(1).details, Some("Food".to_string()));

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}
