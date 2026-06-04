use rex_db::ConnCache;
use std::fs;

use crate::common::create_test_db;

mod common;

#[test]
fn add_new_method_increments_position() {
    let file_name = "test_method_add.sqlite";
    let mut db_conn = create_test_db(file_name);

    // Initial: Cash(1), Bank(2), Other(3)
    let methods = db_conn.get_tx_methods_sorted();
    let names: Vec<&str> = methods.iter().map(|m| m.name.as_str()).collect();
    assert_eq!(names, vec!["Cash", "Bank", "Other"]);
    assert_eq!(methods[0].position, 1);
    assert_eq!(methods[1].position, 2);
    assert_eq!(methods[2].position, 3);

    // Add one method
    let new = vec!["Wallet".to_string()];
    db_conn.add_new_methods(&new).unwrap();

    let methods = db_conn.get_tx_methods_sorted();
    let names: Vec<&str> = methods.iter().map(|m| m.name.as_str()).collect();
    assert_eq!(names, vec!["Cash", "Bank", "Other", "Wallet"]);
    assert_eq!(methods[3].position, 4);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn add_multiple_methods_sequential_positions() {
    let file_name = "test_method_add_multi.sqlite";
    let mut db_conn = create_test_db(file_name);

    let new = vec![
        "Wallet".to_string(),
        "Savings".to_string(),
        "Investments".to_string(),
    ];
    db_conn.add_new_methods(&new).unwrap();

    let methods = db_conn.get_tx_methods_sorted();
    let names: Vec<&str> = methods.iter().map(|m| m.name.as_str()).collect();
    assert_eq!(
        names,
        vec!["Cash", "Bank", "Other", "Wallet", "Savings", "Investments"]
    );
    assert_eq!(methods[3].position, 4);
    assert_eq!(methods[4].position, 5);
    assert_eq!(methods[5].position, 6);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn rename_method_updates_name_preserves_position() {
    let file_name = "test_method_rename.sqlite";
    let mut db_conn = create_test_db(file_name);

    let pos_before = db_conn.get_tx_method_by_name("Cash").unwrap().position;
    db_conn.rename_tx_method("Cash", "CashRenamed").unwrap();

    let renamed = db_conn.get_tx_method_by_name("CashRenamed").unwrap();
    assert_eq!(renamed.name, "CashRenamed");
    assert_eq!(renamed.position, pos_before);

    // Old name no longer accessible via cache
    assert!(db_conn.get_tx_method_by_name("Cash").is_err());

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn reorder_methods_changes_positions() {
    let file_name = "test_method_reorder.sqlite";
    let mut db_conn = create_test_db(file_name);

    // Reverse order: Other, Bank, Cash
    let new_order: Vec<String> = vec!["Other", "Bank", "Cash"]
        .into_iter()
        .map(String::from)
        .collect();

    db_conn.set_new_tx_method_positions(&new_order).unwrap();

    let methods = db_conn.get_tx_methods_sorted();
    let names: Vec<&str> = methods.iter().map(|m| m.name.as_str()).collect();
    assert_eq!(names, vec!["Other", "Bank", "Cash"]);
    // set_new_tx_method_positions assigns 0-based positions
    assert_eq!(methods[0].position, 0);
    assert_eq!(methods[1].position, 1);
    assert_eq!(methods[2].position, 2);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn is_tx_method_empty() {
    // create_test_db already adds 3 methods, so test with a fresh conn
    let file_name = "test_method_empty.sqlite";
    let db_conn = create_test_db(file_name);
    assert!(!db_conn.is_tx_method_empty());
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn get_tx_methods_cumulative_appends_cumulative() {
    let file_name = "test_method_cumulative.sqlite";
    let db_conn = create_test_db(file_name);

    let cumulative = db_conn.get_tx_methods_cumulative();
    assert_eq!(cumulative, vec!["Cash", "Bank", "Other", "Cumulative"]);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn get_final_balances_returns_all_methods() {
    let file_name = "test_method_final_balances.sqlite";
    let mut db_conn = create_test_db(file_name);

    let balances = db_conn.get_final_balances().unwrap();
    let cash_id = db_conn.cache().get_method_id("Cash").unwrap();
    let bank_id = db_conn.cache().get_method_id("Bank").unwrap();
    let other_id = db_conn.cache().get_method_id("Other").unwrap();

    assert!(balances.contains_key(&cash_id));
    assert!(balances.contains_key(&bank_id));
    assert!(balances.contains_key(&other_id));
    assert_eq!(balances.len(), 3);

    // Fresh methods should have zero final balance
    assert_eq!(balances[&cash_id].balance, 0);
    assert_eq!(balances[&bank_id].balance, 0);
    assert_eq!(balances[&other_id].balance, 0);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn get_tx_method_by_name_not_found() {
    let file_name = "test_method_not_found.sqlite";
    let mut db_conn = create_test_db(file_name);

    assert!(db_conn.get_tx_method_by_name("Nonexistent").is_err());

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}
