extern crate rex_tui;
use rex_tui::page_handler::{IndexedData, SortingType};
use rex_tui::summary_page::{
    LargestType, PeakType, SummaryData, SummaryLargest, SummaryMethods, SummaryNet, SummaryPeak,
};
use rex_tui::tx_handler::add_tx;
use rex_tui::utility::sort_table_data;
use std::fs;

mod common;

use crate::common::create_test_db;

#[test]
fn check_summary_data_1() {
    let file_name = "summary_data_1.sqlite";
    let mut conn = create_test_db(file_name);

    add_tx(
        "2022-08-19",
        "Testing transaction",
        "Super Special Bank",
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
        "Cash Cow",
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
        "Super Special Bank",
        "200.00",
        "Income",
        "Food",
        None,
        &mut conn,
    )
    .unwrap();

    let summary_modes = IndexedData::new_modes();

    let my_summary = SummaryData::new(&conn);
    let (old_net, _, _, old_methods) =
        my_summary.get_tx_data(&summary_modes, 5, 1, &None, &None, &conn);

    let my_summary_text = my_summary.get_table_data(&summary_modes, 6, 1);
    let (net, largest, peak, methods) = my_summary.get_tx_data(
        &summary_modes,
        6,
        1,
        &Some(old_methods),
        &Some(old_net),
        &conn,
    );

    let expected_data_1 = vec![vec!["Food", "200.00", "100.00", "100.00", "100.00"]];

    let expected_net = SummaryNet::new(
        200.0,
        100.0,
        None,
        None,
        66.66666666666666,
        33.33333333333333,
        Some("∞".to_string()),
        Some("∞".to_string()),
    );
    let expected_largest_1 = SummaryLargest::new(
        LargestType::Earning,
        "Super Special Bank".to_string(),
        200.0,
        "25-07-2023".to_string(),
    );

    let expected_largest_2 = SummaryLargest::new(
        LargestType::Expense,
        "Cash Cow".to_string(),
        100.0,
        "19-07-2023".to_string(),
    );

    let expected_peak_1 = SummaryPeak::new(PeakType::Earning, 200.0, "07-2023".to_string());
    let expected_peak_2 = SummaryPeak::new(PeakType::Expense, 100.0, "07-2023".to_string());

    let expected_methods = vec![
        SummaryMethods::new(
            "Super Special Bank".to_string(),
            200.0,
            0.0,
            100.0,
            0.0,
            None,
            None,
            Some("∞".to_string()),
            Some("∞".to_string()),
        ),
        SummaryMethods::new(
            "Cash Cow".to_string(),
            0.0,
            100.0,
            0.0,
            100.0,
            None,
            None,
            Some("∞".to_string()),
            Some("∞".to_string()),
        ),
    ];

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();

    assert_eq!(my_summary_text, expected_data_1);
    assert_eq!(net, expected_net);
    assert_eq!(peak, vec![expected_peak_1, expected_peak_2]);
    assert_eq!(largest, vec![expected_largest_1, expected_largest_2]);
    assert_eq!(methods, expected_methods);
}

#[test]
fn check_summary_data_2() {
    let file_name = "summary_data_2.sqlite";
    let mut conn = create_test_db(file_name);

    add_tx(
        "2023-08-19",
        "Testing transaction",
        "Super Special Bank",
        "500.00",
        "Expense",
        "Car",
        None,
        &mut conn,
    )
    .unwrap();

    add_tx(
        "2023-01-19",
        "Testing transaction",
        "Super Special Bank",
        "500.00",
        "Expense",
        "Car",
        None,
        &mut conn,
    )
    .unwrap();

    add_tx(
        "2023-07-19",
        "Testing transaction",
        "Cash Cow",
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
        "Super Special Bank",
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
    let (old_net, _, _, old_methods) =
        my_summary.get_tx_data(&summary_modes, 0, 0, &None, &None, &conn);
    let my_summary_text = my_summary.get_table_data(&summary_modes, 0, 1);
    let (net, largest, peak, methods) = my_summary.get_tx_data(
        &summary_modes,
        0,
        1,
        &Some(old_methods),
        &Some(old_net),
        &conn,
    );

    let expected_net = SummaryNet::new(
        700.0,
        1000.0,
        Some(233.33333333333334),
        Some(333.3333333333333),
        41.17647058823529,
        58.82352941176471,
        Some("↓30.00".to_string()),
        Some("∞".to_string()),
    );
    let expected_largest_1 = SummaryLargest::new(
        LargestType::Earning,
        "Cash Cow".to_string(),
        700.0,
        "19-07-2023".to_string(),
    );

    let expected_largest_2 = SummaryLargest::new(
        LargestType::Expense,
        "Super Special Bank".to_string(),
        500.0,
        "19-01-2023".to_string(),
    );

    let expected_peak_1 = SummaryPeak::new(PeakType::Earning, 700.0, "07-2023".to_string());
    let expected_peak_2 = SummaryPeak::new(PeakType::Expense, 500.0, "01-2023".to_string());

    let expected_methods = vec![
        SummaryMethods::new(
            "Super Special Bank".to_string(),
            0.0,
            1000.0,
            0.0,
            100.0,
            Some(0.0),
            Some(333.3333333333333),
            Some("↓100.00".to_string()),
            Some("∞".to_string()),
        ),
        SummaryMethods::new(
            "Cash Cow".to_string(),
            700.0,
            0.0,
            100.0,
            0.0,
            Some(233.33333333333334),
            Some(0.0),
            Some("∞".to_string()),
            Some("∞".to_string()),
        ),
    ];
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
            "700.00".to_string(),
            "0.00".to_string(),
            "100.00".to_string(),
            "0.00".to_string(),
        ],
    ];

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();

    assert_eq!(my_summary_text, expected_data_1);
    assert_eq!(net, expected_net);
    assert_eq!(peak, vec![expected_peak_1, expected_peak_2]);
    assert_eq!(largest, vec![expected_largest_1, expected_largest_2]);
    assert_eq!(methods, expected_methods);
}

#[test]
fn check_summary_data_3() {
    let file_name = "summary_data_3.sqlite";
    let mut conn = create_test_db(file_name);

    add_tx(
        "2022-08-19",
        "Testing transaction",
        "Super Special Bank",
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
        "Cash Cow",
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
        "Super Special Bank",
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
    let (net, largest, peak, methods) =
        my_summary.get_tx_data(&summary_modes, 0, 1, &None, &None, &conn);

    let expected_net = SummaryNet::new(
        200.0,
        100.0,
        Some(66.66666666666667),
        Some(33.333333333333336),
        66.66666666666666,
        33.33333333333333,
        None,
        None,
    );
    let expected_largest_1 = SummaryLargest::new(
        LargestType::Earning,
        "Cash Cow".to_string(),
        100.0,
        "19-07-2023".to_string(),
    );

    let expected_largest_2 = SummaryLargest::new(
        LargestType::Expense,
        "Super Special Bank".to_string(),
        100.0,
        "19-08-2022".to_string(),
    );

    let expected_peak_1 = SummaryPeak::new(PeakType::Earning, 100.0, "07-2023".to_string());
    let expected_peak_2 = SummaryPeak::new(PeakType::Expense, 100.0, "08-2022".to_string());

    let expected_methods = vec![
        SummaryMethods::new(
            "Super Special Bank".to_string(),
            100.0,
            100.0,
            50.0,
            100.0,
            Some(33.333333333333336),
            Some(33.333333333333336),
            None,
            None,
        ),
        SummaryMethods::new(
            "Cash Cow".to_string(),
            100.0,
            0.0,
            50.00,
            0.0,
            Some(33.333333333333336),
            Some(0.0),
            None,
            None,
        ),
    ];
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

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();

    assert_eq!(my_summary_text, expected_data_1);
    assert_eq!(net, expected_net);
    assert_eq!(peak, vec![expected_peak_1, expected_peak_2]);
    assert_eq!(largest, vec![expected_largest_1, expected_largest_2]);
    assert_eq!(methods, expected_methods);
}

#[test]
fn check_summary_sorting() {
    let file_name = "summary_sorting.sqlite";
    let mut conn = create_test_db(file_name);

    add_tx(
        "2022-08-19",
        "Testing transaction",
        "Super Special Bank",
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
        "Cash Cow",
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
        "Super Special Bank",
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
