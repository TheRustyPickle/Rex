extern crate rex;
use chrono::{naive::NaiveDate, Duration};
use rex::db::*;
use rusqlite::{Connection, Result as sqlResult};
use std::collections::HashMap;
use std::fs;

fn create_test_db(file_name: &str) -> Connection {
    create_db(file_name, vec!["test1".to_string(), "test 2".to_string()]).unwrap();
    Connection::open(file_name).unwrap()
}

#[test]
fn check_last_balances_1() {
    let file_name = "last_balances_1.sqlite";
    let conn = create_test_db(file_name);
    let tx_methods = get_all_tx_methods(&conn);
    let data = get_last_balances(&conn, &tx_methods);
    let expected_data = vec!["0.00".to_string(), "0.00".to_string()];
    conn.close().unwrap();

    fs::remove_file(file_name).unwrap();

    assert_eq!(data, expected_data);
}

#[test]
fn check_last_balances_2() {
    let file_name = "last_balances_2.sqlite";
    let conn = create_test_db(file_name);
    let tx_methods = get_all_tx_methods(&conn);

    add_new_tx(
        "2022-07-19",
        "Testing transaction",
        "test1",
        "159.00",
        "Expense",
        file_name,
        None,
    )
    .unwrap();

    add_new_tx(
        "2022-07-19",
        "Testing transaction",
        "test 2",
        "159.19",
        "Income",
        file_name,
        None,
    )
    .unwrap();

    let data = get_last_balances(&conn, &tx_methods);
    let expected_data = vec!["-159.00".to_string(), "159.19".to_string()];

    delete_tx(1, file_name).unwrap();

    let data_2 = get_last_balances(&conn, &tx_methods);
    let expected_data_2 = vec!["0.00".to_string(), "159.19".to_string()];

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();

    assert_eq!(data, expected_data);
    assert_eq!(data_2, expected_data_2);
}

#[test]
fn check_last_month_balance_1() {
    let file_name = "last_month_balance_1.sqlite".to_string();
    let conn = create_test_db(&file_name);
    let tx_methods = get_all_tx_methods(&conn);

    let data = get_last_time_balance(&conn, 6, 1, &tx_methods);
    let expected_data = HashMap::from([("test1".to_string(), 0.0), ("test 2".to_string(), 0.0)]);

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();

    assert_eq!(data, expected_data);
}

#[test]
fn check_last_balance_id() {
    let file_name = "last_balance_id.sqlite".to_string();
    let conn = create_test_db(&file_name);

    let data = get_last_balance_id(&conn);
    let expected_data: sqlResult<i32> = Ok(49);

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();

    assert_eq!(data, expected_data);
}

#[test]
fn check_last_month_balance_2() {
    let file_name = "last_month_balance_2.sqlite".to_string();
    let conn = create_test_db(&file_name);
    let tx_methods = get_all_tx_methods(&conn);

    add_new_tx(
        "2022-07-19",
        "Testing transaction",
        "test1",
        "100.00",
        "Income",
        &file_name,
        None,
    )
    .unwrap();

    add_new_tx(
        "2022-07-19",
        "Testing transaction",
        "test 2",
        "100.00",
        "Income",
        &file_name,
        None,
    )
    .unwrap();

    add_new_tx(
        "2022-08-19",
        "Testing transaction",
        "test1",
        "100.00",
        "Income",
        &file_name,
        None,
    )
    .unwrap();

    add_new_tx(
        "2022-09-19",
        "Testing transaction",
        "test1",
        "100.00",
        "Income",
        &file_name,
        None,
    )
    .unwrap();

    add_new_tx(
        "2022-10-19",
        "Testing transaction",
        "test1",
        "100.00",
        "Income",
        &file_name,
        None,
    )
    .unwrap();

    let data_1 = get_last_time_balance(&conn, 8, 0, &tx_methods);
    let expected_data_1 =
        HashMap::from([("test 2".to_string(), 100.0), ("test1".to_string(), 200.0)]);

    delete_tx(1, &file_name).unwrap();
    delete_tx(2, &file_name).unwrap();

    let data_2 = get_last_time_balance(&conn, 10, 3, &tx_methods);
    let expected_data_2 =
        HashMap::from([("test 2".to_string(), 0.0), ("test1".to_string(), 300.0)]);

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();

    assert_eq!(data_1, expected_data_1);
    assert_eq!(data_2, expected_data_2);
}

#[test]
#[ignore]
fn check_balance_all_day() {
    let file_name = "check_balance_all_day.sqlite".to_string();
    let conn = create_test_db(&file_name);
    let tx_methods = get_all_tx_methods(&conn);

    let mut current_date = NaiveDate::parse_from_str("2022-01-01", "%Y-%m-%d").unwrap();
    let ending_date = NaiveDate::parse_from_str("2025-12-31", "%Y-%m-%d").unwrap();

    let total_days: f64 = (ending_date - current_date).num_days() as f64 + 1.0;

    let details = "Test Transaction";
    let amount = "1.0";
    let tx_method = "test1";
    let tx_type = "Income";

    loop {
        if current_date == ending_date + Duration::days(1) {
            break;
        }
        add_new_tx(
            &current_date.to_string(),
            details,
            tx_method,
            amount,
            tx_type,
            &file_name,
            None,
        )
        .unwrap();
        current_date += Duration::days(1)
    }

    let data = get_last_balances(&conn, &tx_methods);
    let expected = vec![format!("{total_days:.2}"), "0.00".to_string()];
    assert_eq!(data, expected);

    let mut delete_id_num = total_days as usize;

    loop {
        if delete_id_num == 0 {
            break;
        }
        delete_tx(delete_id_num, &file_name).unwrap();
        delete_id_num -= 1;
    }

    let data_1 = get_last_balances(&conn, &tx_methods);
    let data_2 = get_last_time_balance(&conn, 12, 3, &tx_methods);

    let expected_data_1 = vec!["0.00".to_string(), "0.00".to_string()];
    let mut expected_data_2 = HashMap::new();
    for (i, _x) in &data_2 {
        expected_data_2.insert(i.to_string(), 0.0);
    }

    fs::remove_file(file_name).unwrap();

    assert_eq!(data_1, expected_data_1);
    assert_eq!(data_2, expected_data_2);
}
