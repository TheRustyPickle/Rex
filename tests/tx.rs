extern crate rex_tui;
use rex_tui::db::create_db;
use rex_tui::tx_handler::add_tx;
use rex_tui::tx_handler::delete_tx;
use rex_tui::utility::{get_all_tx_columns, get_all_txs, get_last_tx_id};
use rusqlite::{Connection, Result as sqlResult};
use std::fs;

fn create_test_db(file_name: &str) -> Connection {
    if let Ok(metadata) = fs::metadata(file_name) {
        if metadata.is_file() {
            fs::remove_file(file_name).expect("Failed to delete existing file");
        }
    }

    let mut conn = Connection::open(file_name).unwrap();
    create_db(&["test1".to_string(), "test 2".to_string()], &mut conn).unwrap();
    conn
}

#[test]
fn check_last_tx_id_1() {
    let file_name = "last_tx_id_1.sqlite";
    let conn = create_test_db(file_name);

    let data = get_last_tx_id(&conn);
    let expected_data: sqlResult<i32> = Err(rusqlite::Error::QueryReturnedNoRows);

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();

    assert_eq!(data, expected_data);
}

#[test]
fn check_last_tx_id_2() {
    let file_name = "last_tx_id_2.sqlite";
    let mut conn = create_test_db(file_name);

    add_tx(
        "2022-09-19",
        "Testing transaction",
        "test1",
        "100.00",
        "Income",
        "Unknown",
        None,
        &mut conn,
    )
    .unwrap();

    let data = get_last_tx_id(&conn);
    let expected_data: sqlResult<i32> = Ok(1);

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();

    assert_eq!(data, expected_data);
}

#[test]
fn check_getting_all_tx_1() {
    let file_name = "getting_tx_1.sqlite";
    let conn = create_test_db(file_name);

    let data = get_all_txs(&conn, 6, 0);
    let expected_data = (Vec::new(), Vec::new(), Vec::new());

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();

    assert_eq!(data, expected_data);
}

#[test]
fn check_getting_all_tx_2() {
    let file_name = "getting_tx_2.sqlite";
    let mut conn = create_test_db(file_name);

    add_tx(
        "2022-07-19",
        "Testing transaction",
        "test1",
        "100.00",
        "Expense",
        "Unknown",
        None,
        &mut conn,
    )
    .unwrap();

    add_tx(
        "2022-07-19",
        "Testing transaction",
        "test 2",
        "100.00",
        "Expense",
        "Unknown",
        None,
        &mut conn,
    )
    .unwrap();

    add_tx(
        "2022-05-15",
        "Testing transaction",
        "test 2",
        "100.00",
        "Expense",
        "Unknown",
        None,
        &mut conn,
    )
    .unwrap();

    add_tx(
        "2022-05-20",
        "Testing transaction",
        "test 2",
        "100.00",
        "Income",
        "Unknown",
        None,
        &mut conn,
    )
    .unwrap();

    add_tx(
        "2022-05-25",
        "Testing transfer",
        "test 2 to test1",
        "100.00",
        "Transfer",
        "Unknown",
        None,
        &mut conn,
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
                "Unknown".to_string(),
            ],
            vec![
                "19-07-2022".to_string(),
                "Testing transaction".to_string(),
                "test 2".to_string(),
                "100.00".to_string(),
                "Expense".to_string(),
                "Unknown".to_string(),
            ],
        ],
        vec![
            vec!["0.00".to_string(), "-100.00".to_string()],
            vec!["0.00".to_string(), "-200.00".to_string()],
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
                "Unknown".to_string(),
            ],
            vec![
                "20-05-2022".to_string(),
                "Testing transaction".to_string(),
                "test 2".to_string(),
                "100.00".to_string(),
                "Income".to_string(),
                "Unknown".to_string(),
            ],
            vec![
                "25-05-2022".to_string(),
                "Testing transfer".to_string(),
                "test 2 to test1".to_string(),
                "100.00".to_string(),
                "Transfer".to_string(),
                "Unknown".to_string(),
            ],
        ],
        vec![
            vec!["0.00".to_string(), "-100.00".to_string()],
            vec!["0.00".to_string(), "0.00".to_string()],
            vec!["100.00".to_string(), "-100.00".to_string()],
        ],
        vec!["3".to_string(), "4".to_string(), "5".to_string()],
    );

    delete_tx(5, &mut conn).unwrap();

    add_tx(
        "2022-05-25",
        "Testing transfer",
        "test 2 to test1",
        "500.00",
        "Transfer",
        "Unknown",
        Some("5"),
        &mut conn,
    )
    .unwrap();

    let data_3 = get_all_txs(&conn, 4, 0);

    let expected_data_3 = (
        vec![
            vec![
                "15-05-2022".to_string(),
                "Testing transaction".to_string(),
                "test 2".to_string(),
                "100.00".to_string(),
                "Expense".to_string(),
                "Unknown".to_string(),
            ],
            vec![
                "20-05-2022".to_string(),
                "Testing transaction".to_string(),
                "test 2".to_string(),
                "100.00".to_string(),
                "Income".to_string(),
                "Unknown".to_string(),
            ],
            vec![
                "25-05-2022".to_string(),
                "Testing transfer".to_string(),
                "test 2 to test1".to_string(),
                "500.00".to_string(),
                "Transfer".to_string(),
                "Unknown".to_string(),
            ],
        ],
        vec![
            vec!["0.00".to_string(), "-100.00".to_string()],
            vec!["0.00".to_string(), "0.00".to_string()],
            vec!["500.00".to_string(), "-500.00".to_string()],
        ],
        vec!["3".to_string(), "4".to_string(), "5".to_string()],
    );

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();

    assert_eq!(data, expected_data);
    assert_eq!(data_2, expected_data_2);
    assert_eq!(data_3, expected_data_3);
}

#[test]

fn check_tx_columns() {
    let file_name = "tx_columns.sqlite";
    let conn = create_test_db(file_name);

    let columns = get_all_tx_columns(&conn);
    let expected_data = vec![
        "date".to_string(),
        "details".to_string(),
        "tx_method".to_string(),
        "amount".to_string(),
        "tx_type".to_string(),
        "id_num".to_string(),
        "tags".to_string(),
    ];

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();

    assert_eq!(columns, expected_data);
    assert_eq!(columns.len(), 7);
}
