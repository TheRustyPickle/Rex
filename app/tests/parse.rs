use rex_app::modifier::parse_search_fields;
use rex_db::ConnCache;
use std::fs;

use crate::common::create_test_db;

mod common;

#[test]
fn parse_search_fields_year_only() {
    let file_name = "test_parse_search_date.sqlite";
    let mut db_conn = create_test_db(file_name);

    let search = parse_search_fields("2024", "", "", "", "", "", "", &db_conn).unwrap();

    assert!(search.date.is_some());
    assert!(search.details.is_none());
    assert!(search.from_method.is_none());
    assert!(search.tags.is_none());

    // Year search should find nothing on empty DB
    let results = search.search_txs(&mut db_conn).unwrap();
    assert!(results.is_empty());

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn parse_search_fields_month_year() {
    let file_name = "test_parse_search_month.sqlite";
    let mut db_conn = create_test_db(file_name);

    let search = parse_search_fields("2024-06", "", "", "", "", "", "", &db_conn).unwrap();

    assert!(search.date.is_some());
    assert!(search.tags.is_none());

    let results = search.search_txs(&mut db_conn).unwrap();
    assert!(results.is_empty());

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn parse_search_fields_exact_date() {
    let file_name = "test_parse_search_exact.sqlite";
    let db_conn = create_test_db(file_name);

    let search = parse_search_fields("2024-06-15", "", "", "", "", "", "", &db_conn).unwrap();

    assert!(search.date.is_some());

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn parse_search_fields_amount_nature_variants() {
    let file_name = "test_parse_search_amount.sqlite";
    let db_conn = create_test_db(file_name);

    let search = parse_search_fields("", "", "", "", ">100", "", "", &db_conn).unwrap();
    assert!(search.amount.is_some());

    let search = parse_search_fields("", "", "", "", "<50", "", "", &db_conn).unwrap();
    assert!(search.amount.is_some());

    let search = parse_search_fields("", "", "", "", ">=200", "", "", &db_conn).unwrap();
    assert!(search.amount.is_some());

    let search = parse_search_fields("", "", "", "", "<=10", "", "", &db_conn).unwrap();
    assert!(search.amount.is_some());

    let search = parse_search_fields("", "", "", "", "500", "", "", &db_conn).unwrap();
    assert!(search.amount.is_some());

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn parse_search_fields_nonexistent_tags_filtered() {
    let file_name = "test_parse_search_tags.sqlite";
    let db_conn = create_test_db(file_name);

    // Non-existent tags are silently skipped by parse_search_fields
    let search =
        parse_search_fields("", "", "", "", "", "", "FakeTag, AnotherFake", &db_conn).unwrap();
    // tags get filtered to empty because no tag exists in cache (except "Unknown")
    assert!(
        search.tags.is_none() || search.tags.as_ref().unwrap().is_empty(),
        "Non-existent tags should be filtered out, got: {:?}",
        search.tags
    );

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn parse_search_fields_all_empty_returns_empty_search() {
    let file_name = "test_parse_search_empty.sqlite";
    let db_conn = create_test_db(file_name);

    let search = parse_search_fields("", "", "", "", "", "", "", &db_conn).unwrap();

    assert!(search.date.is_none());
    assert!(search.details.is_none());
    assert!(search.tx_type.is_none());
    assert!(search.from_method.is_none());
    assert!(search.to_method.is_none());
    assert!(search.amount.is_none());
    assert!(search.tags.is_none());

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn parse_search_fields_combined_filters() {
    let file_name = "test_parse_search_combined.sqlite";
    let db_conn = create_test_db(file_name);
    let cash_id = db_conn.cache().get_method_id("Cash").unwrap();

    let search =
        parse_search_fields("2024", "Salary", "Cash", "", "", "Income", "", &db_conn).unwrap();

    assert!(search.date.is_some());
    assert_eq!(search.details, Some("Salary"));
    assert_eq!(search.from_method, Some(cash_id));
    assert_eq!(search.tx_type, Some("Income"));

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}
