extern crate rex_tui;
use rex_tui::db::*;
use rex_tui::home_page::TransactionData;
use rex_tui::tx_handler::add_tx;
use rex_tui::utility::get_all_txs;
use rusqlite::Connection;
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

fn add_dummy_tx(conn: &mut Connection) {
    add_tx(
        "2022-08-19",
        "Testing transaction",
        "test1",
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
        "test 2",
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
        "test1",
        "200.00",
        "Income",
        "Food",
        None,
        conn,
    )
    .unwrap();
}

#[test]
fn test_home_data() {
    let file_name = "home_data_1.sqlite";
    let mut conn = create_test_db(file_name);
    add_dummy_tx(&mut conn);

    let tx_data_1 = TransactionData::new(1, 1, &conn);
    let tx_data_2 = TransactionData::new(6, 1, &conn);

    let is_tx_empty_1 = tx_data_1.is_tx_empty();
    let is_tx_empty_2 = tx_data_2.is_tx_empty();

    assert!(is_tx_empty_1);
    assert!(!is_tx_empty_2);

    let all_tx_1 = tx_data_1.get_txs();
    let all_tx_2 = tx_data_2.get_txs();

    let expected_data_1: Vec<Vec<String>> = Vec::new();

    let tx_1 = vec![
        "19-07-2023",
        "Testing transaction",
        "test 2",
        "100.00",
        "Expense",
        "Food",
    ]
    .into_iter()
    .map(std::string::ToString::to_string)
    .collect();
    let tx_2 = vec![
        "25-07-2023",
        "Testing transaction",
        "test1",
        "200.00",
        "Income",
        "Food",
    ]
    .into_iter()
    .map(std::string::ToString::to_string)
    .collect();

    let expected_data_2: Vec<Vec<String>> = vec![tx_1, tx_2];

    assert_eq!(all_tx_1, expected_data_1);
    assert_eq!(all_tx_2, expected_data_2);

    let all_changes_1 = tx_data_2.get_changes(0);
    let all_changes_2 = tx_data_2.get_changes(1);
    let all_balance_1 = tx_data_2.get_balance(0);
    let all_balance_2 = tx_data_2.get_balance(1);

    // Tx method changes for that 1 tx
    let expected_changes_1: Vec<String> = vec!["Changes", "0.00", "↓100.00"]
        .into_iter()
        .map(std::string::ToString::to_string)
        .collect();
    let expected_changes_2: Vec<String> = vec!["Changes", "↑200.00", "0.00"]
        .into_iter()
        .map(std::string::ToString::to_string)
        .collect();

    // Balances are that tx was counted + the total balance combining the first two
    let expected_balance_1: Vec<String> = vec!["Balance", "-100.00", "-100.00", "-200.00"]
        .into_iter()
        .map(std::string::ToString::to_string)
        .collect();
    let expected_balance_2: Vec<String> = vec!["Balance", "100.00", "-100.00", "0.00"]
        .into_iter()
        .map(std::string::ToString::to_string)
        .collect();

    assert_eq!(all_balance_1, expected_balance_1);
    assert_eq!(all_changes_1, expected_changes_1);
    assert_eq!(all_balance_2, expected_balance_2);
    assert_eq!(all_changes_2, expected_changes_2);

    let last_balance_1 = tx_data_1.get_last_balance(&conn);
    let last_balance_2 = tx_data_2.get_last_balance(&conn);

    // Regardless of the month of the TransactionData, the last balance will be the same
    let expected_data: Vec<_> = vec!["Balance", "100.00", "-100.00", "0.00"]
        .into_iter()
        .map(std::string::ToString::to_string)
        .collect();

    assert_eq!(last_balance_1, expected_data);
    assert_eq!(last_balance_2, expected_data);

    // Id num is incremented by 1 as txs are added
    let id_num_1 = tx_data_2.get_id_num(0);
    let id_num_2 = tx_data_2.get_id_num(1);

    assert_eq!(id_num_1, 2);
    assert_eq!(id_num_2, 3);

    let total_income_1 = tx_data_1.get_total_income(None, &conn);
    let total_income_2 = tx_data_2.get_total_income(None, &conn);

    // No tx available within the selected index so 0 balance
    let expected_data_1: Vec<String> = vec!["Income", "0.00", "0.00", "0.00"]
        .into_iter()
        .map(std::string::ToString::to_string)
        .collect();

    let expected_data_2: Vec<String> = vec!["Income", "200.00", "0.00", "200.00"]
        .into_iter()
        .map(std::string::ToString::to_string)
        .collect();

    assert_eq!(total_income_1, expected_data_1);
    assert_eq!(total_income_2, expected_data_2);

    let total_expense_1 = tx_data_1.get_total_expense(None, &conn);
    let total_expense_2 = tx_data_2.get_total_expense(None, &conn);

    // No tx available within the selected index so 0
    let expected_data_1: Vec<String> = vec!["Expense", "0.00", "0.00", "0.00"]
        .into_iter()
        .map(std::string::ToString::to_string)
        .collect();

    let expected_data_2: Vec<String> = vec!["Expense", "0.00", "100.00", "100.00"]
        .into_iter()
        .map(std::string::ToString::to_string)
        .collect();

    assert_eq!(total_expense_1, expected_data_1);
    assert_eq!(total_expense_2, expected_data_2);

    tx_data_2.del_tx(0, &mut conn).unwrap();

    let tx_data_2 = TransactionData::new(6, 1, &conn);

    assert!(!tx_data_2.is_tx_empty());
    assert_eq!(tx_data_2.get_txs().len(), 1);

    let txs = get_all_txs(&conn, 6, 1);
    let tx_data = TransactionData::new_search(txs.0, txs.2);

    assert_eq!(tx_data.get_txs().len(), 1);

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();
}
