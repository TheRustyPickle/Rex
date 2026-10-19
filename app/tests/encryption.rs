use chrono::NaiveDate;
use rex_app::conn::{DbConn, FetchNature, OpenError};
use std::fs;
use std::io::Read;
use std::path::Path;

use crate::common::{add_tx, create_test_db};

mod common;

const SQLITE_HEADER: &[u8] = b"SQLite format 3\0";

fn starts_with_sqlite_header(path: &str) -> bool {
    let mut header = [0u8; 16];
    fs::File::open(path)
        .unwrap()
        .read_exact(&mut header)
        .unwrap();
    header == SQLITE_HEADER
}

fn tx_count(db: &mut DbConn) -> usize {
    db.fetch_txs_with_date(
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        FetchNature::All,
    )
    .unwrap()
    .len()
}

fn seed(db: &mut DbConn) {
    add_tx(
        db,
        "2024-06-01",
        "Salary",
        "Cash",
        "",
        "3000.00",
        "Income",
        "Work",
    );
    add_tx(
        db,
        "2024-06-05",
        "Rent",
        "Cash",
        "",
        "1000.00",
        "Expense",
        "Home",
    );
    add_tx(
        db,
        "2024-07-05",
        "Move",
        "Cash",
        "Bank",
        "200.00",
        "Transfer",
        "",
    );
}

fn cleanup(file_name: &str) {
    fs::remove_file(file_name).unwrap();
    assert!(!Path::new(&format!("{file_name}.encrypting")).exists());
}

#[test]
fn plain_database_is_readable_and_reports_not_encrypted() {
    let file_name = "test_encryption_plain.sqlite";
    let db = create_test_db(file_name);

    assert!(!db.is_encrypted());
    assert!(starts_with_sqlite_header(file_name));

    drop(db);
    cleanup(file_name);
}

#[test]
fn set_password_makes_the_file_unreadable_without_it() {
    let file_name = "test_encryption_set.sqlite";
    let mut db = create_test_db(file_name);
    seed(&mut db);

    db.set_password("correct horse").unwrap();
    assert!(db.is_encrypted());

    // The live connection keeps working
    assert_eq!(tx_count(&mut db), 3);
    drop(db);

    assert!(!starts_with_sqlite_header(file_name));
    assert!(matches!(
        DbConn::open(file_name, None),
        Err(OpenError::NotReadable)
    ));
    assert!(matches!(
        DbConn::open(file_name, Some("wrong")),
        Err(OpenError::NotReadable)
    ));

    let mut reopened = DbConn::open(file_name, Some("correct horse")).unwrap();
    assert!(reopened.is_encrypted());
    assert_eq!(tx_count(&mut reopened), 3);
    assert_eq!(
        reopened
            .get_tx_methods_sorted()
            .iter()
            .map(|m| m.name.as_str())
            .collect::<Vec<_>>(),
        vec!["Cash", "Bank", "Other"]
    );

    drop(reopened);
    cleanup(file_name);
}

#[test]
fn data_survives_encryption_including_balances_and_tags() {
    let file_name = "test_encryption_data.sqlite";
    let mut db = create_test_db(file_name);
    seed(&mut db);

    let before = db.get_final_balances().unwrap();
    let cash = db.get_tx_method_by_name("Cash").unwrap().id;
    let before_cash = before[&cash].balance;

    db.set_password("pw").unwrap();
    drop(db);

    let mut db = DbConn::open(file_name, Some("pw")).unwrap();
    let after = db.get_final_balances().unwrap();
    assert_eq!(after[&cash].balance, before_cash);

    let tags: Vec<String> = db
        .get_tags_sorted()
        .iter()
        .map(|t| t.name.clone())
        .collect();
    assert_eq!(tags, vec!["Home", "Unknown", "Work"]);

    drop(db);
    cleanup(file_name);
}

#[test]
fn an_encrypted_database_accepts_new_data() {
    let file_name = "test_encryption_write.sqlite";
    let mut db = create_test_db(file_name);
    seed(&mut db);
    db.set_password("pw").unwrap();

    add_tx(
        &mut db,
        "2024-08-01",
        "More",
        "Cash",
        "",
        "5.00",
        "Expense",
        "Home",
    );
    drop(db);

    let mut db = DbConn::open(file_name, Some("pw")).unwrap();
    assert_eq!(tx_count(&mut db), 4);

    drop(db);
    cleanup(file_name);
}

#[test]
fn changing_the_password_replaces_the_old_one() {
    let file_name = "test_encryption_change.sqlite";
    let mut db = create_test_db(file_name);
    seed(&mut db);

    db.set_password("first").unwrap();
    db.set_password("second").unwrap();
    drop(db);

    assert!(matches!(
        DbConn::open(file_name, Some("first")),
        Err(OpenError::NotReadable)
    ));

    let mut db = DbConn::open(file_name, Some("second")).unwrap();
    assert_eq!(tx_count(&mut db), 3);

    drop(db);
    cleanup(file_name);
}

#[test]
fn passwords_with_quotes_and_unicode_work() {
    let file_name = "test_encryption_quotes.sqlite";
    let mut db = create_test_db(file_name);
    seed(&mut db);

    let password = "it's \"a\" pässwörd; DROP TABLE txs;--";
    db.set_password(password).unwrap();
    drop(db);

    let mut db = DbConn::open(file_name, Some(password)).unwrap();
    assert_eq!(tx_count(&mut db), 3);

    drop(db);
    cleanup(file_name);
}

#[test]
fn empty_password_is_rejected_and_changes_nothing() {
    let file_name = "test_encryption_empty.sqlite";
    let mut db = create_test_db(file_name);
    seed(&mut db);

    assert!(db.set_password("").is_err());
    assert!(!db.is_encrypted());
    drop(db);

    assert!(starts_with_sqlite_header(file_name));
    assert_eq!(tx_count(&mut DbConn::open(file_name, None).unwrap()), 3);

    cleanup(file_name);
}

#[test]
fn a_leftover_temp_file_from_a_crash_does_not_block_encrypting() {
    let file_name = "test_encryption_leftover.sqlite";
    let mut db = create_test_db(file_name);
    seed(&mut db);

    fs::write(format!("{file_name}.encrypting"), b"garbage").unwrap();

    db.set_password("pw").unwrap();
    drop(db);

    let mut db = DbConn::open(file_name, Some("pw")).unwrap();
    assert_eq!(tx_count(&mut db), 3);

    drop(db);
    cleanup(file_name);
}
