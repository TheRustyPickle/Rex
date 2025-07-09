extern crate rex_tui;
use chrono::NaiveDate;
use rex_tui::chart_page::ChartData;
use rex_tui::page_handler::IndexedData;
use rex_tui::tx_handler::add_tx;
use rusqlite::Connection;
use std::fs;

mod common;

use crate::common::create_test_db;

fn add_dummy_tx(conn: &mut Connection) {
    add_tx(
        "2022-08-19",
        "Testing transaction",
        "Super Special Bank",
        "100.00",
        "Expense",
        "Car",
        None,
        conn,
    )
    .unwrap();

    add_tx(
        "2023-07-19",
        "Testing transaction",
        "Cash Cow",
        "100.00",
        "Expense",
        "Food",
        None,
        conn,
    )
    .unwrap();

    add_tx(
        "2023-07-25",
        "Testing transaction",
        "Super Special Bank",
        "200.00",
        "Income",
        "Food",
        None,
        conn,
    )
    .unwrap();
}

#[test]
fn check_chart_date() {
    let file_name = "chart_data_1.sqlite";
    let mut conn = create_test_db(file_name);
    add_dummy_tx(&mut conn);

    let chart_data = ChartData::new(&conn);

    let mut chart_mode = IndexedData::new_modes();

    let chart_dates_1 = chart_data.get_all_dates(&chart_mode, 1, 1);
    let chart_dates_2 = chart_data.get_all_dates(&chart_mode, 6, 1);

    let expected_data_1 = Vec::new();
    let expected_data_2 = vec![
        NaiveDate::from_ymd_opt(2023, 7, 19).unwrap(),
        NaiveDate::from_ymd_opt(2023, 7, 25).unwrap(),
    ];

    assert_eq!(chart_dates_1, expected_data_1);
    assert_eq!(chart_dates_2, expected_data_2);

    chart_mode.next();

    let chart_dates_1 = chart_data.get_all_dates(&chart_mode, 1, 1);
    let chart_dates_2 = chart_data.get_all_dates(&chart_mode, 6, 1);

    let expected_data_1 = vec![
        NaiveDate::from_ymd_opt(2023, 7, 19).unwrap(),
        NaiveDate::from_ymd_opt(2023, 7, 25).unwrap(),
    ];
    let expected_data_2 = vec![
        NaiveDate::from_ymd_opt(2023, 7, 19).unwrap(),
        NaiveDate::from_ymd_opt(2023, 7, 25).unwrap(),
    ];

    assert_eq!(chart_dates_1, expected_data_1);
    assert_eq!(chart_dates_2, expected_data_2);

    chart_mode.next();

    let chart_dates_1 = chart_data.get_all_dates(&chart_mode, 1, 1);
    let chart_dates_2 = chart_data.get_all_dates(&chart_mode, 6, 1);

    let expected_data_1 = vec![
        NaiveDate::from_ymd_opt(2022, 8, 19).unwrap(),
        NaiveDate::from_ymd_opt(2023, 7, 19).unwrap(),
        NaiveDate::from_ymd_opt(2023, 7, 25).unwrap(),
    ];
    let expected_data_2 = vec![
        NaiveDate::from_ymd_opt(2022, 8, 19).unwrap(),
        NaiveDate::from_ymd_opt(2023, 7, 19).unwrap(),
        NaiveDate::from_ymd_opt(2023, 7, 25).unwrap(),
    ];

    assert_eq!(chart_dates_1, expected_data_1);
    assert_eq!(chart_dates_2, expected_data_2);

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();
}

#[test]
fn check_chart_data() {
    let file_name = "chart_data_2.sqlite";
    let mut conn = create_test_db(file_name);
    add_dummy_tx(&mut conn);

    let chart_data = ChartData::new(&conn);

    let mut chart_mode = IndexedData::new_modes();

    let first_tx = [
        "19-08-2022",
        "Testing transaction",
        "Super Special Bank",
        "100.00",
        "Expense",
        "Car",
    ]
    .map(std::string::ToString::to_string)
    .into_iter()
    .collect();

    let second_tx = [
        "19-07-2023",
        "Testing transaction",
        "Cash Cow",
        "100.00",
        "Expense",
        "Food",
    ]
    .map(std::string::ToString::to_string)
    .into_iter()
    .collect();

    let third_tx = [
        "25-07-2023",
        "Testing transaction",
        "Super Special Bank",
        "200.00",
        "Income",
        "Food",
    ]
    .map(std::string::ToString::to_string)
    .into_iter()
    .collect();

    let chart_data_1 = chart_data.get_data(&chart_mode, 1, 1);
    let chart_dates_2 = chart_data.get_data(&chart_mode, 6, 1);

    let expected_data_1 = (Vec::new(), Vec::new());

    let first_balance = ["-100.00", "-100.00"]
        .map(std::string::ToString::to_string)
        .into_iter()
        .collect();
    let second_balance = ["100.00", "-100.00"]
        .map(std::string::ToString::to_string)
        .into_iter()
        .collect();

    let expected_data_2 = (
        vec![&second_tx, &third_tx],
        vec![&first_balance, &second_balance],
    );

    assert_eq!(chart_data_1, expected_data_1);
    assert_eq!(chart_dates_2, expected_data_2);

    chart_mode.next();

    let chart_data_1 = chart_data.get_data(&chart_mode, 1, 1);
    let chart_dates_2 = chart_data.get_data(&chart_mode, 6, 1);

    let expected_data_1 = (
        vec![&second_tx, &third_tx],
        vec![&first_balance, &second_balance],
    );

    let expected_data_2 = (
        vec![&second_tx, &third_tx],
        vec![&first_balance, &second_balance],
    );

    assert_eq!(chart_data_1, expected_data_1);
    assert_eq!(chart_dates_2, expected_data_2);

    chart_mode.next();

    let chart_data_1 = chart_data.get_data(&chart_mode, 1, 1);
    let chart_dates_2 = chart_data.get_data(&chart_mode, 6, 1);

    let first_balance = ["-100.00", "0.00"]
        .map(std::string::ToString::to_string)
        .into_iter()
        .collect();
    let second_balance = ["-100.00", "-100.00"]
        .map(std::string::ToString::to_string)
        .into_iter()
        .collect();

    let third_balance = ["100.00", "-100.00"]
        .map(std::string::ToString::to_string)
        .into_iter()
        .collect();

    let expected_data_1 = (
        vec![&first_tx, &second_tx, &third_tx],
        vec![&first_balance, &second_balance, &third_balance],
    );

    let expected_data_2 = (
        vec![&first_tx, &second_tx, &third_tx],
        vec![&first_balance, &second_balance, &third_balance],
    );

    assert_eq!(chart_data_1, expected_data_1);
    assert_eq!(chart_dates_2, expected_data_2);

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();
}
