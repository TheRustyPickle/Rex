extern crate rex_tui;
use rex_tui::db::create_db;
use rex_tui::tx_handler::*;
use rex_tui::utility::traits::AutoFiller;
use rusqlite::Connection;
use std::fs;

struct Testing {
    data: Vec<String>,
    expected: Vec<String>,
}
impl AutoFiller for Testing {}

fn create_test_db(file_name: &str) -> Connection {
    if let Ok(metadata) = fs::metadata(file_name) {
        if metadata.is_file() {
            fs::remove_file(file_name).expect("Failed to delete existing file");
        }
    }

    let mut conn = Connection::open(file_name).unwrap();
    create_db(
        &vec!["Super Special Bank".to_string(), "Cash Cow".to_string()],
        &mut conn,
    )
    .unwrap();
    conn
}

fn add_dummy_tx(conn: &mut Connection) {
    add_tx(
        "2022-08-19",
        "Car expense",
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
        "Food cost",
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
        "Selling goods",
        "Super Special Bank",
        "200.00",
        "Income",
        "Goods",
        None,
        conn,
    )
    .unwrap();
}

#[test]
fn autofiller_test() {
    let file_name = "autofiller_test.sqlite";
    let mut conn = create_test_db(file_name);

    let data = vec![
        "car",
        "expense",
        "sll",
        "f o o d",
        "Selling goods",
        "coast",
        "r ex",
        "",
    ]
    .into_iter()
    .map(|a| a.to_string())
    .collect::<Vec<String>>();
    let expected = vec![
        "Car expense",
        "Car expense",
        "Selling goods",
        "Food cost",
        "",
        "Food cost",
        "Car expense",
        "",
    ]
    .into_iter()
    .map(|a| a.to_string())
    .collect::<Vec<String>>();

    let test_data = Testing { data, expected };

    // Nothing to take suggestion from so empty string as autofiller
    for i in 0..test_data.data.len() {
        let result = test_data.autofill_details(&test_data.data[i], &conn);
        assert_eq!(result, String::new());
    }

    add_dummy_tx(&mut conn);

    for i in 0..test_data.data.len() {
        let result = test_data.autofill_details(&test_data.data[i], &conn);
        assert_eq!(result, test_data.expected[i]);
    }

    let data = vec![
        "sup", "cash", "CoW", "Cis", "sup", "bank", "CaSh CoW", "Cash Cow", "",
    ]
    .into_iter()
    .map(|a| a.to_string())
    .collect::<Vec<String>>();
    let expected = vec![
        "Super Special Bank",
        "Cash Cow",
        "Cash Cow",
        "Cash Cow",
        "Super Special Bank",
        "Super Special Bank",
        "Cash Cow",
        "",
        "",
    ]
    .into_iter()
    .map(|a| a.to_string())
    .collect::<Vec<String>>();

    let test_data = Testing { data, expected };

    for i in 0..test_data.data.len() {
        let result = test_data.autofill_tx_method(&test_data.data[i], &conn);
        assert_eq!(result, test_data.expected[i]);
    }

    let data = vec![
        "foo", "goo", "fod", "gid", "rac", "Car", "", "Food,", "Food, Go",
    ]
    .into_iter()
    .map(|a| a.to_string())
    .collect::<Vec<String>>();
    let expected = vec!["Food", "Goods", "Food", "Goods", "Car", "", "", "", "Goods"]
        .into_iter()
        .map(|a| a.to_string())
        .collect::<Vec<String>>();

    let test_data = Testing { data, expected };

    for i in 0..test_data.data.len() {
        let result = test_data.autofill_tags(&test_data.data[i], &conn);
        assert_eq!(result, test_data.expected[i]);
    }

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();
}
