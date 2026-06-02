use rex_app::modifier::parse_search_fields;
use std::fs;

use crate::common::{add_tx, create_test_db};

mod common;

#[test]
fn search_view_empty_by_constructor() {
    let view = rex_app::views::SearchView::new_empty();
    assert!(view.is_empty());
    assert_eq!(view.tx_array().len(), 0);
}

#[test]
fn search_view_with_results() {
    let file_name = "test_search_view_results.sqlite";
    let mut db_conn = create_test_db(file_name);

    add_tx(
        &mut db_conn, "2024-06-01", "Rent", "Cash", "", "500.00", "Expense", "Rent",
    );
    add_tx(
        &mut db_conn, "2024-06-01", "Salary", "Cash", "", "1000.00", "Income", "Work",
    );

    let search = parse_search_fields(
        "", "", "", "", "", "", "", &db_conn,
    )
    .unwrap();
    let view = db_conn.search_txs(search).unwrap();

    assert!(!view.is_empty());
    assert_eq!(view.get_tx(0).details, Some("Rent".into()));
    assert_eq!(view.get_tx(1).details, Some("Salary".into()));

    let arr = view.tx_array();
    assert_eq!(arr.len(), 2);
    // Each row has 6 columns: date, details, method, amount, tx_type, tags
    assert_eq!(arr[0].len(), 6);
    assert_eq!(arr[0][1], "Rent");
    assert_eq!(arr[0][4], "Expense");
    assert_eq!(arr[1][1], "Salary");
    assert_eq!(arr[1][4], "Income");

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn search_view_no_results() {
    let file_name = "test_search_view_empty.sqlite";
    let mut db_conn = create_test_db(file_name);

    add_tx(
        &mut db_conn, "2024-07-01", "Only tx", "Cash", "", "10.00", "Expense", "Tag",
    );

    // Search for something that doesn't match
    let search = parse_search_fields(
        "", "", "", "", "", "Income", "", &db_conn,
    )
    .unwrap();
    let view = db_conn.search_txs(search).unwrap();

    assert!(view.is_empty());
    assert_eq!(view.tx_array().len(), 0);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}
