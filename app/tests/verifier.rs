use rex_app::ui_helper::{DateType, Output, VerifierError};
use std::fs;

use crate::common::create_test_db;

mod common;

// ---- Date verification ----

#[test]
fn verify_date_exact_valid() {
    let file_name = "test_verify_date.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = "2024-06-15".to_string();
    let result = db_conn.verify().date(&mut s, DateType::Exact).unwrap();
    assert!(matches!(result, Output::Accepted(_)));
    assert_eq!(s, "2024-06-15");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn verify_date_empty_returns_nothing() {
    let file_name = "test_verify_date_empty.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = String::new();
    let result = db_conn.verify().date(&mut s, DateType::Exact).unwrap();
    assert!(matches!(result, Output::Nothing(_)));
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn verify_date_year_too_short_is_corrected() {
    let file_name = "test_verify_date_year_short.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = "24-06-15".to_string();
    let result = db_conn.verify().date(&mut s, DateType::Exact);
    assert!(matches!(result, Err(VerifierError::InvalidYear)));
    assert_eq!(s, "2022-06-15");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn verify_date_year_too_long_is_truncated() {
    let file_name = "test_verify_date_year_long.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = "20245-06-15".to_string();
    let result = db_conn.verify().date(&mut s, DateType::Exact);
    assert!(matches!(result, Err(VerifierError::InvalidYear)));
    assert_eq!(s, "2024-06-15");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn verify_date_month_out_of_range_is_capped() {
    let file_name = "test_verify_date_month_big.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = "2024-13-15".to_string();
    let result = db_conn.verify().date(&mut s, DateType::Exact);
    assert!(matches!(result, Err(VerifierError::MonthTooBig)));
    assert_eq!(s, "2024-12-15");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn verify_date_day_out_of_range_is_capped() {
    let file_name = "test_verify_date_day_big.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = "2024-06-32".to_string();
    let result = db_conn.verify().date(&mut s, DateType::Exact);
    assert!(matches!(result, Err(VerifierError::DayTooBig)));
    assert_eq!(s, "2024-06-31");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn verify_date_nonexistent_fails() {
    let file_name = "test_verify_date_nonexistent.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = "2024-02-30".to_string();
    let result = db_conn.verify().date(&mut s, DateType::Exact);
    assert!(matches!(result, Err(VerifierError::NonExistingDate)));
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn verify_date_monthly_valid() {
    let file_name = "test_verify_date_monthly.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = "2024-06".to_string();
    let result = db_conn.verify().date(&mut s, DateType::Monthly).unwrap();
    assert!(matches!(result, Output::Accepted(_)));
    assert_eq!(s, "2024-06");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn verify_date_yearly_valid() {
    let file_name = "test_verify_date_yearly.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = "2024".to_string();
    let result = db_conn.verify().date(&mut s, DateType::Yearly).unwrap();
    assert!(matches!(result, Output::Accepted(_)));
    assert_eq!(s, "2024");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn verify_date_wrong_part_count_is_corrected() {
    let file_name = "test_verify_date_wrong_parts.sqlite";
    let mut db_conn = create_test_db(file_name);

    // Exact expects 3 parts, monthly expects 2
    let mut s = "2024-06".to_string();
    let result = db_conn.verify().date(&mut s, DateType::Exact);
    assert!(matches!(result, Err(VerifierError::InvalidDate)));
    assert_eq!(s, "2022-01-01");

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

// ---- Amount verification ----

#[test]
fn verify_amount_valid() {
    let file_name = "test_verify_amount.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = "42.50".to_string();
    let v = db_conn.verify();
    let result = v.amount(&mut s).unwrap();
    assert!(matches!(result, Output::Accepted(_)));
    assert_eq!(s, "42.50");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn verify_amount_empty_returns_nothing() {
    let file_name = "test_verify_amount_empty.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = String::new();
    let v = db_conn.verify();
    let result = v.amount(&mut s).unwrap();
    assert!(matches!(result, Output::Nothing(_)));
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn verify_amount_integer_gets_decimals() {
    let file_name = "test_verify_amount_integer.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = "10".to_string();
    let v = db_conn.verify();
    v.amount(&mut s).unwrap();
    assert_eq!(s, "10.00");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn verify_amount_one_decimal_gets_padded() {
    let file_name = "test_verify_amount_one_dec.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = "10.5".to_string();
    let v = db_conn.verify();
    v.amount(&mut s).unwrap();
    assert_eq!(s, "10.50");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn verify_amount_negative_becomes_positive_via_calc() {
    let file_name = "test_verify_amount_negative.sqlite";
    let mut db_conn = create_test_db(file_name);
    // '-' is a calc operator, so -5.00 goes through calc path:
    // no left operand → resolves to just the right operand = 5.00
    let mut s = "-5.00".to_string();
    let v = db_conn.verify();
    v.amount(&mut s).unwrap();
    assert_eq!(s, "5.00");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn verify_amount_zero_is_rejected() {
    let file_name = "test_verify_amount_zero.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = "0".to_string();
    let v = db_conn.verify();
    let result = v.amount(&mut s);
    assert!(matches!(result, Err(VerifierError::AmountBelowZero)));
    assert_eq!(s, "0.00");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn verify_amount_calculation_multiplication() {
    let file_name = "test_verify_amount_calc.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = "1+2*3".to_string();
    let v = db_conn.verify();
    v.amount(&mut s).unwrap();
    assert_eq!(s, "7.00");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn verify_amount_calculation_division() {
    let file_name = "test_verify_amount_calc_div.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = "10/2".to_string();
    let v = db_conn.verify();
    v.amount(&mut s).unwrap();
    assert_eq!(s, "5.00");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn verify_amount_calculation_complex() {
    let file_name = "test_verify_amount_calc_complex.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = "10+5*2-3".to_string();
    let v = db_conn.verify();
    v.amount(&mut s).unwrap();
    assert_eq!(s, "17.00");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn verify_amount_non_numeric_chars_stripped() {
    let file_name = "test_verify_amount_strip.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = " $ 1,234.56 ".to_string();
    let v = db_conn.verify();
    v.amount(&mut s).unwrap();
    assert_eq!(s, "1234.56");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

// ---- Tx Method verification ----

#[test]
fn verify_tx_method_exact_match() {
    let file_name = "test_verify_method.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = "Cash".to_string();
    let v = db_conn.verify();
    let result = v.tx_method(&mut s).unwrap();
    assert!(matches!(result, Output::Accepted(_)));
    assert_eq!(s, "Cash");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn verify_tx_method_case_insensitive() {
    let file_name = "test_verify_method_case.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = "cash".to_string();
    let v = db_conn.verify();
    let result = v.tx_method(&mut s).unwrap();
    assert!(matches!(result, Output::Accepted(_)));
    assert_eq!(s, "Cash");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn verify_tx_method_empty() {
    let file_name = "test_verify_method_empty.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = String::new();
    let v = db_conn.verify();
    let result = v.tx_method(&mut s).unwrap();
    assert!(matches!(result, Output::Nothing(_)));
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn verify_tx_method_fuzzy_correction() {
    let file_name = "test_verify_method_fuzzy.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = "Csh".to_string();
    let v = db_conn.verify();
    let result = v.tx_method(&mut s);
    assert!(matches!(result, Err(VerifierError::InvalidTxMethod)));
    assert_eq!(s, "Cash");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

// ---- Tx Type verification ----

#[test]
fn verify_tx_type_empty() {
    let file_name = "test_verify_type_empty.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = String::new();
    let v = db_conn.verify();
    let result = v.tx_type(&mut s).unwrap();
    assert!(matches!(result, Output::Nothing(_)));
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn verify_tx_type_shortcuts() {
    let file_name = "test_verify_type_short.sqlite";
    let mut db_conn = create_test_db(file_name);

    let cases = [
        ("e", "Expense"),
        ("E", "Expense"),
        ("i", "Income"),
        ("I", "Income"),
        ("t", "Transfer"),
        ("T", "Transfer"),
        ("b", "Borrow"),
        ("B", "Borrow"),
        ("l", "Lend"),
        ("L", "Lend"),
        ("br", "Borrow Repay"),
        ("BR", "Borrow Repay"),
        ("lr", "Lend Repay"),
        ("LR", "Lend Repay"),
    ];

    for (input, expected) in cases {
        let mut s = input.to_string();
        let v = db_conn.verify();
        let result = v.tx_type(&mut s).unwrap();
        assert!(
            matches!(result, Output::Accepted(_)),
            "Expected Accepted for input '{input}'"
        );
        assert_eq!(s, expected, "Mismatch for input '{input}'");
    }

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn verify_tx_type_exact_match() {
    let file_name = "test_verify_type_exact.sqlite";
    let mut db_conn = create_test_db(file_name);

    for expected in [
        "Income",
        "Expense",
        "Transfer",
        "Borrow",
        "Lend",
        "Borrow Repay",
        "Lend Repay",
    ] {
        let mut s = expected.to_string();
        let v = db_conn.verify();
        let result = v.tx_type(&mut s).unwrap();
        assert!(matches!(result, Output::Accepted(_)));
        assert_eq!(s, expected);
    }

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn verify_tx_type_fuzzy_correction() {
    let file_name = "test_verify_type_fuzzy.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = "Incom".to_string();
    let v = db_conn.verify();
    let result = v.tx_type(&mut s);
    assert!(matches!(result, Err(VerifierError::InvalidTxType)));
    assert_eq!(s, "Income");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

// ---- Tags verification ----

#[test]
fn verify_tags_dedup_removes_duplicates() {
    let file_name = "test_verify_tags_dedup.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = "A, A, B".to_string();
    let v = db_conn.verify();
    v.tags(&mut s);
    assert_eq!(s, "A, B");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn verify_tags_empty_is_unchanged() {
    let file_name = "test_verify_tags_empty.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = String::new();
    let v = db_conn.verify();
    v.tags(&mut s);
    assert_eq!(s, "");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn verify_tags_trims_whitespace() {
    let file_name = "test_verify_tags_trim.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = " Tag1 , Tag2 , Tag3 ".to_string();
    let v = db_conn.verify();
    v.tags(&mut s);
    assert_eq!(s, "Tag1, Tag2, Tag3");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

// ---- Tags forced verification ----

#[test]
fn verify_tags_forced_all_existing() {
    let file_name = "test_verify_tags_forced.sqlite";
    let mut db_conn = create_test_db(file_name);
    // Tags must exist in DB. Add one via a transaction.
    use crate::common::add_tx;
    add_tx(
        &mut db_conn,
        "2024-01-01",
        "Test",
        "Cash",
        "",
        "10.00",
        "Expense",
        "ExistingTag, AnotherTag",
    );

    let mut s = "ExistingTag, AnotherTag".to_string();
    let v = db_conn.verify();
    let result = v.tags_forced(&mut s).unwrap();
    assert!(matches!(result, Output::Accepted(_)));
    assert_eq!(s, "ExistingTag, AnotherTag");

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn verify_tags_forced_nonexistent_filtered() {
    let file_name = "test_verify_tags_forced_filter.sqlite";
    let mut db_conn = create_test_db(file_name);
    use crate::common::add_tx;
    add_tx(
        &mut db_conn,
        "2024-02-01",
        "Test",
        "Cash",
        "",
        "10.00",
        "Expense",
        "RealTag",
    );

    let mut s = "RealTag, FakeTag".to_string();
    let v = db_conn.verify();
    let result = v.tags_forced(&mut s);
    assert!(matches!(result, Err(VerifierError::NonExistingTag)));
    assert_eq!(s, "RealTag");

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn verify_tags_forced_empty() {
    let file_name = "test_verify_tags_forced_empty.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = String::new();
    let v = db_conn.verify();
    let result = v.tags_forced(&mut s).unwrap();
    assert!(matches!(result, Output::Nothing(_)));
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

// ---- Verifier error paths ----

#[test]
fn verify_date_parsing_error() {
    let file_name = "test_verify_date_parse_err.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = "abc-def-ghi".to_string();
    let result = db_conn.verify().date(&mut s, DateType::Exact);
    assert!(matches!(result, Err(VerifierError::ParsingError(_))));
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn verify_amount_only_symbols_becomes_zero() {
    let file_name = "test_verify_amount_parse_err.sqlite";
    let mut db_conn = create_test_db(file_name);
    // "+" is a calc symbol; empty operands → result is "" → ".00" → 0.00 → AmountBelowZero
    let mut s = "+".to_string();
    let v = db_conn.verify();
    let result = v.amount(&mut s);
    assert!(matches!(result, Err(VerifierError::AmountBelowZero)));
    assert_eq!(s, "0.00");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn verify_tx_method_not_found_fuzzy_corrects() {
    let file_name = "test_verify_method_err.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = "NonExistent".to_string();
    let v = db_conn.verify();
    let result = v.tx_method(&mut s);
    assert!(matches!(result, Err(VerifierError::InvalidTxMethod)));
    // Fuzzy corrects to closest match among Cash/Bank/Other
    assert!(!s.is_empty());
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn verify_tx_type_long_invalid_fuzzy_corrects() {
    let file_name = "test_verify_type_long_err.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = "SomethingWeird".to_string();
    let v = db_conn.verify();
    let result = v.tx_type(&mut s);
    assert!(matches!(result, Err(VerifierError::InvalidTxType)));
    // Gets fuzzy-corrected to closest match
    assert!(!s.is_empty());
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn verify_tx_type_short_invalid_fuzzy_corrects() {
    let file_name = "test_verify_type_short_err.sqlite";
    let mut db_conn = create_test_db(file_name);
    // Short (<2 chars) but not E/I/T/B/L/BR/LR
    let mut s = "x".to_string();
    let v = db_conn.verify();
    let result = v.tx_type(&mut s);
    assert!(matches!(result, Err(VerifierError::InvalidTxType)));
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}
