extern crate rex;
use rex::db::*;
use rex::page_handler::IndexedData;
use rex::summary_page::SummaryData;
use rex::tx_handler::add_tx;
use rusqlite::Connection;
use std::fs;

fn create_test_db(file_name: &str) -> Connection {
    if let Ok(metadata) = fs::metadata(file_name) {
        if metadata.is_file() {
            fs::remove_file(file_name).expect("Failed to delete existing file");
        }
    }

    let mut conn = Connection::open(file_name).unwrap();
    create_db(vec!["test1".to_string(), "test 2".to_string()], &mut conn).unwrap();
    conn
}

#[test]
fn check_summary_data() {
    let file_name = "summary_data.sqlite";
    let mut conn = create_test_db(&file_name);

    add_tx(
        "2022-08-19",
        "Testing transaction",
        "test1",
        "159.00",
        "Expense",
        "Car",
        None,
        &mut conn,
    )
    .unwrap();

    add_tx(
        "2023-07-19",
        "Testing transaction",
        "test 2",
        "159.19",
        "Income",
        "Food",
        None,
        &mut conn,
    )
    .unwrap();

    add_tx(
        "2024-07-19",
        "Testing transaction",
        "test1",
        "159.19",
        "Income",
        "Food",
        None,
        &mut conn,
    )
    .unwrap();

    let mut summary_modes = IndexedData::new_modes();
    summary_modes.next();
    summary_modes.next();

    let my_summary = SummaryData::new(&summary_modes, 0, 0, &conn);
    let my_summary_text = my_summary.get_table_data();
    let my_summary_text_2 = my_summary.get_tx_data();

    let expected_data_1 = vec![
        vec!["Car".to_string(), "0.00".to_string(), "159.00".to_string()],
        vec!["Food".to_string(), "318.38".to_string(), "0.00".to_string()],
    ];

    let expected_data_2 = vec![
        (318.38, "Total Income:".to_string()),
        (159.00, "Total Expense:".to_string()),
        (159.19, "test 2, Date: 19-07-2023".to_string()),
        (159.00, "test1, Date: 19-08-2022".to_string()),
        (159.19, "July of 2023".to_string()),
        (159.00, "August of 2022".to_string()),
    ];

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();

    assert_eq!(my_summary_text, expected_data_1);
    assert_eq!(my_summary_text_2, expected_data_2);
}
