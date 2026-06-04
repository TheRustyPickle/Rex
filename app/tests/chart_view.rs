use chrono::NaiveDate;
use rex_app::conn::FetchNature;
use rex_db::ConnCache;
use std::fs;

use crate::common::{add_tx, create_test_db};

mod common;

#[test]
fn chart_view_empty() {
    let file_name = "test_chart_empty.sqlite";
    let mut db_conn = create_test_db(file_name);

    let chart = db_conn
        .get_chart_view_with_str("January", "2024", FetchNature::Monthly)
        .unwrap();

    assert!(chart.is_empty());
    assert_eq!(chart.len(), 0);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn chart_view_date_bounds() {
    let file_name = "test_chart_bounds.sqlite";
    let mut db_conn = create_test_db(file_name);

    add_tx(
        &mut db_conn,
        "2024-03-05",
        "First",
        "Cash",
        "",
        "10.00",
        "Expense",
        "A",
    );
    add_tx(
        &mut db_conn,
        "2024-03-15",
        "Middle",
        "Cash",
        "",
        "20.00",
        "Expense",
        "B",
    );
    add_tx(
        &mut db_conn,
        "2024-03-25",
        "Last",
        "Cash",
        "",
        "30.00",
        "Expense",
        "C",
    );

    let chart = db_conn
        .get_chart_view_with_str("March", "2024", FetchNature::Monthly)
        .unwrap();

    assert!(!chart.is_empty());
    assert_eq!(chart.len(), 3);

    assert_eq!(
        chart.start_date(),
        NaiveDate::from_ymd_opt(2024, 3, 5).unwrap()
    );
    assert_eq!(
        chart.end_date(),
        NaiveDate::from_ymd_opt(2024, 3, 25).unwrap()
    );

    assert!(chart.contains_date(&NaiveDate::from_ymd_opt(2024, 3, 15).unwrap()));
    assert!(!chart.contains_date(&NaiveDate::from_ymd_opt(2024, 3, 10).unwrap()));

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn chart_view_get_tx_and_balance() {
    let file_name = "test_chart_tx_balance.sqlite";
    let mut db_conn = create_test_db(file_name);
    let cash_id = db_conn.cache().get_method_id("Cash").unwrap();
    let bank_id = db_conn.cache().get_method_id("Bank").unwrap();

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
        "2024-06-10",
        "Transfer",
        "Cash",
        "Bank",
        "300.00",
        "Transfer",
        "Move",
    );
    add_tx(
        &mut db_conn,
        "2024-06-20",
        "Expense",
        "Cash",
        "",
        "100.00",
        "Expense",
        "Food",
    );

    let chart = db_conn
        .get_chart_view_with_str("June", "2024", FetchNature::Monthly)
        .unwrap();

    assert_eq!(chart.len(), 3);

    assert_eq!(chart.get_tx(0).details, Some("Income".into()));
    assert_eq!(chart.get_tx(1).details, Some("Transfer".into()));
    assert_eq!(chart.get_tx(2).details, Some("Expense".into()));

    // Running balances: Income(+1000 Cash), Transfer(-300 Cash, +300 Bank), Expense(-100 Cash)
    let b0 = chart.get_balance(0);
    let b1 = chart.get_balance(1);
    let b2 = chart.get_balance(2);

    assert_eq!(b0[&cash_id].value(), 100000);
    assert_eq!(b0[&bank_id].value(), 0);

    assert_eq!(b1[&cash_id].value(), 70000);
    assert_eq!(b1[&bank_id].value(), 30000);

    assert_eq!(b2[&cash_id].value(), 60000);
    assert_eq!(b2[&bank_id].value(), 30000);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn chart_view_single_tx() {
    let file_name = "test_chart_single.sqlite";
    let mut db_conn = create_test_db(file_name);

    add_tx(
        &mut db_conn,
        "2024-09-01",
        "Only",
        "Cash",
        "",
        "50.00",
        "Expense",
        "Tag",
    );

    let chart = db_conn
        .get_chart_view_with_str("September", "2024", FetchNature::Monthly)
        .unwrap();

    assert_eq!(chart.len(), 1);
    assert_eq!(chart.start_date(), chart.end_date());
    assert_eq!(
        chart.start_date(),
        NaiveDate::from_ymd_opt(2024, 9, 1).unwrap()
    );
    assert!(chart.contains_date(&NaiveDate::from_ymd_opt(2024, 9, 1).unwrap()));

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}
