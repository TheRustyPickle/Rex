extern crate rex_tui;

use rex_tui::tx_handler::add_tx;
use rex_tui::utility::*;
use std::env::current_dir;
use std::fs;

mod common;

use crate::common::create_test_db;

#[test]
fn check_unique_tags() {
    let file_name = "tag_checker.sqlite";
    let mut conn = create_test_db(file_name);

    add_tx(
        "2022-07-19",
        "Testing transaction",
        "Super Special Bank",
        "100.00",
        "Income",
        "Test tag",
        None,
        &mut conn,
    )
    .unwrap();

    add_tx(
        "2022-07-19",
        "Testing transaction",
        "Cash Cow",
        "100.00",
        "Income",
        "Tag name",
        None,
        &mut conn,
    )
    .unwrap();

    add_tx(
        "2022-08-19",
        "Testing transaction",
        "Super Special Bank",
        "100.00",
        "Income",
        "test tag",
        None,
        &mut conn,
    )
    .unwrap();

    let all_tags = get_all_tags(&conn);
    let expected_data = ["Tag name", "Test tag", "test tag"]
        .iter()
        .map(|s| (*s).to_string())
        .collect::<Vec<String>>();

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();

    assert_eq!(all_tags, expected_data);
}

#[test]
fn check_restricted_test() {
    let word_list = ["Unknown", "Words", "Testing"]
        .iter()
        .map(|s| (*s).to_string())
        .collect::<Vec<String>>();

    let is_restricted = check_restricted("cancel", None);
    let not_restricted = check_restricted("some word", None);

    assert!(is_restricted);
    assert!(!not_restricted);

    let is_restricted = check_restricted("unknown", Some(&word_list));
    let not_restricted = check_restricted("some word", Some(&word_list));

    assert!(is_restricted);
    assert!(!not_restricted);
}

#[test]
fn github_parser_test() {
    let body = "## Updates
* Some release
* text that are taken
* from the latest github release
    
## Changes
more stuff"
        .to_string();

    let parsed = parse_github_body(&body);

    let expected_data = "
• Some release
• text that are taken
• from the latest github release
"
    .to_string();

    assert_eq!(parsed, expected_data);
}

#[test]
fn test_location_json() {
    let mut current_dir = current_dir().unwrap();
    let json_exists = is_location_changed(&current_dir);
    assert_eq!(json_exists, None);

    create_change_location_file(&current_dir, &current_dir);

    let json_exists = is_location_changed(&current_dir);
    assert_eq!(json_exists, Some(current_dir.clone()));

    current_dir.pop();
    current_dir.push("location.json");
    fs::remove_file(current_dir).unwrap();
}

#[test]
fn misc_tests() {
    let file_name = "misc_check.sqlite";
    let conn = create_test_db(file_name);

    let mut all_tx_methods = get_all_tx_methods(&conn);
    let all_tx_methods_cumulative = get_all_tx_methods_cumulative(&conn);

    all_tx_methods.push("Cumulative".to_string());

    assert_eq!(all_tx_methods, all_tx_methods_cumulative);

    let table_names = get_all_table_names(&conn);

    let expected_tables = [
        "tx_all",
        "sqlite_sequence",
        "balance_all",
        "changes_all",
        "activities",
        "activity_txs",
    ]
    .iter()
    .map(|s| (*s).to_string())
    .collect::<Vec<String>>();

    assert_eq!(table_names, expected_tables);

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();
}
