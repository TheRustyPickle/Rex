use std::fs;

use crate::common::add_tx;
use crate::common::create_test_db;

mod common;

#[test]
fn rename_tag_updates_name() {
    let file_name = "test_tag_rename.sqlite";
    let mut db_conn = create_test_db(file_name);

    let tx = add_tx(
        &mut db_conn,
        "2025-06-15",
        "Weekly shopping",
        "Cash",
        "",
        "50.00",
        "Expense",
        "Groceries",
    );

    let tag_id = {
        let tags = db_conn.get_tags_sorted();
        let groc_tag = tags.iter().find(|t| t.name == "Groceries").unwrap();
        assert_eq!(groc_tag.name, "Groceries");
        groc_tag.id
    };

    db_conn.rename_tag("Groceries", "Food").unwrap();

    let tags = db_conn.get_tags_sorted();
    let renamed = tags.iter().find(|t| t.id == tag_id).unwrap();
    assert_eq!(renamed.name, "Food");

    let old_exists = tags.iter().any(|t| t.name == "Groceries");
    assert!(!old_exists);

    drop(tags);

    let full_tx = db_conn.fetch_tx_with_id(tx.id).unwrap();
    let tx_tag_names: Vec<&str> = full_tx.tags.iter().map(|t| t.name.as_str()).collect();
    assert!(tx_tag_names.contains(&"Food"));
    assert!(!tx_tag_names.contains(&"Groceries"));

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn rename_tag_to_existing_fails() {
    let file_name = "test_tag_rename_dup.sqlite";
    let mut db_conn = create_test_db(file_name);

    add_tx(
        &mut db_conn,
        "2025-06-15",
        "Groceries",
        "Cash",
        "",
        "50.00",
        "Expense",
        "Groceries",
    );
    add_tx(
        &mut db_conn,
        "2025-06-16",
        "Food expense",
        "Cash",
        "",
        "30.00",
        "Expense",
        "Food",
    );

    let result = db_conn.rename_tag("Groceries", "Food");
    assert!(result.is_err());

    let tags = db_conn.get_tags_sorted();
    let still_groceries = tags.iter().any(|t| t.name == "Groceries");
    assert!(still_groceries);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}
