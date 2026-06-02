use rex_app::modifier::parse_search_fields;
use std::fs;

use crate::common::{add_tx, create_test_db};

mod common;

fn search_and_count(db_conn: &mut rex_app::conn::DbConn, amount: &str) -> usize {
    let search = parse_search_fields("", "", "", "", amount, "", "", db_conn).unwrap();
    search.search_txs(db_conn).unwrap().len()
}

#[test]
fn search_amount_exact() {
    let file_name = "test_search_amount_exact.sqlite";
    let mut db_conn = create_test_db(file_name);

    add_tx(
        &mut db_conn,
        "2024-01-01",
        "T1",
        "Cash",
        "",
        "10.00",
        "Expense",
        "A",
    );
    add_tx(
        &mut db_conn,
        "2024-01-01",
        "T2",
        "Cash",
        "",
        "50.00",
        "Expense",
        "B",
    );
    add_tx(
        &mut db_conn,
        "2024-01-01",
        "T3",
        "Cash",
        "",
        "100.00",
        "Expense",
        "C",
    );
    add_tx(
        &mut db_conn,
        "2024-01-01",
        "T4",
        "Cash",
        "",
        "50.00",
        "Expense",
        "D",
    );

    assert_eq!(search_and_count(&mut db_conn, "50.00"), 2);
    assert_eq!(search_and_count(&mut db_conn, "10.00"), 1);
    assert_eq!(search_and_count(&mut db_conn, "100.00"), 1);
    assert_eq!(search_and_count(&mut db_conn, "999.00"), 0);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn search_amount_more_than() {
    let file_name = "test_search_amount_more.sqlite";
    let mut db_conn = create_test_db(file_name);

    add_tx(
        &mut db_conn,
        "2024-02-01",
        "A",
        "Cash",
        "",
        "10.00",
        "Expense",
        "A",
    );
    add_tx(
        &mut db_conn,
        "2024-02-01",
        "B",
        "Cash",
        "",
        "50.00",
        "Expense",
        "B",
    );
    add_tx(
        &mut db_conn,
        "2024-02-01",
        "C",
        "Cash",
        "",
        "100.00",
        "Expense",
        "C",
    );

    assert_eq!(search_and_count(&mut db_conn, ">50.00"), 1);
    assert_eq!(search_and_count(&mut db_conn, ">10.00"), 2);
    assert_eq!(search_and_count(&mut db_conn, ">100.00"), 0);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn search_amount_more_than_equal() {
    let file_name = "test_search_amount_more_eq.sqlite";
    let mut db_conn = create_test_db(file_name);

    add_tx(
        &mut db_conn,
        "2024-03-01",
        "A",
        "Cash",
        "",
        "50.00",
        "Expense",
        "A",
    );
    add_tx(
        &mut db_conn,
        "2024-03-01",
        "B",
        "Cash",
        "",
        "100.00",
        "Expense",
        "B",
    );

    assert_eq!(search_and_count(&mut db_conn, ">=50.00"), 2);
    assert_eq!(search_and_count(&mut db_conn, ">=100.00"), 1);
    assert_eq!(search_and_count(&mut db_conn, ">=200.00"), 0);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn search_amount_less_than() {
    let file_name = "test_search_amount_less.sqlite";
    let mut db_conn = create_test_db(file_name);

    add_tx(
        &mut db_conn,
        "2024-04-01",
        "A",
        "Cash",
        "",
        "10.00",
        "Expense",
        "A",
    );
    add_tx(
        &mut db_conn,
        "2024-04-01",
        "B",
        "Cash",
        "",
        "50.00",
        "Expense",
        "B",
    );
    add_tx(
        &mut db_conn,
        "2024-04-01",
        "C",
        "Cash",
        "",
        "100.00",
        "Expense",
        "C",
    );

    assert_eq!(search_and_count(&mut db_conn, "<50.00"), 1);
    assert_eq!(search_and_count(&mut db_conn, "<100.00"), 2);
    assert_eq!(search_and_count(&mut db_conn, "<10.00"), 0);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn search_amount_less_than_equal() {
    let file_name = "test_search_amount_less_eq.sqlite";
    let mut db_conn = create_test_db(file_name);

    add_tx(
        &mut db_conn,
        "2024-05-01",
        "A",
        "Cash",
        "",
        "50.00",
        "Expense",
        "A",
    );
    add_tx(
        &mut db_conn,
        "2024-05-01",
        "B",
        "Cash",
        "",
        "100.00",
        "Expense",
        "B",
    );

    assert_eq!(search_and_count(&mut db_conn, "<=50.00"), 1);
    assert_eq!(search_and_count(&mut db_conn, "<=100.00"), 2);
    assert_eq!(search_and_count(&mut db_conn, "<=10.00"), 0);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn search_amount_combined_with_other_filters() {
    let file_name = "test_search_amount_combined.sqlite";
    let mut db_conn = create_test_db(file_name);

    add_tx(
        &mut db_conn,
        "2024-06-01",
        "Groceries",
        "Cash",
        "",
        "50.00",
        "Expense",
        "Food",
    );
    add_tx(
        &mut db_conn,
        "2024-06-01",
        "Groceries",
        "Cash",
        "",
        "80.00",
        "Expense",
        "Food",
    );
    add_tx(
        &mut db_conn,
        "2024-06-01",
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
        "Groceries",
        "Cash",
        "",
        "90.00",
        "Expense",
        "Food",
    );

    // Expense Groceries > $50 in June 2024
    let search = parse_search_fields(
        "2024-06",
        "Groceries",
        "Cash",
        "",
        ">50.00",
        "Expense",
        "Food",
        &db_conn,
    )
    .unwrap();
    let results = search.search_txs(&mut db_conn).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].amount.value(), 8000);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}
