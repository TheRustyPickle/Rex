extern crate rex_tui;
use rex_tui::db::*;
use rex_tui::page_handler::{IndexedData, SortingType};
use rex_tui::summary_page::SummaryData;
use rex_tui::tx_handler::add_tx;
use rex_tui::utility::sort_table_data;
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

#[test]
fn check_summary_data_1() {
    let file_name = "summary_data_1.sqlite";
    let mut conn = create_test_db(file_name);

    add_tx(
        "2022-08-19",
        "Testing transaction",
        "test1",
        "100.00",
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
        "100.00",
        "Expense",
        "Food",
        None,
        &mut conn,
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
        &mut conn,
    )
    .unwrap();

    let summary_modes = IndexedData::new_modes();

    let my_summary = SummaryData::new(&conn);
    let my_summary_text = my_summary.get_table_data(&summary_modes, 6, 1);
    let my_summary_text_2 = my_summary.get_tx_data(&summary_modes, 6, 1, &conn);

    let expected_data_1 = vec![vec!["Food", "200.00", "100.00", "100.00", "100.00"]];

    let expected_data_2 = (
        vec![
            vec![
                "Total Income".to_string(),
                "200.00".to_string(),
                "66.67%".to_string(),
            ],
            vec![
                "Total Expense".to_string(),
                "100.00".to_string(),
                "33.33%".to_string(),
            ],
            vec!["Net".to_string(), "100.00".to_string(), "-".to_string()],
        ],
        vec![
            vec![
                "Average Income".to_string(),
                "200.00".to_string(),
                "-".to_string(),
            ],
            vec![
                "Average Expense".to_string(),
                "100.00".to_string(),
                "-".to_string(),
            ],
        ],
        vec![
            vec![
                "Largest Income".to_string(),
                "25-07-2023".to_string(),
                "200.00".to_string(),
                "test1".to_string(),
            ],
            vec![
                "Largest Expense".to_string(),
                "19-07-2023".to_string(),
                "100.00".to_string(),
                "test 2".to_string(),
            ],
            vec![
                "Months Checked".to_string(),
                "1".to_string(),
                "-".to_string(),
                "-".to_string(),
            ],
        ],
        vec![
            vec![
                "Peak Earning".to_string(),
                "07-2023".to_string(),
                "200.00".to_string(),
                "-".to_string(),
            ],
            vec![
                "Peak Expense".to_string(),
                "07-2023".to_string(),
                "100.00".to_string(),
                "-".to_string(),
            ],
        ],
        vec![
            vec![
                "test1".to_string(),
                "200.00".to_string(),
                "0.00".to_string(),
                "100.00%".to_string(),
                "0.00".to_string(),
                "200.00".to_string(),
                "0.00".to_string(),
            ],
            vec![
                "test 2".to_string(),
                "0.00".to_string(),
                "100.00".to_string(),
                "0.00".to_string(),
                "100.00%".to_string(),
                "0.00".to_string(),
                "100.00".to_string(),
            ],
        ],
    );

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();

    assert_eq!(my_summary_text, expected_data_1);
    assert_eq!(my_summary_text_2, expected_data_2);
}

#[test]
fn check_summary_data_2() {
    let file_name = "summary_data_2.sqlite";
    let mut conn = create_test_db(file_name);

    add_tx(
        "2022-08-19",
        "Testing transaction",
        "test1",
        "500.00",
        "Expense",
        "Car",
        None,
        &mut conn,
    )
    .unwrap();

    add_tx(
        "2022-01-19",
        "Testing transaction",
        "test1",
        "500.00",
        "Expense",
        "Car",
        None,
        &mut conn,
    )
    .unwrap();

    add_tx(
        "2022-07-19",
        "Testing transaction",
        "test 2",
        "700.00",
        "Income",
        "Food",
        None,
        &mut conn,
    )
    .unwrap();

    add_tx(
        "2022-05-19",
        "Testing transaction",
        "test1",
        "1000.00",
        "Income",
        "Food",
        None,
        &mut conn,
    )
    .unwrap();

    let mut summary_modes = IndexedData::new_modes();
    summary_modes.next();

    let my_summary = SummaryData::new(&conn);
    let my_summary_text = my_summary.get_table_data(&summary_modes, 0, 0);
    let my_summary_text_2 = my_summary.get_tx_data(&summary_modes, 0, 0, &conn);

    let expected_data_1 = vec![
        vec![
            "Car".to_string(),
            "0.00".to_string(),
            "1000.00".to_string(),
            "0.00".to_string(),
            "100.00".to_string(),
        ],
        vec![
            "Food".to_string(),
            "1700.00".to_string(),
            "0.00".to_string(),
            "100.00".to_string(),
            "0.00".to_string(),
        ],
    ];

    let expected_data_2 = (
        vec![
            vec![
                "Total Income".to_string(),
                "1700.00".to_string(),
                "62.96%".to_string(),
            ],
            vec![
                "Total Expense".to_string(),
                "1000.00".to_string(),
                "37.04%".to_string(),
            ],
            vec!["Net".to_string(), "700.00".to_string(), "-".to_string()],
        ],
        vec![
            vec![
                "Average Income".to_string(),
                "425.00".to_string(),
                "-".to_string(),
            ],
            vec![
                "Average Expense".to_string(),
                "250.00".to_string(),
                "-".to_string(),
            ],
        ],
        vec![
            vec![
                "Largest Income".to_string(),
                "19-05-2022".to_string(),
                "1000.00".to_string(),
                "test1".to_string(),
            ],
            vec![
                "Largest Expense".to_string(),
                "19-01-2022".to_string(),
                "500.00".to_string(),
                "test1".to_string(),
            ],
            vec![
                "Months Checked".to_string(),
                "4".to_string(),
                "-".to_string(),
                "-".to_string(),
            ],
        ],
        vec![
            vec![
                "Peak Earning".to_string(),
                "05-2022".to_string(),
                "1000.00".to_string(),
                "-".to_string(),
            ],
            vec![
                "Peak Expense".to_string(),
                "01-2022".to_string(),
                "500.00".to_string(),
                "-".to_string(),
            ],
        ],
        vec![
            vec![
                "test1".to_string(),
                "1000.00".to_string(),
                "1000.00".to_string(),
                "58.82%".to_string(),
                "100.00%".to_string(),
                "250.00".to_string(),
                "250.00".to_string(),
            ],
            vec![
                "test 2".to_string(),
                "700.00".to_string(),
                "0.00".to_string(),
                "41.18%".to_string(),
                "0.00".to_string(),
                "175.00".to_string(),
                "0.00".to_string(),
            ],
        ],
    );

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();

    assert_eq!(my_summary_text, expected_data_1);
    assert_eq!(my_summary_text_2, expected_data_2);
}

#[test]
fn check_summary_data_3() {
    let file_name = "summary_data_3.sqlite";
    let mut conn = create_test_db(file_name);

    add_tx(
        "2022-08-19",
        "Testing transaction",
        "test1",
        "100.00",
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
        "100.00",
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
        "100.00",
        "Income",
        "Food",
        None,
        &mut conn,
    )
    .unwrap();

    let mut summary_modes = IndexedData::new_modes();
    summary_modes.next();
    summary_modes.next();

    let my_summary = SummaryData::new(&conn);
    let my_summary_text = my_summary.get_table_data(&summary_modes, 0, 1);
    let my_summary_text_2 = my_summary.get_tx_data(&summary_modes, 0, 1, &conn);

    let expected_data_1 = vec![
        vec![
            "Car".to_string(),
            "0.00".to_string(),
            "100.00".to_string(),
            "0.00".to_string(),
            "100.00".to_string(),
        ],
        vec![
            "Food".to_string(),
            "200.00".to_string(),
            "0.00".to_string(),
            "100.00".to_string(),
            "0.00".to_string(),
        ],
    ];

    let expected_data_2 = (
        vec![
            vec![
                "Total Income".to_string(),
                "200.00".to_string(),
                "66.67%".to_string(),
            ],
            vec![
                "Total Expense".to_string(),
                "100.00".to_string(),
                "33.33%".to_string(),
            ],
            vec!["Net".to_string(), "100.00".to_string(), "-".to_string()],
        ],
        vec![
            vec![
                "Average Income".to_string(),
                "66.67".to_string(),
                "-".to_string(),
            ],
            vec![
                "Average Expense".to_string(),
                "33.33".to_string(),
                "-".to_string(),
            ],
        ],
        vec![
            vec![
                "Largest Income".to_string(),
                "19-07-2023".to_string(),
                "100.00".to_string(),
                "test 2".to_string(),
            ],
            vec![
                "Largest Expense".to_string(),
                "19-08-2022".to_string(),
                "100.00".to_string(),
                "test1".to_string(),
            ],
            vec![
                "Months Checked".to_string(),
                "3".to_string(),
                "-".to_string(),
                "-".to_string(),
            ],
        ],
        vec![
            vec![
                "Peak Earning".to_string(),
                "07-2023".to_string(),
                "100.00".to_string(),
                "-".to_string(),
            ],
            vec![
                "Peak Expense".to_string(),
                "08-2022".to_string(),
                "100.00".to_string(),
                "-".to_string(),
            ],
        ],
        vec![
            vec![
                "test1".to_string(),
                "100.00".to_string(),
                "100.00".to_string(),
                "50.00%".to_string(),
                "100.00%".to_string(),
                "33.33".to_string(),
                "33.33".to_string(),
            ],
            vec![
                "test 2".to_string(),
                "100.00".to_string(),
                "0.00".to_string(),
                "50.00%".to_string(),
                "0.00".to_string(),
                "33.33".to_string(),
                "0.00".to_string(),
            ],
        ],
    );

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();

    assert_eq!(my_summary_text, expected_data_1);
    assert_eq!(my_summary_text_2, expected_data_2);
}

#[test]
fn check_summary_sorting() {
    let file_name = "summary_sorting.sqlite";
    let mut conn = create_test_db(file_name);

    add_tx(
        "2022-08-19",
        "Testing transaction",
        "test1",
        "1000.00",
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
        "500.00",
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
        "2000.00",
        "Income",
        "Bank",
        None,
        &mut conn,
    )
    .unwrap();

    let mut summary_modes = IndexedData::new_modes();
    summary_modes.next();
    summary_modes.next();

    let my_summary = SummaryData::new(&conn);
    let table_data = my_summary.get_table_data(&summary_modes, 0, 0);

    let sorted_data_1 = sort_table_data(table_data.clone(), &SortingType::ByTags);
    let sorted_data_2 = sort_table_data(table_data.clone(), &SortingType::ByIncome);
    let sorted_data_3 = sort_table_data(table_data.clone(), &SortingType::ByExpense);

    let expected_data_1 = vec![
        ["Bank", "2000.00", "0.00", "80.00", "0.00"]
            .iter()
            .map(|s| (*s).to_string())
            .collect::<Vec<String>>(),
        ["Car", "0.00", "1000.00", "0.00", "100.00"]
            .iter()
            .map(|s| (*s).to_string())
            .collect::<Vec<String>>(),
        ["Food", "500.00", "0.00", "20.00", "0.00"]
            .iter()
            .map(|s| (*s).to_string())
            .collect::<Vec<String>>(),
    ];

    let expected_data_2 = vec![
        ["Bank", "2000.00", "0.00", "80.00", "0.00"]
            .iter()
            .map(|s| (*s).to_string())
            .collect::<Vec<String>>(),
        ["Food", "500.00", "0.00", "20.00", "0.00"]
            .iter()
            .map(|s| (*s).to_string())
            .collect::<Vec<String>>(),
        ["Car", "0.00", "1000.00", "0.00", "100.00"]
            .iter()
            .map(|s| (*s).to_string())
            .collect::<Vec<String>>(),
    ];

    let expected_data_3 = vec![
        ["Car", "0.00", "1000.00", "0.00", "100.00"]
            .iter()
            .map(|s| (*s).to_string())
            .collect::<Vec<String>>(),
        ["Bank", "2000.00", "0.00", "80.00", "0.00"]
            .iter()
            .map(|s| (*s).to_string())
            .collect::<Vec<String>>(),
        ["Food", "500.00", "0.00", "20.00", "0.00"]
            .iter()
            .map(|s| (*s).to_string())
            .collect::<Vec<String>>(),
    ];

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();

    assert_eq!(sorted_data_1, expected_data_1);
    assert_eq!(sorted_data_2, expected_data_2);
    assert_eq!(sorted_data_3, expected_data_3);
}
