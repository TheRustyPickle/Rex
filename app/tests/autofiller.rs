use std::fs;

use crate::common::{add_tx, create_test_db};

mod common;

// ---- Tx Method autofill ----

#[test]
fn autofill_method_exact_returns_empty() {
    let file_name = "test_autofill_method.sqlite";
    let mut db_conn = create_test_db(file_name);
    // Exact match produces no suggestion
    let result = db_conn.autofill().tx_method("Cash");
    assert_eq!(result, "");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn autofill_method_fuzzy_suggests_match() {
    let file_name = "test_autofill_method_fuzzy.sqlite";
    let mut db_conn = create_test_db(file_name);
    let result = db_conn.autofill().tx_method("Csh");
    assert_eq!(result, "Cash");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn autofill_method_empty_returns_empty() {
    let file_name = "test_autofill_method_empty.sqlite";
    let mut db_conn = create_test_db(file_name);
    let result = db_conn.autofill().tx_method("");
    assert_eq!(result, "");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

// ---- Tx Type autofill ----

#[test]
fn autofill_tx_type_short_e() {
    let file_name = "test_autofill_type_e.sqlite";
    let mut db_conn = create_test_db(file_name);
    let result = db_conn.autofill().tx_type("e");
    assert_eq!(result, "Expense");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn autofill_tx_type_short_i() {
    let file_name = "test_autofill_type_i.sqlite";
    let mut db_conn = create_test_db(file_name);
    let result = db_conn.autofill().tx_type("i");
    assert_eq!(result, "Income");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn autofill_tx_type_short_t() {
    let file_name = "test_autofill_type_t.sqlite";
    let mut db_conn = create_test_db(file_name);
    let result = db_conn.autofill().tx_type("t");
    assert_eq!(result, "Transfer");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn autofill_tx_type_short_b() {
    let file_name = "test_autofill_type_b.sqlite";
    let mut db_conn = create_test_db(file_name);
    let result = db_conn.autofill().tx_type("b");
    assert_eq!(result, "Borrow");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn autofill_tx_type_short_l() {
    let file_name = "test_autofill_type_l.sqlite";
    let mut db_conn = create_test_db(file_name);
    let result = db_conn.autofill().tx_type("l");
    assert_eq!(result, "Lend");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn autofill_tx_type_short_br() {
    let file_name = "test_autofill_type_br.sqlite";
    let mut db_conn = create_test_db(file_name);
    let result = db_conn.autofill().tx_type("br");
    assert_eq!(result, "Borrow Repay");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn autofill_tx_type_short_lr() {
    let file_name = "test_autofill_type_lr.sqlite";
    let mut db_conn = create_test_db(file_name);
    let result = db_conn.autofill().tx_type("lr");
    assert_eq!(result, "Lend Repay");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn autofill_tx_type_exact_returns_empty() {
    let file_name = "test_autofill_type_exact.sqlite";
    let mut db_conn = create_test_db(file_name);
    let result = db_conn.autofill().tx_type("Expense");
    assert_eq!(result, "");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn autofill_tx_type_fuzzy_suggests_match() {
    let file_name = "test_autofill_type_fuzzy.sqlite";
    let mut db_conn = create_test_db(file_name);
    let result = db_conn.autofill().tx_type("Epense");
    assert_eq!(result, "Expense");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn autofill_tx_type_empty_returns_empty() {
    let file_name = "test_autofill_type_empty.sqlite";
    let mut db_conn = create_test_db(file_name);
    let result = db_conn.autofill().tx_type("");
    assert_eq!(result, "");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

// ---- Tags autofill ----

#[test]
fn autofill_tags_fuzzy_suggests_match() {
    let file_name = "test_autofill_tags.sqlite";
    let mut db_conn = create_test_db(file_name);
    add_tx(
        &mut db_conn,
        "2024-01-01",
        "T1",
        "Cash",
        "",
        "10.00",
        "Expense",
        "Groceries",
    );

    let result = db_conn.autofill().tags("Gro");
    assert_eq!(result, "Groceries");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn autofill_tags_exact_returns_empty() {
    let file_name = "test_autofill_tags_exact.sqlite";
    let mut db_conn = create_test_db(file_name);
    add_tx(
        &mut db_conn,
        "2024-02-01",
        "T1",
        "Cash",
        "",
        "10.00",
        "Expense",
        "Rent",
    );

    let result = db_conn.autofill().tags("Rent");
    assert_eq!(result, "");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn autofill_tags_empty_returns_empty() {
    let file_name = "test_autofill_tags_empty.sqlite";
    let mut db_conn = create_test_db(file_name);
    add_tx(
        &mut db_conn,
        "2024-03-01",
        "T1",
        "Cash",
        "",
        "10.00",
        "Expense",
        "Tag",
    );

    let result = db_conn.autofill().tags("");
    assert_eq!(result, "");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn autofill_tags_only_last_tag_suggested() {
    let file_name = "test_autofill_tags_multi.sqlite";
    let mut db_conn = create_test_db(file_name);
    add_tx(
        &mut db_conn,
        "2024-04-01",
        "T1",
        "Cash",
        "",
        "10.00",
        "Expense",
        "Groceries, Salary",
    );

    // Only the last tag (after comma) is matched
    let result = db_conn.autofill().tags("Groceries, Sal");
    assert_eq!(result, "Salary");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

// ---- Details autofill ----

#[test]
fn autofill_details_fuzzy_suggests_match() {
    let file_name = "test_autofill_details.sqlite";
    let mut db_conn = create_test_db(file_name);
    add_tx(
        &mut db_conn,
        "2024-05-01",
        "Amazon purchase",
        "Cash",
        "",
        "50.00",
        "Expense",
        "Shopping",
    );

    let result = db_conn.autofill().details("Amaz");
    assert_eq!(result, "Amazon purchase");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn autofill_details_on_empty_db_returns_empty() {
    let file_name = "test_autofill_details_empty_db.sqlite";
    let mut db_conn = create_test_db(file_name);
    // No transactions = no details to match
    let result = db_conn.autofill().details("anything");
    assert_eq!(result, "");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}
