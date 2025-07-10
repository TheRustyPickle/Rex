extern crate rex_tui;
use chrono::{Datelike, Local};
use rex_tui::db::YEARS;
use rex_tui::page_handler::{ActivityType, DateType};
use rex_tui::tx_handler::*;
use rex_tui::utility::{
    add_new_activity, add_new_activity_tx, get_all_activities, get_search_data, switch_tx_index,
};
use rusqlite::Connection;
use std::fs;

use crate::common::create_test_db;

mod common;

fn add_dummy_tx(conn: &mut Connection) {
    let mut tx_data = TxData {
        date: "2022-08-19".to_string(),
        details: "Car expense".to_string(),
        from_method: "Super Special Bank".to_string(),
        to_method: String::new(),
        amount: "100.00".to_string(),
        tx_type: "Expense".to_string(),
        tags: "Car".to_string(),
        editing_tx: false,
        autofill: String::new(),
        id_num: 0,
        current_index: 0,
        tx_status: Vec::new(),
    };

    tx_data.add_tx(conn).unwrap();

    tx_data.details = "Edited Car expense".to_string();

    tx_data.editing_tx = true;
    tx_data.id_num = 1;

    tx_data.add_tx(conn).unwrap();

    let mut tx_data_2 = TxData {
        date: "2022-08-19".to_string(),
        details: "House expense".to_string(),
        from_method: "Cash Cow".to_string(),
        to_method: String::new(),
        amount: "500.00".to_string(),
        tx_type: "Expense".to_string(),
        tags: "House".to_string(),
        editing_tx: false,
        autofill: String::new(),
        id_num: 0,
        current_index: 0,
        tx_status: Vec::new(),
    };

    tx_data_2.add_tx(conn).unwrap();

    let all_texts: Vec<String> = vec![
        tx_data.date.clone(),
        tx_data.details.clone(),
        tx_data.from_method.clone(),
        tx_data.amount.clone(),
        tx_data.tx_type.clone(),
        tx_data.tags.clone(),
        "1".to_string(),
    ];

    let all_texts_2: Vec<String> = vec![
        tx_data_2.date.clone(),
        tx_data_2.details.clone(),
        tx_data_2.from_method.clone(),
        tx_data_2.amount.clone(),
        tx_data_2.tx_type.clone(),
        tx_data_2.tags.clone(),
    ];

    switch_tx_index(1, 2, &all_texts, &all_texts_2, conn);

    get_search_data("", "Selling", "", "", "", "", "", &DateType::Exact, conn);
    get_search_data(
        "",
        "Selling",
        "Super Special Bank",
        "",
        "",
        "",
        "",
        &DateType::Exact,
        conn,
    );

    let activity_num = add_new_activity(ActivityType::DeleteTX(Some(1)), conn);
    add_new_activity_tx(&all_texts, activity_num, conn);
}

#[test]
fn activity_test() {
    let file_name = "activity_test.sqlite";
    let mut conn = create_test_db(file_name);

    let year = Local::now().year().to_string();
    let month = Local::now().month() as usize;

    let year_index = YEARS.iter().position(|y| y == &year).unwrap();

    let month_index = month - 1;

    add_dummy_tx(&mut conn);

    let activities = get_all_activities(1, 0, &conn);

    assert_eq!(activities.0.len(), 0);
    assert_eq!(activities.1.len(), 0);

    let activities = get_all_activities(month_index, year_index, &conn);

    assert_eq!(activities.0.len(), 7);
    assert_eq!(activities.1.len(), 7);

    for details in activities.0 {
        match details.activity_type {
            ActivityType::NewTX => {
                let activity_num = details.activity_num();
                let activity_txs = activities.1.get(&activity_num).unwrap();
                assert_eq!(activity_txs.len(), 1);

                assert_eq!(details.description, details.activity_type.to_details());

                let tx = &activity_txs[0];

                let expected_date = "19-08-2022".to_string();
                let expected_details = ["Car expense".to_string(), "House expense".to_string()];
                let expected_from_method =
                    ["Super Special Bank".to_string(), "Cash Cow".to_string()];
                let expected_amount = ["100.00".to_string(), "500.00".to_string()];
                let expected_tx_type = "Expense".to_string();
                let expected_tags = ["Car".to_string(), "House".to_string()];

                assert_eq!(tx.date, expected_date);
                assert!(expected_details.contains(&tx.details));
                assert!(expected_from_method.contains(&tx.tx_method));
                assert!(expected_amount.contains(&tx.amount));
                assert_eq!(tx.tx_type, expected_tx_type);
                assert!(expected_tags.contains(&tx.tags));
            }
            ActivityType::EditTX(_) => {
                let activity_num = details.activity_num();
                let activity_txs = activities.1.get(&activity_num).unwrap();
                assert_eq!(activity_txs.len(), 2);

                assert_eq!(
                    details.description,
                    ActivityType::EditTX(Some(1)).to_details()
                );

                let expected_date = "19-08-2022".to_string();
                let expected_details =
                    ["Car expense".to_string(), "Edited Car expense".to_string()];
                let expected_from_method = "Super Special Bank".to_string();
                let expected_amount = "100.00".to_string();
                let expected_tx_type = "Expense".to_string();
                let expected_tags = "Car".to_string();

                activity_txs.iter().enumerate().for_each(|(index, tx)| {
                    assert_eq!(tx.date, expected_date);
                    if index == 1 {
                        assert_eq!(tx.details, expected_details[0]);
                    } else {
                        assert_eq!(tx.details, expected_details[1]);
                    }
                    assert_eq!(tx.tx_method, expected_from_method);
                    assert_eq!(tx.amount, expected_amount);
                    assert_eq!(tx.tx_type, expected_tx_type);
                    assert_eq!(tx.tags, expected_tags);
                });
            }
            ActivityType::DeleteTX(_) => {
                let activity_num = details.activity_num();
                let activity_txs = activities.1.get(&activity_num).unwrap();

                assert_eq!(activity_txs.len(), 1);

                assert_eq!(
                    details.description,
                    ActivityType::DeleteTX(Some(1)).to_details()
                );

                let tx = &activity_txs[0];

                let expected_date = "19-08-2022".to_string();
                let expected_details = "Edited Car expense".to_string();
                let expected_from_method = "Super Special Bank".to_string();
                let expected_amount = "100.00".to_string();
                let expected_tx_type = "Expense".to_string();
                let expected_tags = "Car".to_string();

                assert_eq!(tx.date, expected_date);
                assert_eq!(tx.details, expected_details);
                assert_eq!(tx.tx_method, expected_from_method);
                assert_eq!(tx.amount, expected_amount);
                assert_eq!(tx.tx_type, expected_tx_type);
                assert_eq!(tx.tags, expected_tags);
            }
            ActivityType::IDNumSwap(_, _) => {
                let activity_num = details.activity_num();
                let activity_txs = activities.1.get(&activity_num).unwrap();

                assert_eq!(activity_txs.len(), 2);

                assert_eq!(
                    details.description,
                    ActivityType::IDNumSwap(Some(1), Some(2)).to_details()
                );

                let expected_date = "19-08-2022".to_string();
                let expected_details = "Edited Car expense".to_string();
                let expected_from_method = "Super Special Bank".to_string();
                let expected_amount = "100.00".to_string();
                let expected_tx_type = "Expense".to_string();
                let expected_tags = "Car".to_string();

                assert_eq!(activity_txs[0].date, expected_date);
                assert_eq!(activity_txs[0].details, expected_details);
                assert_eq!(activity_txs[0].tx_method, expected_from_method);
                assert_eq!(activity_txs[0].amount, expected_amount);
                assert_eq!(activity_txs[0].tx_type, expected_tx_type);
                assert_eq!(activity_txs[0].tags, expected_tags);
                assert_eq!(activity_txs[0].id_num, "2".to_string());

                let expected_date = "19-08-2022".to_string();
                let expected_details = "House expense".to_string();
                let expected_from_method = "Cash Cow".to_string();
                let expected_amount = "500.00".to_string();
                let expected_tx_type = "Expense".to_string();
                let expected_tags = "House".to_string();

                assert_eq!(activity_txs[1].date, expected_date);
                assert_eq!(activity_txs[1].details, expected_details);
                assert_eq!(activity_txs[1].tx_method, expected_from_method);
                assert_eq!(activity_txs[1].amount, expected_amount);
                assert_eq!(activity_txs[1].tx_type, expected_tx_type);
                assert_eq!(activity_txs[1].tags, expected_tags);
                assert_eq!(activity_txs[1].id_num, "1".to_string());
            }
            ActivityType::SearchTX(_) => {
                let activity_num = details.activity_num();
                let activity_txs = activities.1.get(&activity_num).unwrap();

                let tx = &activity_txs[0];
                assert_eq!(activity_txs.len(), 1);

                let searched_with = if tx.tx_method.is_empty() { 1 } else { 2 };

                assert_eq!(
                    details.description,
                    ActivityType::SearchTX(Some(searched_with)).to_details()
                );

                let expected_details = "Selling".to_string();
                let expected_from_method = [String::new(), "Super Special Bank".to_string()];

                assert_eq!(tx.date, String::new());
                assert_eq!(tx.details, expected_details);
                assert!(expected_from_method.contains(&tx.tx_method));
                assert_eq!(tx.amount, String::new());
                assert_eq!(tx.tx_type, String::new());
                assert_eq!(tx.tags, String::new());
            }
        }
    }

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();
}
