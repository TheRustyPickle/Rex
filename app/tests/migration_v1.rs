use chrono::NaiveDate;
use diesel::RunQueryDsl;
use rex_app::conn::{DbConn, FetchNature, get_conn};
use rex_db::ConnCache;
use rex_db::models::Balance;
use std::fs;

use crate::common::{add_tx, create_test_db};

mod common;

fn monthly_balance(db: &mut DbConn, method: &str, year: i32, month: u32) -> i64 {
    let method_id = db.cache().get_method_id(method).unwrap();
    let date = NaiveDate::from_ymd_opt(year, month, 1).unwrap();

    Balance::get_balance(date, FetchNature::Monthly, db)
        .unwrap()
        .into_iter()
        .find(|b| b.method_id == method_id)
        .unwrap()
        .balance
}

fn final_balance(db: &mut DbConn, method: &str) -> i64 {
    let method_id = db.cache().get_method_id(method).unwrap();

    db.get_final_balances().unwrap()[&method_id].balance
}

fn cash_months(db: &mut DbConn, months: std::ops::RangeInclusive<u32>) -> Vec<i64> {
    months
        .map(|month| monthly_balance(db, "Cash", 2024, month))
        .collect()
}

/// Income in Jan, expense in Feb, income in Apr: Cash ends at 100 / 70 / 70 / 120
fn seed_cash_history(db: &mut DbConn) {
    add_tx(db, "2024-01-10", "a", "Cash", "", "100.00", "Income", "x");
    add_tx(db, "2024-02-10", "b", "Cash", "", "30.00", "Expense", "x");
    add_tx(db, "2024-04-10", "c", "Cash", "", "50.00", "Income", "x");
}

fn run_sql(db: &mut DbConn, query: &str) {
    diesel::sql_query(query).execute(&mut db.conn).unwrap();
}

#[test]
fn migration_on_database_without_any_methods_is_a_no_op() {
    let file_name = "test_migration_v1_no_methods.sqlite";
    let _ = fs::remove_file(file_name);

    let mut db_conn = get_conn(file_name);
    db_conn.initiate_v1_migration().unwrap();

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn migration_without_transactions_is_a_no_op() {
    let file_name = "test_migration_v1_no_txs.sqlite";
    let mut db_conn = create_test_db(file_name);

    db_conn.initiate_v1_migration().unwrap();

    for method in ["Cash", "Bank", "Other"] {
        assert_eq!(final_balance(&mut db_conn, method), 0);
    }

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn migration_leaves_correct_balances_untouched() {
    let file_name = "test_migration_v1_untouched.sqlite";
    let mut db_conn = create_test_db(file_name);
    seed_cash_history(&mut db_conn);

    db_conn.initiate_v1_migration().unwrap();

    assert_eq!(
        cash_months(&mut db_conn, 1..=4),
        vec![10000, 7000, 7000, 12000]
    );
    assert_eq!(final_balance(&mut db_conn, "Cash"), 12000);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn migration_repairs_corrupted_monthly_and_final_balances() {
    let file_name = "test_migration_v1_repair.sqlite";
    let mut db_conn = create_test_db(file_name);
    seed_cash_history(&mut db_conn);

    run_sql(
        &mut db_conn,
        "UPDATE balances SET balance = balance + 777 WHERE is_final_balance = 0 AND month IN (1, 2, 3)",
    );
    run_sql(
        &mut db_conn,
        "UPDATE balances SET balance = 1 WHERE is_final_balance = 1",
    );

    db_conn.initiate_v1_migration().unwrap();

    assert_eq!(
        cash_months(&mut db_conn, 1..=4),
        vec![10000, 7000, 7000, 12000]
    );
    assert_eq!(final_balance(&mut db_conn, "Cash"), 12000);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn migration_is_idempotent() {
    let file_name = "test_migration_v1_idempotent.sqlite";
    let mut db_conn = create_test_db(file_name);
    seed_cash_history(&mut db_conn);

    run_sql(
        &mut db_conn,
        "UPDATE balances SET balance = balance + 500 WHERE is_final_balance = 0",
    );

    db_conn.initiate_v1_migration().unwrap();
    let first_run = cash_months(&mut db_conn, 1..=4);
    let first_final = final_balance(&mut db_conn, "Cash");

    db_conn.initiate_v1_migration().unwrap();

    assert_eq!(cash_months(&mut db_conn, 1..=4), first_run);
    assert_eq!(final_balance(&mut db_conn, "Cash"), first_final);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn migration_starts_from_the_oldest_tx_even_if_its_balance_rows_are_missing() {
    let file_name = "test_migration_v1_missing_oldest.sqlite";
    let mut db_conn = create_test_db(file_name);
    seed_cash_history(&mut db_conn);

    // The oldest month's balance rows are gone while its transaction still exists
    run_sql(
        &mut db_conn,
        "DELETE FROM balances WHERE is_final_balance = 0 AND month = 1",
    );

    db_conn.initiate_v1_migration().unwrap();

    assert_eq!(
        cash_months(&mut db_conn, 1..=4),
        vec![10000, 7000, 7000, 12000]
    );
    assert_eq!(final_balance(&mut db_conn, "Cash"), 12000);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn migration_repairs_every_method_including_transfers() {
    let file_name = "test_migration_v1_transfer.sqlite";
    let mut db_conn = create_test_db(file_name);

    add_tx(
        &mut db_conn,
        "2024-01-10",
        "a",
        "Cash",
        "",
        "100.00",
        "Income",
        "x",
    );
    add_tx(
        &mut db_conn,
        "2024-02-10",
        "b",
        "Cash",
        "Bank",
        "40.00",
        "Transfer",
        "x",
    );

    run_sql(
        &mut db_conn,
        "UPDATE balances SET balance = balance + 500 WHERE is_final_balance = 0",
    );
    run_sql(
        &mut db_conn,
        "UPDATE balances SET balance = 9 WHERE is_final_balance = 1",
    );

    db_conn.initiate_v1_migration().unwrap();

    assert_eq!(monthly_balance(&mut db_conn, "Cash", 2024, 1), 10000);
    assert_eq!(monthly_balance(&mut db_conn, "Cash", 2024, 2), 6000);
    assert_eq!(monthly_balance(&mut db_conn, "Bank", 2024, 1), 0);
    assert_eq!(monthly_balance(&mut db_conn, "Bank", 2024, 2), 4000);
    assert_eq!(final_balance(&mut db_conn, "Cash"), 6000);
    assert_eq!(final_balance(&mut db_conn, "Bank"), 4000);
    assert_eq!(final_balance(&mut db_conn, "Other"), 0);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}
