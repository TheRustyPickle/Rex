use rex_app::conn::FetchNature;
use std::fs;

use crate::common::{add_tx, create_test_db};

mod common;

#[test]
fn summary_monthly_basic_income_expense() {
    let file_name = "test_summary_basic.sqlite";
    let mut db_conn = create_test_db(file_name);

    add_tx(
        &mut db_conn,
        "2024-06-01",
        "Salary",
        "Cash",
        "",
        "3000.00",
        "Income",
        "Work",
    );
    add_tx(
        &mut db_conn,
        "2024-06-15",
        "Rent",
        "Cash",
        "",
        "1000.00",
        "Expense",
        "Housing",
    );
    add_tx(
        &mut db_conn,
        "2024-06-20",
        "Food",
        "Cash",
        "",
        "500.00",
        "Expense",
        "Groceries",
    );

    let summary_view = db_conn
        .get_summary_with_str("June", "2024", FetchNature::Monthly)
        .unwrap();
    let full = summary_view.generate_summary(None, &db_conn);

    // Net: income=3000, expense=1500
    let net = full.net_array();
    assert_eq!(net[0][1], "3000.00");
    assert_eq!(net[0][2], "1500.00");

    // Per-method breakdown: only Cash has data
    let methods = full.method_array();
    let cash_row = methods.iter().find(|r| r[0] == "Cash").unwrap();
    assert_eq!(cash_row[1], "3000.00"); // earning
    assert_eq!(cash_row[2], "1500.00"); // expense

    // Largest earning: Salary 3000 on 01-06-2024
    let largest = full.largest_array();
    assert_eq!(largest[0][0], "Largest Earning");
    assert_eq!(largest[0][2], "3000.00");
    assert_eq!(largest[1][0], "Largest Expense");
    assert_eq!(largest[1][2], "1000.00");

    // Peak earning = 3000 (only one month with income)
    let peak = full.peak_array();
    assert_eq!(peak[0][0], "Peak Earning");
    assert_eq!(peak[0][2], "3000.00");
    assert_eq!(peak[1][0], "Peak Expense");
    assert_eq!(peak[1][2], "1500.00");

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn summary_largest_single_tx_each() {
    let file_name = "test_summary_largest.sqlite";
    let mut db_conn = create_test_db(file_name);

    add_tx(
        &mut db_conn,
        "2024-07-01",
        "Small",
        "Cash",
        "",
        "10.00",
        "Income",
        "A",
    );
    add_tx(
        &mut db_conn,
        "2024-07-05",
        "Big",
        "Cash",
        "",
        "5000.00",
        "Income",
        "B",
    );
    add_tx(
        &mut db_conn,
        "2024-07-10",
        "Medium",
        "Cash",
        "",
        "100.00",
        "Expense",
        "C",
    );
    add_tx(
        &mut db_conn,
        "2024-07-15",
        "Huge",
        "Cash",
        "",
        "2000.00",
        "Expense",
        "D",
    );

    let summary_view = db_conn
        .get_summary_with_str("July", "2024", FetchNature::Monthly)
        .unwrap();
    let full = summary_view.generate_summary(None, &db_conn);

    let largest = full.largest_array();
    assert_eq!(largest[0][2], "5000.00"); // largest earning
    assert_eq!(largest[1][2], "2000.00"); // largest expense

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn summary_peak_across_months() {
    let file_name = "test_summary_peak.sqlite";
    let mut db_conn = create_test_db(file_name);

    // Month 1: low earning
    add_tx(
        &mut db_conn,
        "2024-01-15",
        "Jan",
        "Cash",
        "",
        "100.00",
        "Income",
        "A",
    );
    // Month 2: high earning
    add_tx(
        &mut db_conn,
        "2024-02-15",
        "Feb",
        "Cash",
        "",
        "5000.00",
        "Income",
        "B",
    );
    // Month 3: medium earning
    add_tx(
        &mut db_conn,
        "2024-03-15",
        "Mar",
        "Cash",
        "",
        "1000.00",
        "Income",
        "C",
    );

    let summary_view = db_conn
        .get_summary_with_str("January", "2024", FetchNature::Yearly)
        .unwrap();
    let full = summary_view.generate_summary(None, &db_conn);

    let peak = full.peak_array();
    assert_eq!(peak[0][0], "Peak Earning");
    assert_eq!(peak[0][2], "5000.00");
    assert_eq!(peak[0][1], "02-2024"); // Feb date format

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn summary_lend_borrows() {
    let file_name = "test_summary_lend_borrow.sqlite";
    let mut db_conn = create_test_db(file_name);

    add_tx(
        &mut db_conn,
        "2024-08-01",
        "Borrow1",
        "Cash",
        "",
        "500.00",
        "Borrow",
        "Loan",
    );
    add_tx(
        &mut db_conn,
        "2024-08-10",
        "Lend1",
        "Cash",
        "",
        "200.00",
        "Lend",
        "Friend",
    );
    add_tx(
        &mut db_conn,
        "2024-08-15",
        "Repay half",
        "Cash",
        "",
        "250.00",
        "Borrow Repay",
        "Loan",
    );
    add_tx(
        &mut db_conn,
        "2024-08-20",
        "Got back",
        "Cash",
        "",
        "100.00",
        "Lend Repay",
        "Friend",
    );

    let summary_view = db_conn
        .get_summary_with_str("August", "2024", FetchNature::Monthly)
        .unwrap();
    let full = summary_view.generate_summary(None, &db_conn);

    // Net borrows: 500 - 250 = 250
    // Net lends: 200 - 100 = 100
    let lb = full.lend_borrows_array();
    assert_eq!(lb[0][0], "250.00");
    assert_eq!(lb[0][1], "100.00");

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn summary_multiple_methods() {
    let file_name = "test_summary_methods.sqlite";
    let mut db_conn = create_test_db(file_name);

    add_tx(
        &mut db_conn,
        "2024-09-01",
        "Income",
        "Cash",
        "",
        "1000.00",
        "Income",
        "Work",
    );
    add_tx(
        &mut db_conn,
        "2024-09-05",
        "Savings",
        "Bank",
        "",
        "500.00",
        "Income",
        "Interest",
    );
    add_tx(
        &mut db_conn,
        "2024-09-10",
        "Coffee",
        "Cash",
        "",
        "50.00",
        "Expense",
        "Food",
    );

    let summary_view = db_conn
        .get_summary_with_str("September", "2024", FetchNature::Monthly)
        .unwrap();
    let full = summary_view.generate_summary(None, &db_conn);

    let methods = full.method_array();
    let cash_row = methods.iter().find(|r| r[0] == "Cash").unwrap();
    let bank_row = methods.iter().find(|r| r[0] == "Bank").unwrap();
    let other_row = methods.iter().find(|r| r[0] == "Other").unwrap();

    assert_eq!(cash_row[1], "1000.00"); // Cash earning
    assert_eq!(cash_row[2], "50.00"); // Cash expense
    assert_eq!(bank_row[1], "500.00"); // Bank earning
    assert_eq!(bank_row[2], "0.00"); // Bank expense
    assert_eq!(other_row[1], "0.00"); // Other: no activity
    assert_eq!(other_row[2], "0.00");

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn summary_tags_array_groups_by_tag() {
    let file_name = "test_summary_tags.sqlite";
    let mut db_conn = create_test_db(file_name);

    add_tx(
        &mut db_conn,
        "2024-10-01",
        "Income",
        "Cash",
        "",
        "2000.00",
        "Income",
        "Work",
    );
    add_tx(
        &mut db_conn,
        "2024-10-10",
        "Groceries",
        "Cash",
        "",
        "200.00",
        "Expense",
        "Food",
    );
    add_tx(
        &mut db_conn,
        "2024-10-20",
        "Rent",
        "Cash",
        "",
        "1000.00",
        "Expense",
        "Housing",
    );

    let summary_view = db_conn
        .get_summary_with_str("October", "2024", FetchNature::Monthly)
        .unwrap();
    let tags = summary_view.tags_array(None, &db_conn);

    // Find row for "Work" tag
    let work_row = tags.iter().find(|r| r[0] == "Work").unwrap();
    assert_eq!(work_row[1], "2000.00"); // income amount
    assert_eq!(work_row[2], "0.00"); // expense amount

    let food_row = tags.iter().find(|r| r[0] == "Food").unwrap();
    assert_eq!(food_row[1], "0.00");
    assert_eq!(food_row[2], "200.00"); // expense

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn summary_yearly_with_monthly_averages() {
    let file_name = "test_summary_yearly.sqlite";
    let mut db_conn = create_test_db(file_name);

    // Jan: income 100
    add_tx(
        &mut db_conn,
        "2024-01-15",
        "Jan",
        "Cash",
        "",
        "100.00",
        "Income",
        "A",
    );
    // Feb: income 300
    add_tx(
        &mut db_conn,
        "2024-02-15",
        "Feb",
        "Cash",
        "",
        "300.00",
        "Income",
        "B",
    );

    let summary_view = db_conn
        .get_summary_with_str("January", "2024", FetchNature::Yearly)
        .unwrap();
    let full = summary_view.generate_summary(None, &db_conn);

    // Total income: 400, over 2 months, avg = 200
    let net = full.net_array();
    assert_eq!(net[0][1], "400.00"); // total income

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn summary_tags_array_with_compare() {
    let file_name = "test_summary_tags_compare.sqlite";
    let mut db_conn = create_test_db(file_name);

    // August: income + expense
    add_tx(
        &mut db_conn,
        "2024-08-01",
        "Salary",
        "Cash",
        "",
        "2000.00",
        "Income",
        "Work",
    );
    add_tx(
        &mut db_conn,
        "2024-08-15",
        "Rent",
        "Cash",
        "",
        "1000.00",
        "Expense",
        "Housing",
    );

    // September: different amounts
    add_tx(
        &mut db_conn,
        "2024-09-01",
        "Salary",
        "Cash",
        "",
        "2500.00",
        "Income",
        "Work",
    );
    add_tx(
        &mut db_conn,
        "2024-09-15",
        "Rent",
        "Cash",
        "",
        "800.00",
        "Expense",
        "Housing",
    );

    let sep = db_conn
        .get_summary_with_str("September", "2024", FetchNature::Monthly)
        .unwrap();
    let aug = db_conn
        .get_summary_with_str("August", "2024", FetchNature::Monthly)
        .unwrap();

    let tags = sep.tags_array(Some(&aug), &db_conn);
    // Should include MoM/YoY comparison columns with ↑/↓ percentages
    let work_row = tags.iter().find(|r| r[0] == "Work").unwrap();
    // Income amount for Work tag
    assert_eq!(work_row[1], "2500.00");
    // Should have compare columns (positions 6-7 after income/expense %)
    assert!(work_row.len() > 6, "Should have compare columns");

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn summary_with_last_summary_mom_comparison() {
    let file_name = "test_summary_mom.sqlite";
    let mut db_conn = create_test_db(file_name);

    // August
    add_tx(
        &mut db_conn,
        "2024-08-01",
        "Income",
        "Cash",
        "",
        "1000.00",
        "Income",
        "Work",
    );
    add_tx(
        &mut db_conn,
        "2024-08-15",
        "Expense",
        "Cash",
        "",
        "200.00",
        "Expense",
        "Food",
    );

    // September — higher numbers
    add_tx(
        &mut db_conn,
        "2024-09-01",
        "Income",
        "Cash",
        "",
        "2000.00",
        "Income",
        "Work",
    );
    add_tx(
        &mut db_conn,
        "2024-09-15",
        "Expense",
        "Cash",
        "",
        "300.00",
        "Expense",
        "Food",
    );

    let aug_view = db_conn
        .get_summary_with_str("August", "2024", FetchNature::Monthly)
        .unwrap();
    let aug_full = aug_view.generate_summary(None, &db_conn);

    let sep_view = db_conn
        .get_summary_with_str("September", "2024", FetchNature::Monthly)
        .unwrap();
    let sep_full = sep_view.generate_summary(Some(&aug_full), &db_conn);

    // Net should have MoM comparison strings (↑/↓ with percentages)
    let net = sep_full.net_array();
    assert!(
        net[0].len() >= 7,
        "Net array should have MoM columns, got len {}",
        net[0].len()
    );

    // Methods should have MoM comparison
    let methods = sep_full.method_array();
    let cash_row = methods.iter().find(|r| r[0] == "Cash").unwrap();
    assert!(cash_row.len() >= 7, "Method row should have MoM columns");

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn summary_lend_borrows_with_mom() {
    let file_name = "test_summary_lb_mom.sqlite";
    let mut db_conn = create_test_db(file_name);

    // August: borrow 500
    add_tx(
        &mut db_conn,
        "2024-08-01",
        "Borrow",
        "Cash",
        "",
        "500.00",
        "Borrow",
        "Loan",
    );

    let aug_view = db_conn
        .get_summary_with_str("August", "2024", FetchNature::Monthly)
        .unwrap();
    let aug_full = aug_view.generate_summary(None, &db_conn);

    // September: borrow more, repay some
    add_tx(
        &mut db_conn,
        "2024-09-01",
        "Borrow more",
        "Cash",
        "",
        "300.00",
        "Borrow",
        "Loan",
    );
    add_tx(
        &mut db_conn,
        "2024-09-15",
        "Repay",
        "Cash",
        "",
        "200.00",
        "Borrow Repay",
        "Loan",
    );

    let sep_view = db_conn
        .get_summary_with_str("September", "2024", FetchNature::Monthly)
        .unwrap();
    let sep_full = sep_view.generate_summary(Some(&aug_full), &db_conn);

    // Lend borrows should have MoM columns
    let lb = sep_full.lend_borrows_array();
    assert!(
        lb[0].len() >= 2,
        "Lend borrows array should have value columns"
    );

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn summary_all_fetch_nature() {
    let file_name = "test_summary_all.sqlite";
    let mut db_conn = create_test_db(file_name);

    add_tx(
        &mut db_conn,
        "2024-01-15",
        "Jan income",
        "Cash",
        "",
        "1000.00",
        "Income",
        "Work",
    );
    add_tx(
        &mut db_conn,
        "2024-06-20",
        "Jun expense",
        "Cash",
        "",
        "200.00",
        "Expense",
        "Food",
    );

    let summary_view = db_conn
        .get_summary_with_str("January", "2024", FetchNature::All)
        .unwrap();
    let full = summary_view.generate_summary(None, &db_conn);

    let net = full.net_array();
    assert_eq!(net[0][1], "1000.00"); // total income
    assert_eq!(net[0][2], "200.00"); // total expense

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}
