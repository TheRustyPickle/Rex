use chrono::{Datelike, Local, NaiveDate, Weekday};
use rex_app::conn::{DbConn, FetchNature};
use rex_app::modifier::parse_recurring_tx_fields;
use rex_db::ConnCache;
use rex_db::models::RecurringTx;
use std::fs;

use crate::common::create_test_db;

mod common;

/// Adding or editing a rule already processes everything due, so the number of
/// materialized transactions is simply the number of transactions present. A follow-up
/// explicit run must then have nothing left to do.
fn materialized_count(db_conn: &mut DbConn) -> usize {
    let count = db_conn
        .fetch_txs_with_date(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            FetchNature::All,
        )
        .unwrap()
        .len();

    assert_eq!(db_conn.process_due_recurring_txs().unwrap(), 0);

    count
}

#[test]
fn add_daily_recurring_tx_shows_up_in_list() {
    let file_name = "test_recurring_add_daily.sqlite";
    let mut db_conn = create_test_db(file_name);

    let new_recurring = parse_recurring_tx_fields(
        "2024-06-01",
        "Coffee",
        "Cash",
        "",
        "5.00",
        "Expense",
        "Daily",
        "1",
        "",
        "",
        "",
        &db_conn,
    )
    .unwrap();

    db_conn.add_recurring_tx(new_recurring, "Food").unwrap();

    let all = db_conn.get_recurring_txs().unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].amount.value(), 500);
    // Start date is in the past, so adding the rule already created its due occurrences
    // and advanced the schedule past today
    let today = Local::now().date_naive();
    assert_eq!(all[0].last_recurred_date, Some(today));
    assert!(all[0].next_recurring_date > today);
    assert!(!all[0].is_paused);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn weekly_recurring_tx_rolls_forward_when_start_date_is_a_different_weekday() {
    let file_name = "test_recurring_weekly_mismatch.sqlite";
    let db_conn = create_test_db(file_name);

    // The start date is just an anchor, not a constraint on which weekday is picked.
    // 2024-06-01 is a Saturday - choosing Monday should roll the first occurrence
    // forward to 2024-06-03, not error.
    let new_recurring = parse_recurring_tx_fields(
        "2024-06-01",
        "",
        "Cash",
        "",
        "10.00",
        "Expense",
        "Weekly",
        "1",
        "Monday",
        "",
        "",
        &db_conn,
    )
    .unwrap();

    assert_eq!(
        new_recurring.next_recurring_date,
        NaiveDate::from_ymd_opt(2024, 6, 3).unwrap()
    );

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn process_due_recurring_txs_creates_transaction_and_updates_balance() {
    let file_name = "test_recurring_process_due.sqlite";
    let mut db_conn = create_test_db(file_name);

    let past_date = "2024-01-01";

    let new_recurring = parse_recurring_tx_fields(
        past_date, "Salary", "Cash", "", "1000.00", "Income", "Monthly", "1", "1", "", "", &db_conn,
    )
    .unwrap();

    db_conn.add_recurring_tx(new_recurring, "Work").unwrap();

    let created = materialized_count(&mut db_conn);
    assert!(created >= 1);

    let cash_id = db_conn.cache().get_method_id("Cash").unwrap();
    let balances = db_conn.get_final_balances().unwrap();
    assert!(balances.get(&cash_id).unwrap().balance > 0);

    let recurring = &db_conn.get_recurring_txs().unwrap()[0];
    assert!(recurring.last_recurred_date.is_some());
    assert!(recurring.next_recurring_date > NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn process_due_recurring_txs_catches_up_multiple_missed_daily_occurrences() {
    let file_name = "test_recurring_catch_up.sqlite";
    let mut db_conn = create_test_db(file_name);

    let new_recurring = parse_recurring_tx_fields(
        "2024-01-01",
        "Snack",
        "Cash",
        "",
        "2.00",
        "Expense",
        "Daily",
        "1",
        "",
        "",
        "",
        &db_conn,
    )
    .unwrap();

    db_conn.add_recurring_tx(new_recurring, "Food").unwrap();

    let created = materialized_count(&mut db_conn);

    // From 2024-01-01 to today is a lot more than a handful of days
    assert!(created > 30);

    let tx_view = db_conn
        .fetch_txs_with_date(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            FetchNature::All,
        )
        .unwrap();
    assert_eq!(tx_view.len(), created);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn monthly_recurring_tx_clamps_end_of_month_without_drift() {
    let file_name = "test_recurring_monthly_clamp.sqlite";
    let mut db_conn = create_test_db(file_name);

    let new_recurring = parse_recurring_tx_fields(
        "2024-01-31",
        "Rent",
        "Cash",
        "",
        "100.00",
        "Expense",
        "Monthly",
        "1",
        "31",
        "",
        "",
        &db_conn,
    )
    .unwrap();

    db_conn.add_recurring_tx(new_recurring, "Bills").unwrap();
    materialized_count(&mut db_conn);

    let tx_view = db_conn
        .fetch_txs_with_date(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            FetchNature::All,
        )
        .unwrap();

    let dates: Vec<NaiveDate> = (0..tx_view.len())
        .map(|i| tx_view.get_tx(i).date.date())
        .collect();

    assert!(dates.contains(&NaiveDate::from_ymd_opt(2024, 1, 31).unwrap()));
    assert!(dates.contains(&NaiveDate::from_ymd_opt(2024, 2, 29).unwrap()));
    assert!(dates.contains(&NaiveDate::from_ymd_opt(2024, 3, 31).unwrap()));

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn paused_recurring_tx_is_skipped_by_processing() {
    let file_name = "test_recurring_paused_skip.sqlite";
    let mut db_conn = create_test_db(file_name);

    let new_recurring = parse_recurring_tx_fields(
        "2024-01-01",
        "",
        "Cash",
        "",
        "5.00",
        "Expense",
        "Daily",
        "1",
        "",
        "",
        "",
        &db_conn,
    )
    .unwrap();

    db_conn.add_recurring_tx(new_recurring, "").unwrap();

    // Adding already processed everything due. Pause it, then rewind its schedule so it is
    // due again - a paused rule must still be skipped.
    let before = materialized_count(&mut db_conn);
    assert!(before > 0);

    let id = db_conn.get_recurring_txs().unwrap()[0].id;
    db_conn.set_recurring_paused(id, true).unwrap();
    RecurringTx::update_schedule(
        id,
        None,
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        &mut db_conn,
    )
    .unwrap();

    let created = db_conn.process_due_recurring_txs().unwrap();
    assert_eq!(created, 0);
    assert_eq!(materialized_count(&mut db_conn), before);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn edit_recurring_tx_updates_fields() {
    let file_name = "test_recurring_edit.sqlite";
    let mut db_conn = create_test_db(file_name);

    let new_recurring = parse_recurring_tx_fields(
        "2024-06-01",
        "Coffee",
        "Cash",
        "",
        "5.00",
        "Expense",
        "Daily",
        "1",
        "",
        "",
        "",
        &db_conn,
    )
    .unwrap();

    db_conn.add_recurring_tx(new_recurring, "Food").unwrap();
    let id = db_conn.get_recurring_txs().unwrap()[0].id;

    let updated = parse_recurring_tx_fields(
        "2024-06-01",
        "Big Coffee",
        "Cash",
        "",
        "8.00",
        "Expense",
        "Daily",
        "2",
        "",
        "",
        "",
        &db_conn,
    )
    .unwrap();

    db_conn.edit_recurring_tx(id, updated, "Food").unwrap();

    let all = db_conn.get_recurring_txs().unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].amount.value(), 800);
    assert_eq!(all[0].details.as_deref(), Some("Big Coffee"));
    assert_eq!(all[0].recur_interval, 2);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn delete_recurring_tx_keeps_previously_materialized_transactions() {
    let file_name = "test_recurring_delete.sqlite";
    let mut db_conn = create_test_db(file_name);

    let new_recurring = parse_recurring_tx_fields(
        "2024-01-01",
        "Snack",
        "Cash",
        "",
        "2.00",
        "Expense",
        "Daily",
        "1",
        "",
        "",
        "",
        &db_conn,
    )
    .unwrap();

    db_conn.add_recurring_tx(new_recurring, "Food").unwrap();
    let created = materialized_count(&mut db_conn);
    assert!(created > 0);

    let id = db_conn.get_recurring_txs().unwrap()[0].id;
    db_conn.delete_recurring_tx(id).unwrap();

    assert!(db_conn.get_recurring_txs().unwrap().is_empty());

    let tx_view = db_conn
        .fetch_txs_with_date(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            FetchNature::All,
        )
        .unwrap();
    assert_eq!(tx_view.len(), created);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn end_date_stops_generating_further_occurrences() {
    let file_name = "test_recurring_end_date.sqlite";
    let mut db_conn = create_test_db(file_name);

    let new_recurring = parse_recurring_tx_fields(
        "2024-01-01",
        "",
        "Cash",
        "",
        "1.00",
        "Expense",
        "Daily",
        "1",
        "",
        "",
        "2024-01-05",
        &db_conn,
    )
    .unwrap();

    db_conn.add_recurring_tx(new_recurring, "").unwrap();
    let created = materialized_count(&mut db_conn);

    // 2024-01-01 through 2024-01-05 inclusive = 5 occurrences, regardless of how long
    // ago that end date is relative to "today"
    assert_eq!(created, 5);

    // Running it again should not create any more
    let created_again = db_conn.process_due_recurring_txs().unwrap();
    assert_eq!(created_again, 0);

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn yearly_recurring_tx_across_leap_day() {
    let file_name = "test_recurring_yearly_leap.sqlite";
    let mut db_conn = create_test_db(file_name);

    let new_recurring = parse_recurring_tx_fields(
        "2024-02-29",
        "Anniversary",
        "Cash",
        "",
        "50.00",
        "Expense",
        "Yearly",
        "1",
        "29",
        "February",
        "",
        &db_conn,
    )
    .unwrap();

    db_conn.add_recurring_tx(new_recurring, "").unwrap();
    materialized_count(&mut db_conn);

    let tx_view = db_conn
        .fetch_txs_with_date(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            FetchNature::All,
        )
        .unwrap();

    let dates: Vec<NaiveDate> = (0..tx_view.len())
        .map(|i| tx_view.get_tx(i).date.date())
        .collect();

    assert!(dates.contains(&NaiveDate::from_ymd_opt(2024, 2, 29).unwrap()));
    assert!(dates.contains(&NaiveDate::from_ymd_opt(2025, 2, 28).unwrap()));

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn weekly_recurring_tx_fires_only_on_the_chosen_weekday() {
    let file_name = "test_recurring_weekly_fires.sqlite";
    let mut db_conn = create_test_db(file_name);

    // 2024-01-01 is a Monday; choosing Friday should roll the first occurrence
    // forward to 2024-01-05, then every 7 days after that.
    let new_recurring = parse_recurring_tx_fields(
        "2024-01-01",
        "Standing order",
        "Cash",
        "",
        "20.00",
        "Expense",
        "Weekly",
        "1",
        "Friday",
        "",
        "",
        &db_conn,
    )
    .unwrap();

    assert_eq!(
        new_recurring.next_recurring_date,
        NaiveDate::from_ymd_opt(2024, 1, 5).unwrap()
    );

    db_conn.add_recurring_tx(new_recurring, "Bills").unwrap();
    let created = materialized_count(&mut db_conn);
    assert!(created > 0);

    let tx_view = db_conn
        .fetch_txs_with_date(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            FetchNature::All,
        )
        .unwrap();

    let dates: Vec<NaiveDate> = (0..tx_view.len())
        .map(|i| tx_view.get_tx(i).date.date())
        .collect();

    assert_eq!(dates.len(), created);
    assert_eq!(dates[0], NaiveDate::from_ymd_opt(2024, 1, 5).unwrap());

    for date in &dates {
        assert_eq!(date.weekday(), Weekday::Fri);
    }

    // Every occurrence exactly 7 days after the previous one - no drift
    for pair in dates.windows(2) {
        assert_eq!((pair[1] - pair[0]).num_days(), 7);
    }

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn biweekly_recurring_tx_spaces_occurrences_two_weeks_apart() {
    let file_name = "test_recurring_biweekly.sqlite";
    let mut db_conn = create_test_db(file_name);

    // 2024-01-01 is already a Monday, so it satisfies the rule immediately
    let new_recurring = parse_recurring_tx_fields(
        "2024-01-01",
        "Paycheck",
        "Cash",
        "",
        "500.00",
        "Income",
        "Weekly",
        "2",
        "Monday",
        "",
        "",
        &db_conn,
    )
    .unwrap();

    assert_eq!(
        new_recurring.next_recurring_date,
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()
    );

    db_conn.add_recurring_tx(new_recurring, "Work").unwrap();
    let created = materialized_count(&mut db_conn);
    assert!(created > 0);

    let tx_view = db_conn
        .fetch_txs_with_date(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            FetchNature::All,
        )
        .unwrap();

    let dates: Vec<NaiveDate> = (0..tx_view.len())
        .map(|i| tx_view.get_tx(i).date.date())
        .collect();

    for date in &dates {
        assert_eq!(date.weekday(), Weekday::Mon);
    }

    // Every-2-weeks: exactly 14 days apart, never 7
    for pair in dates.windows(2) {
        assert_eq!((pair[1] - pair[0]).num_days(), 14);
    }

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}
