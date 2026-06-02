use chrono::{Datelike, Local};
use std::fs;

use crate::common::{add_tx, create_test_db};

mod common;

#[test]
fn activity_view_empty() {
    let file_name = "test_activity_empty.sqlite";
    let mut db_conn = create_test_db(file_name);

    // Query a month with no activity
    let view = db_conn
        .get_activity_view_with_str("January", "2020")
        .unwrap();

    assert!(view.is_empty());
    assert_eq!(view.total_activity(), 0);
    assert!(view.get_activity_table().is_empty());
    assert_eq!(view.get_activity_txs_table(None).len(), 0);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn activity_view_after_add_tx() {
    let file_name = "test_activity_add.sqlite";
    let mut db_conn = create_test_db(file_name);

    add_tx(
        &mut db_conn,
        "2024-05-01",
        "Test tx",
        "Cash",
        "",
        "10.00",
        "Expense",
        "Tag",
    );

    // Activity date is Local::now(), so query the current month
    let now = Local::now().naive_local();
    let month_names = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    let month = month_names[now.month0() as usize];
    let year = now.year().to_string();

    let view = db_conn.get_activity_view_with_str(month, &year).unwrap();

    assert!(!view.is_empty());
    assert!(view.total_activity() >= 1);

    // get_activity_txs returns the transaction snapshots within the activity
    let txs = view.get_activity_txs(0);
    assert!(!txs.is_empty());

    // add_extra_field is false for AddTx
    assert!(!view.add_extra_field(0));

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn activity_view_edit_has_extra_field() {
    let file_name = "test_activity_edit.sqlite";
    let mut db_conn = create_test_db(file_name);

    let old_tx = add_tx(
        &mut db_conn,
        "2024-06-01",
        "Original",
        "Cash",
        "",
        "10.00",
        "Expense",
        "A",
    );

    let new_tx = rex_app::modifier::parse_tx_fields(
        "2024-06-01",
        "Changed",
        "Cash",
        "",
        "20.00",
        "Expense",
        &db_conn,
    )
    .unwrap();
    db_conn.edit_tx(&old_tx, new_tx, "A").unwrap();

    let now = Local::now().naive_local();
    let month_names = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    let month = month_names[now.month0() as usize];
    let year = now.year().to_string();

    let view = db_conn.get_activity_view_with_str(month, &year).unwrap();

    // Find the EditTx activity
    let mut found = false;
    for i in 0..view.total_activity() {
        if view.add_extra_field(i) {
            found = true;
            // EditTx should have old and new tx displayed
            let table = view.get_activity_txs_table(Some(i));
            assert!(!table.is_empty());
            break;
        }
    }
    assert!(found, "Expected an EditTx activity with extra field");

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn activity_view_get_table_returns_formatted() {
    let file_name = "test_activity_table.sqlite";
    let mut db_conn = create_test_db(file_name);

    add_tx(
        &mut db_conn,
        "2024-07-01",
        "Test",
        "Cash",
        "",
        "15.00",
        "Expense",
        "Tag",
    );

    let now = Local::now().naive_local();
    let month_names = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    let month = month_names[now.month0() as usize];
    let year = now.year().to_string();

    let view = db_conn.get_activity_view_with_str(month, &year).unwrap();

    let table = view.get_activity_table();
    assert!(!table.is_empty());
    // Each activity row has date and activity type
    assert!(
        table[0].len() >= 2,
        "Activity table row should have date + type"
    );

    let txs_table = view.get_activity_txs_table(Some(0));
    assert!(!txs_table.is_empty());

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}
