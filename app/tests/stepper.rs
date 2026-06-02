use rex_app::ui_helper::{DateType, StepType};
use std::fs;

use crate::common::{add_tx, create_test_db};

mod common;

// ---- Date stepping ----

#[test]
fn step_date_exact_up() {
    let file_name = "test_step_date.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = "2024-06-15".to_string();
    db_conn
        .step()
        .date(&mut s, StepType::StepUp, DateType::Exact)
        .unwrap();
    assert_eq!(s, "2024-06-16");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn step_date_exact_down() {
    let file_name = "test_step_date_down.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = "2024-06-15".to_string();
    db_conn
        .step()
        .date(&mut s, StepType::StepDown, DateType::Exact)
        .unwrap();
    assert_eq!(s, "2024-06-14");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn step_date_monthly_up() {
    let file_name = "test_step_date_monthly.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = "2024-06".to_string();
    db_conn
        .step()
        .date(&mut s, StepType::StepUp, DateType::Monthly)
        .unwrap();
    assert_eq!(s, "2024-07");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn step_date_monthly_down() {
    let file_name = "test_step_date_monthly_down.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = "2024-06".to_string();
    db_conn
        .step()
        .date(&mut s, StepType::StepDown, DateType::Monthly)
        .unwrap();
    assert_eq!(s, "2024-05");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn step_date_yearly_up() {
    let file_name = "test_step_date_yearly.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = "2024".to_string();
    db_conn
        .step()
        .date(&mut s, StepType::StepUp, DateType::Yearly)
        .unwrap();
    assert_eq!(s, "2025");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn step_date_yearly_down() {
    let file_name = "test_step_date_yearly_down.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = "2024".to_string();
    db_conn
        .step()
        .date(&mut s, StepType::StepDown, DateType::Yearly)
        .unwrap();
    assert_eq!(s, "2023");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn step_date_from_empty_defaults() {
    let file_name = "test_step_date_empty.sqlite";
    let mut db_conn = create_test_db(file_name);

    let mut s = String::new();
    db_conn
        .step()
        .date(&mut s, StepType::StepUp, DateType::Exact)
        .unwrap();
    assert_eq!(s, "2022-01-01");

    let mut s = String::new();
    db_conn
        .step()
        .date(&mut s, StepType::StepUp, DateType::Monthly)
        .unwrap();
    assert_eq!(s, "2022-01");

    let mut s = String::new();
    db_conn
        .step()
        .date(&mut s, StepType::StepUp, DateType::Yearly)
        .unwrap();
    assert_eq!(s, "2022");

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn step_date_month_boundary_wraps_year() {
    let file_name = "test_step_date_month_wrap.sqlite";
    let mut db_conn = create_test_db(file_name);

    let mut s = "2024-12".to_string();
    db_conn
        .step()
        .date(&mut s, StepType::StepUp, DateType::Monthly)
        .unwrap();
    assert_eq!(s, "2025-01");

    let mut s = "2024-01".to_string();
    db_conn
        .step()
        .date(&mut s, StepType::StepDown, DateType::Monthly)
        .unwrap();
    assert_eq!(s, "2023-12");

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

// ---- Amount stepping ----

#[test]
fn step_amount_up() {
    let file_name = "test_step_amount.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = "5.00".to_string();
    db_conn.step().amount(&mut s, StepType::StepUp).unwrap();
    assert_eq!(s, "6.00");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn step_amount_down() {
    let file_name = "test_step_amount_down.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = "5.00".to_string();
    db_conn.step().amount(&mut s, StepType::StepDown).unwrap();
    assert_eq!(s, "4.00");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn step_amount_down_hits_floor() {
    let file_name = "test_step_amount_floor.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = "0.00".to_string();
    db_conn.step().amount(&mut s, StepType::StepDown).unwrap();
    assert_eq!(s, "0.00");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn step_amount_from_empty_defaults() {
    let file_name = "test_step_amount_empty.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = String::new();
    db_conn.step().amount(&mut s, StepType::StepUp).unwrap();
    assert_eq!(s, "0.00");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn step_amount_negative_restores_to_one() {
    let file_name = "test_step_amount_negative.sqlite";
    let mut db_conn = create_test_db(file_name);
    // After previous verification corrects negative to positive, stepping up
    // from a state where VerifierError::AmountBelowZero would be returned
    // should set the amount to 1.00. The easiest way to trigger this is
    // to call with "0.00" which after verify returns AmountBelowZero.
    // But 0.00 is caught by the stepping branch: if step_up && AmountBelowZero → 1.00
    // Actually '0.00' amount verify returns AmountBelowZero.
    // Let's verify: stepping up on "0.00" → verify rejects → AmountBelowZero, StepUp → sets "1.00"
    let mut s = "0.00".to_string();
    db_conn.step().amount(&mut s, StepType::StepUp).unwrap();
    assert_eq!(s, "1.00");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

// ---- Tx Method stepping ----

#[test]
fn step_tx_method_up() {
    let file_name = "test_step_method.sqlite";
    let mut db_conn = create_test_db(file_name);
    // Methods: Cash(1), Bank(2), Other(3)
    let mut s = "Cash".to_string();
    db_conn.step().tx_method(&mut s, StepType::StepUp).unwrap();
    assert_eq!(s, "Bank");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn step_tx_method_wraps_at_end() {
    let file_name = "test_step_method_wrap.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = "Other".to_string();
    db_conn.step().tx_method(&mut s, StepType::StepUp).unwrap();
    assert_eq!(s, "Cash");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn step_tx_method_down_wraps_at_start() {
    let file_name = "test_step_method_wrap_down.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = "Cash".to_string();
    db_conn
        .step()
        .tx_method(&mut s, StepType::StepDown)
        .unwrap();
    assert_eq!(s, "Other");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn step_tx_method_from_empty() {
    let file_name = "test_step_method_empty.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = String::new();
    db_conn.step().tx_method(&mut s, StepType::StepUp).unwrap();
    assert_eq!(s, "Cash");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

// ---- Tx Type stepping ----

#[test]
fn step_tx_type_up() {
    let file_name = "test_step_type.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = "Income".to_string();
    db_conn.step().tx_type(&mut s, StepType::StepUp).unwrap();
    assert_eq!(s, "Expense");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn step_tx_type_down_wraps() {
    let file_name = "test_step_type_wrap.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = "Income".to_string();
    db_conn.step().tx_type(&mut s, StepType::StepDown).unwrap();
    // TxType order: Income, Expense, Transfer, Borrow, Lend, BorrowRepay, LendRepay
    // Wrapping down from first goes to last
    assert_eq!(s, "Lend Repay");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn step_tx_type_cycles_all() {
    let file_name = "test_step_type_cycle.sqlite";
    let mut db_conn = create_test_db(file_name);
    let expected = [
        "Income",
        "Expense",
        "Transfer",
        "Borrow",
        "Lend",
        "Borrow Repay",
        "Lend Repay",
    ];

    let mut s = "Income".to_string();
    (1..expected.len()).for_each(|i| {
        db_conn.step().tx_type(&mut s, StepType::StepUp).unwrap();
        assert_eq!(s, expected[i]);
    });

    // Wrap back to first
    db_conn.step().tx_type(&mut s, StepType::StepUp).unwrap();
    assert_eq!(s, "Income");

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn step_tx_type_from_empty() {
    let file_name = "test_step_type_empty.sqlite";
    let mut db_conn = create_test_db(file_name);
    let mut s = String::new();
    db_conn.step().tx_type(&mut s, StepType::StepUp).unwrap();
    assert_eq!(s, "Income");
    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

// ---- Tag stepping ----

#[test]
fn step_tag_up() {
    let file_name = "test_step_tag.sqlite";
    let mut db_conn = create_test_db(file_name);

    // Add tags to the DB
    add_tx(
        &mut db_conn,
        "2024-01-01",
        "T1",
        "Cash",
        "",
        "10.00",
        "Expense",
        "Alpha",
    );
    add_tx(
        &mut db_conn,
        "2024-01-01",
        "T2",
        "Cash",
        "",
        "20.00",
        "Expense",
        "Beta",
    );
    add_tx(
        &mut db_conn,
        "2024-01-01",
        "T3",
        "Cash",
        "",
        "30.00",
        "Expense",
        "Gamma",
    );

    let mut s = "Alpha".to_string();
    db_conn.step().tag(&mut s, StepType::StepUp).unwrap();
    assert_eq!(s, "Beta");

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn step_tag_wraps_at_end() {
    let file_name = "test_step_tag_wrap.sqlite";
    let mut db_conn = create_test_db(file_name);

    add_tx(
        &mut db_conn,
        "2024-02-01",
        "T1",
        "Cash",
        "",
        "10.00",
        "Expense",
        "Alpha",
    );
    add_tx(
        &mut db_conn,
        "2024-02-01",
        "T2",
        "Cash",
        "",
        "20.00",
        "Expense",
        "Zulu",
    );

    let mut s = "Zulu".to_string();
    db_conn.step().tag(&mut s, StepType::StepUp).unwrap();
    assert_eq!(s, "Alpha");

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn step_tag_down_wraps() {
    let file_name = "test_step_tag_wrap_down.sqlite";
    let mut db_conn = create_test_db(file_name);

    add_tx(
        &mut db_conn,
        "2024-03-01",
        "T1",
        "Cash",
        "",
        "10.00",
        "Expense",
        "Alpha",
    );
    add_tx(
        &mut db_conn,
        "2024-03-01",
        "T2",
        "Cash",
        "",
        "20.00",
        "Expense",
        "Beta",
    );

    // Tags include pre-seeded "Unknown": ["Alpha", "Beta", "Unknown"]
    // Step down from Alpha (index 0) wraps to last
    let mut s = "Alpha".to_string();
    db_conn.step().tag(&mut s, StepType::StepDown).unwrap();
    assert_eq!(s, "Unknown");

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn step_tag_from_empty() {
    let file_name = "test_step_tag_empty.sqlite";
    let mut db_conn = create_test_db(file_name);

    add_tx(
        &mut db_conn,
        "2024-04-01",
        "T1",
        "Cash",
        "",
        "10.00",
        "Expense",
        "Alpha",
    );

    let mut s = String::new();
    db_conn.step().tag(&mut s, StepType::StepUp).unwrap();
    assert_eq!(s, "Alpha");

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn step_tag_fuzzy_corrects_then_steps() {
    let file_name = "test_step_tag_fuzzy.sqlite";
    let mut db_conn = create_test_db(file_name);

    add_tx(
        &mut db_conn,
        "2024-05-01",
        "T1",
        "Cash",
        "",
        "10.00",
        "Expense",
        "Groceries",
    );
    add_tx(
        &mut db_conn,
        "2024-05-01",
        "T2",
        "Cash",
        "",
        "20.00",
        "Expense",
        "Rent",
    );

    // Invalid tag gets fuzzy-corrected, then stepped
    let mut s = "Groce".to_string();
    let result = db_conn.step().tag(&mut s, StepType::StepUp);
    assert!(result.is_err()); // Invalid tag error
    assert_eq!(s, "Groceries");

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn step_tag_multiple_preserves_other_tags() {
    let file_name = "test_step_tag_multi.sqlite";
    let mut db_conn = create_test_db(file_name);

    add_tx(
        &mut db_conn,
        "2024-06-01",
        "T1",
        "Cash",
        "",
        "10.00",
        "Expense",
        "Alpha",
    );
    add_tx(
        &mut db_conn,
        "2024-06-01",
        "T2",
        "Cash",
        "",
        "20.00",
        "Expense",
        "Beta",
    );
    add_tx(
        &mut db_conn,
        "2024-06-01",
        "T3",
        "Cash",
        "",
        "30.00",
        "Expense",
        "Gamma",
    );

    // Only the last tag in a comma-separated list gets stepped
    let mut s = "Alpha, Beta".to_string();
    db_conn.step().tag(&mut s, StepType::StepUp).unwrap();
    assert_eq!(s, "Alpha, Gamma");

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}
