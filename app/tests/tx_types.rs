use chrono::NaiveDate;
use rex_app::conn::FetchNature;
use rex_app::modifier::parse_tx_fields;
use rex_db::ConnCache;
use std::fs;

use crate::common::{add_tx, create_test_db};

mod common;

#[test]
fn transfer_moves_money_between_methods() {
    let file_name = "test_transfer_basic.sqlite";
    let mut db_conn = create_test_db(file_name);
    let cash_id = db_conn.cache().get_method_id("Cash").unwrap();
    let bank_id = db_conn.cache().get_method_id("Bank").unwrap();
    let other_id = db_conn.cache().get_method_id("Other").unwrap();

    // Seed Cash with income
    add_tx(
        &mut db_conn,
        "2024-06-01",
        "Salary",
        "Cash",
        "",
        "1000.00",
        "Income",
        "Salary",
    );

    // Transfer 300 from Cash to Bank
    add_tx(
        &mut db_conn,
        "2024-06-10",
        "Move funds",
        "Cash",
        "Bank",
        "300.00",
        "Transfer",
        "Transfer",
    );

    let date = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
    let tx_view = db_conn
        .fetch_txs_with_date(date, FetchNature::Monthly)
        .unwrap();
    assert_eq!(tx_view.len(), 2);

    let balance = tx_view.get_tx_balance(1);
    assert_eq!(balance[&cash_id].value(), 70000); // 1000 - 300
    assert_eq!(balance[&bank_id].value(), 30000); // 0 + 300
    assert_eq!(balance[&other_id].value(), 0); // unchanged

    // Total across all methods unchanged
    let total: i64 = balance.values().map(|c| c.value()).sum();
    assert_eq!(total, 100000);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn transfer_then_reverse_transfer_returns_to_original() {
    let file_name = "test_transfer_reverse.sqlite";
    let mut db_conn = create_test_db(file_name);
    let cash_id = db_conn.cache().get_method_id("Cash").unwrap();
    let bank_id = db_conn.cache().get_method_id("Bank").unwrap();

    add_tx(
        &mut db_conn,
        "2024-07-01",
        "Salary",
        "Cash",
        "",
        "1000.00",
        "Income",
        "Salary",
    );

    add_tx(
        &mut db_conn,
        "2024-07-05",
        "To Bank",
        "Cash",
        "Bank",
        "300.00",
        "Transfer",
        "Transfer",
    );

    add_tx(
        &mut db_conn,
        "2024-07-10",
        "Back",
        "Bank",
        "Cash",
        "300.00",
        "Transfer",
        "Transfer",
    );

    let date = NaiveDate::from_ymd_opt(2024, 7, 1).unwrap();
    let tx_view = db_conn
        .fetch_txs_with_date(date, FetchNature::Monthly)
        .unwrap();
    let balance = tx_view.get_tx_balance(2);
    assert_eq!(balance[&cash_id].value(), 100000); // 1000 - 300 + 300
    assert_eq!(balance[&bank_id].value(), 0); // 0 + 300 - 300

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn delete_transfer_reverses_both_methods() {
    let file_name = "test_transfer_delete.sqlite";
    let mut db_conn = create_test_db(file_name);
    let cash_id = db_conn.cache().get_method_id("Cash").unwrap();
    let bank_id = db_conn.cache().get_method_id("Bank").unwrap();

    add_tx(
        &mut db_conn,
        "2024-08-01",
        "Salary",
        "Cash",
        "",
        "500.00",
        "Income",
        "Salary",
    );

    let transfer = add_tx(
        &mut db_conn,
        "2024-08-10",
        "Move",
        "Cash",
        "Bank",
        "200.00",
        "Transfer",
        "Transfer",
    );

    let date = NaiveDate::from_ymd_opt(2024, 8, 1).unwrap();
    let tx_view = db_conn
        .fetch_txs_with_date(date, FetchNature::Monthly)
        .unwrap();
    assert_eq!(tx_view.get_tx_balance(1)[&cash_id].value(), 30000);
    assert_eq!(tx_view.get_tx_balance(1)[&bank_id].value(), 20000);

    db_conn.delete_tx(&transfer).unwrap();

    let tx_view = db_conn
        .fetch_txs_with_date(date, FetchNature::Monthly)
        .unwrap();
    assert_eq!(tx_view.len(), 1);
    assert_eq!(tx_view.get_tx_balance(0)[&cash_id].value(), 50000);
    assert_eq!(tx_view.get_tx_balance(0)[&bank_id].value(), 0);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn edit_transfer_amount_updates_both_methods() {
    let file_name = "test_transfer_edit.sqlite";
    let mut db_conn = create_test_db(file_name);
    let cash_id = db_conn.cache().get_method_id("Cash").unwrap();
    let bank_id = db_conn.cache().get_method_id("Bank").unwrap();

    add_tx(
        &mut db_conn,
        "2024-09-01",
        "Salary",
        "Cash",
        "",
        "1000.00",
        "Income",
        "Salary",
    );

    let transfer = add_tx(
        &mut db_conn,
        "2024-09-10",
        "Move",
        "Cash",
        "Bank",
        "100.00",
        "Transfer",
        "Transfer",
    );

    let new_tx = parse_tx_fields(
        "2024-09-10",
        "Move",
        "Cash",
        "Bank",
        "500.00",
        "Transfer",
        &db_conn,
    )
    .unwrap();
    db_conn.edit_tx(&transfer, new_tx, "Transfer").unwrap();

    let date = NaiveDate::from_ymd_opt(2024, 9, 1).unwrap();
    let tx_view = db_conn
        .fetch_txs_with_date(date, FetchNature::Monthly)
        .unwrap();
    assert_eq!(tx_view.get_tx_balance(1)[&cash_id].value(), 50000); // 1000 - 500
    assert_eq!(tx_view.get_tx_balance(1)[&bank_id].value(), 50000); // 0 + 500

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn borrow_increases_balance() {
    let file_name = "test_borrow.sqlite";
    let mut db_conn = create_test_db(file_name);
    let cash_id = db_conn.cache().get_method_id("Cash").unwrap();

    add_tx(
        &mut db_conn,
        "2024-10-01",
        "Borrow from bank",
        "Cash",
        "",
        "500.00",
        "Borrow",
        "Loan",
    );

    let date = NaiveDate::from_ymd_opt(2024, 10, 1).unwrap();
    let tx_view = db_conn
        .fetch_txs_with_date(date, FetchNature::Monthly)
        .unwrap();
    assert_eq!(tx_view.get_tx_balance(0)[&cash_id].value(), 50000);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn borrow_repay_decreases_balance() {
    let file_name = "test_borrow_repay.sqlite";
    let mut db_conn = create_test_db(file_name);
    let cash_id = db_conn.cache().get_method_id("Cash").unwrap();

    // Need money first
    add_tx(
        &mut db_conn,
        "2024-11-01",
        "Borrow",
        "Cash",
        "",
        "500.00",
        "Borrow",
        "Loan",
    );

    add_tx(
        &mut db_conn,
        "2024-11-15",
        "Repay loan",
        "Cash",
        "",
        "500.00",
        "Borrow Repay",
        "Loan",
    );

    let date = NaiveDate::from_ymd_opt(2024, 11, 1).unwrap();
    let tx_view = db_conn
        .fetch_txs_with_date(date, FetchNature::Monthly)
        .unwrap();
    // Borrow +500, Repay -500 = 0
    assert_eq!(tx_view.get_tx_balance(1)[&cash_id].value(), 0);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn lend_decreases_balance() {
    let file_name = "test_lend.sqlite";
    let mut db_conn = create_test_db(file_name);
    let cash_id = db_conn.cache().get_method_id("Cash").unwrap();

    add_tx(
        &mut db_conn,
        "2024-12-01",
        "Income",
        "Cash",
        "",
        "1000.00",
        "Income",
        "Salary",
    );

    add_tx(
        &mut db_conn,
        "2024-12-10",
        "Lend to friend",
        "Cash",
        "",
        "200.00",
        "Lend",
        "Friend",
    );

    let date = NaiveDate::from_ymd_opt(2024, 12, 1).unwrap();
    let tx_view = db_conn
        .fetch_txs_with_date(date, FetchNature::Monthly)
        .unwrap();
    assert_eq!(tx_view.get_tx_balance(1)[&cash_id].value(), 80000);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn lend_repay_increases_balance() {
    let file_name = "test_lend_repay.sqlite";
    let mut db_conn = create_test_db(file_name);
    let cash_id = db_conn.cache().get_method_id("Cash").unwrap();

    add_tx(
        &mut db_conn,
        "2025-01-01",
        "Lend",
        "Cash",
        "",
        "200.00",
        "Lend",
        "Friend",
    );

    add_tx(
        &mut db_conn,
        "2025-01-20",
        "Friend repaid",
        "Cash",
        "",
        "200.00",
        "Lend Repay",
        "Friend",
    );

    let date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
    let tx_view = db_conn
        .fetch_txs_with_date(date, FetchNature::Monthly)
        .unwrap();
    // Lend -200, Repay +200 = 0
    assert_eq!(tx_view.get_tx_balance(1)[&cash_id].value(), 0);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn all_tx_types_impact_balance_correctly() {
    let file_name = "test_all_tx_types.sqlite";
    let mut db_conn = create_test_db(file_name);
    let cash_id = db_conn.cache().get_method_id("Cash").unwrap();
    let bank_id = db_conn.cache().get_method_id("Bank").unwrap();

    // Income: +500 Cash
    add_tx(
        &mut db_conn,
        "2025-02-01",
        "Income",
        "Cash",
        "",
        "500.00",
        "Income",
        "Salary",
    );

    // Expense: -100 Cash
    add_tx(
        &mut db_conn,
        "2025-02-05",
        "Expense",
        "Cash",
        "",
        "100.00",
        "Expense",
        "Food",
    );

    // Transfer: -200 from Cash, +200 to Bank
    add_tx(
        &mut db_conn,
        "2025-02-10",
        "Transfer",
        "Cash",
        "Bank",
        "200.00",
        "Transfer",
        "Xfer",
    );

    // Borrow: +300 Cash
    add_tx(
        &mut db_conn,
        "2025-02-15",
        "Borrow",
        "Cash",
        "",
        "300.00",
        "Borrow",
        "Loan",
    );

    // BorrowRepay: -150 Cash
    add_tx(
        &mut db_conn,
        "2025-02-20",
        "Borrow Repay",
        "Cash",
        "",
        "150.00",
        "Borrow Repay",
        "Loan",
    );

    // Lend: -50 Cash
    add_tx(
        &mut db_conn,
        "2025-02-25",
        "Lend",
        "Cash",
        "",
        "50.00",
        "Lend",
        "Friend",
    );

    // LendRepay: +25 Cash
    add_tx(
        &mut db_conn,
        "2025-02-28",
        "Lend Repay",
        "Cash",
        "",
        "25.00",
        "Lend Repay",
        "Friend",
    );

    let date = NaiveDate::from_ymd_opt(2025, 2, 1).unwrap();
    let tx_view = db_conn
        .fetch_txs_with_date(date, FetchNature::Monthly)
        .unwrap();
    let balance = tx_view.get_tx_balance(6);

    // Cash: 500 - 100 - 200 + 300 - 150 - 50 + 25 = 325
    assert_eq!(balance[&cash_id].value(), 32500);
    // Bank: 0 + 200 = 200
    assert_eq!(balance[&bank_id].value(), 20000);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}
