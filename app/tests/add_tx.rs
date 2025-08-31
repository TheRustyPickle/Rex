use app::fetcher::get_txs_date;
use app::modifier::parse_tx_fields;
use chrono::NaiveDate;
use db::ConnCache;
use std::fs;

use crate::common::create_test_db;

mod common;

#[test]
fn add_tx_test() {
    let file_name = "test_add_tx.sqlite";

    let mut db_conn = create_test_db(file_name);
    let cash_method = db_conn.cache().get_method_id("Cash").unwrap();

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

    let date_list = [
        NaiveDate::from_ymd_opt(2022, 7, 1).unwrap(),
        NaiveDate::from_ymd_opt(2022, 8, 1).unwrap(),
        NaiveDate::from_ymd_opt(2022, 9, 1).unwrap(),
        NaiveDate::from_ymd_opt(2022, 10, 1).unwrap(),
        NaiveDate::from_ymd_opt(2022, 10, 1).unwrap(),
        NaiveDate::from_ymd_opt(2022, 10, 1).unwrap(),
        NaiveDate::from_ymd_opt(2022, 10, 1).unwrap(),
        NaiveDate::from_ymd_opt(2022, 11, 1).unwrap(),
    ];

    let balance_list = [
        1000 * 100,
        900 * 100,
        0,
        5000 * 100,
        4900 * 100,
        4800 * 100,
        4700 * 100,
        (4700 - 10000) * 100,
    ];

    let tx_len = [1, 1, 1, 1, 2, 3, 4, 1];

    for (index, tx_fields) in tx_list.iter().enumerate() {
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

        let tx_list = get_txs_date(date_list[index], &mut db_conn).unwrap();
        assert_eq!(
            tx_list.0.len(),
            tx_len[index],
            "Length mismatch at index {}",
            index
        );

        let balance = tx_list.0[tx_len[index] - 1].balance[&cash_method];
        assert_eq!(
            balance, balance_list[index],
            "Balance mismatch at index {}",
            index
        );
    }

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}
