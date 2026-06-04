use chrono::NaiveDate;
use rex_app::conn::{DbConn, FetchNature};
use rex_app::modifier::parse_tx_fields;
use rex_db::ConnCache;
use std::fs;

use crate::common::create_test_db;

mod common;

#[test]
fn add_tx_test() {
    let file_name = "test_add_tx.sqlite";

    let mut db_conn = create_test_db(file_name);
    let cash_method = db_conn.cache().get_method_id("Cash").unwrap();
    let bank_method = db_conn.cache().get_method_id("Bank").unwrap();

    let tx_list = [
        [
            "2022-07-01",
            "Salary",
            "Cash",
            "",
            "1000.00",
            "Income",
            "Salary",
        ],
        [
            "2022-08-01",
            "Car expense",
            "Cash",
            "",
            "100.00",
            "Expense",
            "Car, Maintenance",
        ],
        [
            "2022-09-01",
            "Bankruptcy",
            "Cash",
            "",
            "900.00",
            "Expense",
            "Bankruptcy",
        ],
        [
            "2022-10-01",
            "Inheritance",
            "Cash",
            "",
            "5000.00",
            "Income",
            "Inheritance",
        ],
        [
            "2022-10-01",
            "Groceries",
            "Cash",
            "",
            "100.00",
            "Expense",
            "Groceries",
        ],
        [
            "2022-10-01",
            "More Groceries",
            "Cash",
            "",
            "100.00",
            "Expense",
            "Groceries",
        ],
        [
            "2022-10-05",
            "Rent",
            "Cash",
            "",
            "100.00",
            "Expense",
            "Rent",
        ],
        [
            "2022-11-01",
            "Debt",
            "Cash",
            "",
            "10000.00",
            "Expense",
            "Debt",
        ],
    ];

    let date_list = [
        NaiveDate::from_ymd_opt(2022, 7, 1).unwrap(),
        NaiveDate::from_ymd_opt(2022, 8, 1).unwrap(),
        NaiveDate::from_ymd_opt(2022, 9, 1).unwrap(),
        NaiveDate::from_ymd_opt(2022, 10, 1).unwrap(),
        NaiveDate::from_ymd_opt(2022, 10, 1).unwrap(),
        NaiveDate::from_ymd_opt(2022, 10, 1).unwrap(),
        NaiveDate::from_ymd_opt(2022, 10, 1).unwrap(),
        NaiveDate::from_ymd_opt(2022, 11, 1).unwrap(),
    ];

    let balance_list = [
        1000 * 100,
        900 * 100,
        0,
        5000 * 100,
        4900 * 100,
        4800 * 100,
        4700 * 100,
        (4700 - 10000) * 100,
    ];

    let tx_len = [1, 1, 1, 1, 2, 3, 4, 1];

    for (index, tx_fields) in tx_list.iter().enumerate() {
        let new_tx = parse_tx_fields(
            tx_fields[0],
            tx_fields[1],
            tx_fields[2],
            tx_fields[3],
            tx_fields[4],
            tx_fields[5],
            &db_conn,
        )
        .unwrap();

        db_conn.add_new_tx(new_tx, tx_fields[6]).unwrap();

        let tx_list = db_conn
            .fetch_txs_with_date(date_list[index], FetchNature::Monthly)
            .unwrap();

        assert_eq!(
            tx_list.len(),
            tx_len[index],
            "Length mismatch at index {}",
            index
        );

        let balance = tx_list.get_tx_balance(tx_len[index] - 1)[&cash_method];
        assert_eq!(
            balance, balance_list[index],
            "Balance mismatch at index {}",
            index
        );
    }

    // Verify individual tx fields after all inserts — October has 4 txs
    let oct = NaiveDate::from_ymd_opt(2022, 10, 1).unwrap();
    let oct_txs = db_conn
        .fetch_txs_with_date(oct, FetchNature::Monthly)
        .unwrap();

    let tx0 = oct_txs.get_tx(0);
    assert_eq!(tx0.details, Some("Inheritance".into()));
    assert_eq!(tx0.amount.value(), 500000);
    assert_eq!(tx0.from_method.name, "Cash");
    assert_eq!(tx0.tags[0].name, "Inheritance");
    assert_eq!(tx0.to_array(false).len(), 6);

    let tx1 = oct_txs.get_tx(1);
    assert_eq!(tx1.details, Some("Groceries".into()));
    assert_eq!(tx1.tags[0].name, "Groceries");

    let tx2 = oct_txs.get_tx(2);
    assert_eq!(tx2.details, Some("More Groceries".into()));
    // Same tag "Groceries" reused
    assert_eq!(tx2.tags[0].name, "Groceries");

    let tx3 = oct_txs.get_tx(3);
    assert_eq!(tx3.details, Some("Rent".into()));
    assert_eq!(tx3.tags[0].name, "Rent");

    // July's single tx
    let jul = NaiveDate::from_ymd_opt(2022, 7, 1).unwrap();
    let jul_txs = db_conn
        .fetch_txs_with_date(jul, FetchNature::Monthly)
        .unwrap();
    assert_eq!(jul_txs.get_tx(0).details, Some("Salary".into()));
    assert_eq!(jul_txs.get_tx(0).tags[0].name, "Salary");

    // Bank method should still be at zero — no txs touched it
    let jul_balance = jul_txs.get_tx_balance(0);
    assert_eq!(jul_balance[&bank_method].value(), 0);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn add_tx_yearly_fetch() {
    let file_name = "test_add_tx_yearly.sqlite";
    let mut db_conn = create_test_db(file_name);

    add_tx_helper(
        &mut db_conn,
        "2024-01-15",
        "Jan",
        "Cash",
        "",
        "100.00",
        "Income",
        "A",
    );
    add_tx_helper(
        &mut db_conn,
        "2024-06-20",
        "Jun",
        "Cash",
        "",
        "200.00",
        "Expense",
        "B",
    );
    add_tx_helper(
        &mut db_conn,
        "2024-12-01",
        "Dec",
        "Cash",
        "",
        "50.00",
        "Expense",
        "C",
    );

    let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let yearly = db_conn
        .fetch_txs_with_date(date, FetchNature::Yearly)
        .unwrap();
    assert_eq!(yearly.len(), 3);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn add_tx_all_fetch() {
    let file_name = "test_add_tx_all.sqlite";
    let mut db_conn = create_test_db(file_name);

    add_tx_helper(
        &mut db_conn,
        "2023-06-15",
        "Old",
        "Cash",
        "",
        "10.00",
        "Expense",
        "A",
    );
    add_tx_helper(
        &mut db_conn,
        "2024-01-01",
        "New",
        "Cash",
        "",
        "20.00",
        "Expense",
        "B",
    );
    add_tx_helper(
        &mut db_conn,
        "2025-03-10",
        "Future",
        "Cash",
        "",
        "30.00",
        "Expense",
        "C",
    );

    let date = NaiveDate::from_ymd_opt(2023, 6, 1).unwrap();
    let all = db_conn.fetch_txs_with_date(date, FetchNature::All).unwrap();
    assert_eq!(all.len(), 3);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn add_tx_multiple_methods_balance_independently() {
    let file_name = "test_add_tx_multi_method.sqlite";
    let mut db_conn = create_test_db(file_name);
    let cash_id = db_conn.cache().get_method_id("Cash").unwrap();
    let bank_id = db_conn.cache().get_method_id("Bank").unwrap();

    add_tx_helper(
        &mut db_conn,
        "2024-03-01",
        "Income",
        "Cash",
        "",
        "500.00",
        "Income",
        "Salary",
    );
    add_tx_helper(
        &mut db_conn,
        "2024-03-05",
        "Savings",
        "Bank",
        "",
        "200.00",
        "Income",
        "Savings",
    );
    add_tx_helper(
        &mut db_conn,
        "2024-03-10",
        "Coffee",
        "Cash",
        "",
        "50.00",
        "Expense",
        "Food",
    );

    let date = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
    let tx_view = db_conn
        .fetch_txs_with_date(date, FetchNature::Monthly)
        .unwrap();
    assert_eq!(tx_view.len(), 3);

    let balance = tx_view.get_tx_balance(2);
    assert_eq!(balance[&cash_id].value(), 45000);
    assert_eq!(balance[&bank_id].value(), 20000);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn add_tx_transfer_sets_to_method() {
    let file_name = "test_add_tx_transfer.sqlite";
    let mut db_conn = create_test_db(file_name);

    add_tx_helper(
        &mut db_conn,
        "2024-04-01",
        "Income",
        "Cash",
        "",
        "1000.00",
        "Income",
        "Salary",
    );
    add_tx_helper(
        &mut db_conn,
        "2024-04-01",
        "Move",
        "Cash",
        "Bank",
        "300.00",
        "Transfer",
        "Move",
    );

    let date = NaiveDate::from_ymd_opt(2024, 4, 1).unwrap();
    let tx_view = db_conn
        .fetch_txs_with_date(date, FetchNature::Monthly)
        .unwrap();

    let transfer = tx_view.get_tx(1);
    assert_eq!(transfer.tx_type.to_string(), "Transfer");
    assert_eq!(transfer.from_method.name, "Cash");
    assert!(transfer.to_method.is_some());
    assert_eq!(transfer.to_method.as_ref().unwrap().name, "Bank");
    // to_array for non-search shows method as "Cash → Bank"
    let array = transfer.to_array(false);
    let method_str = &array[2];
    assert!(method_str.contains("→"));

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn add_tx_to_array_search_format_differs() {
    let file_name = "test_add_tx_to_array.sqlite";
    let mut db_conn = create_test_db(file_name);

    add_tx_helper(
        &mut db_conn,
        "2024-05-15",
        "Test tx",
        "Cash",
        "",
        "42.50",
        "Expense",
        "TestTag",
    );

    let date = NaiveDate::from_ymd_opt(2024, 5, 1).unwrap();
    let tx_view = db_conn
        .fetch_txs_with_date(date, FetchNature::Monthly)
        .unwrap();
    let tx = tx_view.get_tx(0);

    let search_arr = tx.to_array(true);
    assert_eq!(search_arr[0], "2024-05-15");
    assert_eq!(search_arr[1], "Test tx");
    assert_eq!(search_arr[2], "Cash");
    assert_eq!(search_arr[3], "42.50");
    assert_eq!(search_arr[4], "Expense");
    assert_eq!(search_arr[5], "TestTag");

    // Non-search format has weekday/time in date
    let display_arr = tx.to_array(false);
    assert!(display_arr[0].contains("Wed")); // 2024-05-15 is a Wednesday
    assert!(!display_arr[0].contains("2024"));

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

// Local helper to reduce boilerplate
fn add_tx_helper(
    db_conn: &mut DbConn,
    date: &str,
    details: &str,
    from_method: &str,
    to_method: &str,
    amount: &str,
    tx_type: &str,
    tags: &str,
) {
    let new_tx = parse_tx_fields(
        date,
        details,
        from_method,
        to_method,
        amount,
        tx_type,
        db_conn,
    )
    .unwrap();
    db_conn.add_new_tx(new_tx, tags).unwrap();
}
