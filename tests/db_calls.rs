extern crate rex;
use rex::db::*;
use rusqlite::Connection;
use std::fs;

fn create_test_db(file_name: &str) -> Connection {
    create_db(file_name, vec!["test1".to_string(), "test 2".to_string()]).unwrap();
    return Connection::open(file_name).unwrap();
}

#[test]
fn check_getting_tx_methods_1() {
    let file_name = "getting_tx_methods_1.sqlite";
    let conn = create_test_db(file_name);
    let data = get_all_tx_methods(&conn);
    conn.close().unwrap();

    fs::remove_file(file_name).unwrap();

    assert_eq!(data, vec!["test1".to_string(), "test 2".to_string()]);
}

#[test]
fn check_getting_tx_methods_2() {
    let file_name = "getting_tx_methods_2.sqlite";
    let conn = create_test_db(file_name);

    add_new_tx_methods(
        file_name,
        vec!["new method 1".to_string(), "testing methods".to_string()],
    )
    .unwrap();

    let data = get_all_tx_methods(&conn);
    conn.close().unwrap();

    fs::remove_file(file_name).unwrap();

    assert_eq!(
        data,
        vec![
            "test1".to_string(),
            "test 2".to_string(),
            "new method 1".to_string(),
            "testing methods".to_string()
        ]
    );
}

#[test]
fn check_empty_changes() {
    let file_name = "empty_changes.sqlite";
    let conn = create_test_db(file_name);
    let data = get_empty_changes(&conn);
    conn.close().unwrap();

    fs::remove_file(file_name).unwrap();

    assert_eq!(
        data,
        vec![
            "Changes".to_string(),
            "0.00".to_string(),
            "0.00".to_string()
        ]
    );
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
        &file_name,
    )
    .unwrap();

    add_new_tx(
        "2022-07-19",
        "Testing transaction",
        "test 2",
        "159.19",
        "Income",
        &file_name,
    )
    .unwrap();

    let data = get_last_balances(&conn, &tx_methods);
    let expected_data = vec!["-159.00".to_string(), "159.19".to_string()];

    delete_tx(1, &file_name).unwrap();

    let data_2 = get_last_balances(&conn, &tx_methods);
    let expected_data_2 = vec!["0.00".to_string(), "159.19".to_string()];

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();

    assert_eq!(data, expected_data);
    assert_eq!(data_2, expected_data_2);
}

#[test]
fn check_getting_all_changes() {
    let file_name = "getting_changes_1.sqlite";
    let conn = create_test_db(file_name);
    let data = get_all_changes(&conn, 5, 6);
    let empty_data: Vec<Vec<String>> = Vec::new();

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();

    assert_eq!(data, empty_data);
}

#[test]
fn check_getting_all_changes_2() {
    let file_name = "getting_changes_2.sqlite".to_string();
    let conn = create_test_db(&file_name);

    add_new_tx(
        "2022-07-19",
        "Testing transaction",
        "test1",
        "159.00",
        "Expense",
        &file_name,
    )
    .unwrap();

    add_new_tx(
        "2022-07-19",
        "Testing transaction",
        "test 2",
        "159.00",
        "Expense",
        &file_name,
    )
    .unwrap();

    add_new_tx(
        "2022-05-01",
        "Testing transaction",
        "test 2",
        "753.00",
        "Expense",
        &file_name,
    )
    .unwrap();

    // This is the index of the interface. year 0 = 2022, month 0 = January
    let data_1 = get_all_changes(&conn, 6, 0);
    let expected_data_1: Vec<Vec<String>> = vec![
        vec!["↓159.00".to_string(), "0.00".to_string()],
        vec!["0.00".to_string(), "↓159.00".to_string()],
    ];

    let another_data = get_all_changes(&conn, 4, 0);

    let another_expected = vec![vec!["0.00".to_string(), "↓753.00".to_string()]];

    delete_tx(2, &file_name).unwrap();

    let data_2 = get_all_changes(&conn, 6, 0);
    let expected_data_2: Vec<Vec<String>> = vec![vec!["↓159.00".to_string(), "0.00".to_string()]];

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();

    assert_eq!(data_1, expected_data_1);
    assert_eq!(data_2, expected_data_2);
    assert_eq!(another_data, another_expected);
}

#[test]
fn check_getting_all_tx_1() {
    let file_name = "getting_tx_1.sqlite".to_string();
    let conn = create_test_db(&file_name);

    let data = get_all_txs(&conn, 6, 0);
    let expected_data = (Vec::new(), Vec::new(), Vec::new());

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();

    assert_eq!(data, expected_data);
}

#[test]
fn check_getting_all_tx_2() {
    let file_name = "getting_tx_2.sqlite".to_string();
    let conn = create_test_db(&file_name);

    add_new_tx(
        "2022-07-19",
        "Testing transaction",
        "test1",
        "100.00",
        "Expense",
        &file_name,
    )
    .unwrap();

    add_new_tx(
        "2022-07-19",
        "Testing transaction",
        "test 2",
        "100.00",
        "Expense",
        &file_name,
    )
    .unwrap();

    add_new_tx(
        "2022-05-15",
        "Testing transaction",
        "test 2",
        "100.00",
        "Expense",
        &file_name,
    )
    .unwrap();

    add_new_tx(
        "2022-05-20",
        "Testing transaction",
        "test 2",
        "100.00",
        "Income",
        &file_name,
    )
    .unwrap();

    let data = get_all_txs(&conn, 6, 0);
    let data_2 = get_all_txs(&conn, 4, 0);

    let expected_data = (
        vec![
            vec![
                "19-07-2022".to_string(),
                "Testing transaction".to_string(),
                "test1".to_string(),
                "100.00".to_string(),
                "Expense".to_string(),
            ],
            vec![
                "19-07-2022".to_string(),
                "Testing transaction".to_string(),
                "test 2".to_string(),
                "100.00".to_string(),
                "Expense".to_string(),
            ],
        ],
        vec![
            vec!["-100.00".to_string(), "0.00".to_string()],
            vec!["-100.00".to_string(), "-100.00".to_string()],
        ],
        vec!["1".to_string(), "2".to_string()],
    );

    let expected_data_2 = (
        vec![
            vec![
                "15-05-2022".to_string(),
                "Testing transaction".to_string(),
                "test 2".to_string(),
                "100.00".to_string(),
                "Expense".to_string(),
            ],
            vec![
                "20-05-2022".to_string(),
                "Testing transaction".to_string(),
                "test 2".to_string(),
                "100.00".to_string(),
                "Income".to_string(),
            ],
        ],
        vec![
            vec!["0.00".to_string(), "-100.00".to_string()],
            vec!["0.00".to_string(), "0.00".to_string()],
        ],
        vec!["3".to_string(), "4".to_string()],
    );

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();

    assert_eq!(data, expected_data);
    assert_eq!(data_2, expected_data_2);
}
