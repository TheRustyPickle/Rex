use rex_app::modifier::{parse_search_fields, parse_tx_fields};
use std::fs;

use crate::common::create_test_db;

mod common;

#[test]
fn search_tx_test() {
    let file_name = "test_search_tx.sqlite";

    let mut db_conn = create_test_db(file_name);

    let tx_list = [
        [
            "2022-07-01",
            "Salary",
            "Cash",
            "",
            "1000.00",
            "Income",
            "Salary",
        ],
        [
            "2022-08-01",
            "Car expense",
            "Cash",
            "",
            "100.00",
            "Expense",
            "Car, Maintenance",
        ],
        [
            "2022-09-01",
            "Bankruptcy",
            "Cash",
            "",
            "900.00",
            "Expense",
            "Bankruptcy",
        ],
        [
            "2022-10-01",
            "Inheritance",
            "Cash",
            "",
            "5000.00",
            "Income",
            "Inheritance",
        ],
        [
            "2022-10-01",
            "Groceries",
            "Cash",
            "",
            "100.00",
            "Expense",
            "Groceries",
        ],
        [
            "2022-10-01",
            "More Groceries",
            "Cash",
            "",
            "100.00",
            "Expense",
            "Groceries",
        ],
        [
            "2022-10-05",
            "Rent",
            "Cash",
            "",
            "100.00",
            "Expense",
            "Rent",
        ],
        [
            "2022-11-01",
            "Debt",
            "Cash",
            "",
            "10000.00",
            "Expense",
            "Debt",
        ],
    ];

    for tx_fields in tx_list {
        let new_tx = parse_tx_fields(
            tx_fields[0],
            tx_fields[1],
            tx_fields[2],
            tx_fields[3],
            tx_fields[4],
            tx_fields[5],
            &db_conn,
        )
        .unwrap();

        db_conn.add_new_tx(new_tx, tx_fields[6]).unwrap();
    }

    let search_queries = [
        ["2022", "", "", "", "", "", ""],
        ["", "", "", "", "", "Income", ""],
        ["", "", "", "", "", "Expense", ""],
        ["", "", "", "", "", "", "Groceries"],
        ["", "", "", "", "", "", ""],
        ["2023", "", "", "", "", "", ""],
    ];

    let expected_len = [8, 2, 6, 2, 8, 0];

    for (index, search) in search_queries.into_iter().enumerate() {
        let search_tx = parse_search_fields(
            search[0], search[1], search[2], search[3], search[4], search[5], search[6], &db_conn,
        )
        .unwrap();

        let txs = search_tx.search_txs(&mut db_conn).unwrap();

        let expected_len = expected_len[index];

        assert_eq!(
            txs.len(),
            expected_len,
            "Expected {} txs for index {index}",
            expected_len
        );
    }

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}
