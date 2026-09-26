use anyhow::{Context, Result};
use chrono::{Datelike, Days, Local, Months, NaiveDate, NaiveTime};
use rex_db::ConnCache;
use rex_db::models::{
    FullRecurringTx, NewRecurringTx, NewTag, NewTx, RecurrenceFrequency, RecurringTx,
    RecurringTxTag, Tag,
};

use crate::modifier::{activity_new_tx, add_new_tx};
use crate::utils::split_tags;

pub(crate) fn add_new_recurring_tx(
    new: NewRecurringTx,
    tags: &str,
    db_conn: &mut impl ConnCache,
) -> Result<Vec<Tag>> {
    let mut tag_list = split_tags(tags);

    if tag_list.is_empty() {
        tag_list.push("Unknown".to_string());
    }

    let added = new.insert(db_conn).context("Failed on new recurring tx")?;

    let mut new_tags = Vec::new();
    let mut recurring_tx_tags = Vec::new();

    for (index, tag) in tag_list.into_iter().enumerate() {
        if let Ok(tag_id) = db_conn.cache().get_tag_id(&tag) {
            recurring_tx_tags.push(RecurringTxTag::new(added.id, tag_id, index == 0));
            continue;
        }

        let tag_data = NewTag::new(&tag)
            .insert(db_conn)
            .context("Failed on new tag")?;

        new_tags.push(tag_data.clone());
        recurring_tx_tags.push(RecurringTxTag::new(added.id, tag_data.id, index == 0));
    }

    RecurringTxTag::insert_batch(recurring_tx_tags, db_conn)?;

    Ok(new_tags)
}

pub(crate) fn edit_recurring_tx(
    id: i32,
    new: NewRecurringTx,
    tags: &str,
    db_conn: &mut impl ConnCache,
) -> Result<Vec<Tag>> {
    RecurringTx::update(id, &new, db_conn).context("Failed to update recurring tx")?;

    RecurringTxTag::delete_by_recurring_tx_id(id, db_conn)?;

    let mut tag_list = split_tags(tags);

    if tag_list.is_empty() {
        tag_list.push("Unknown".to_string());
    }

    let mut new_tags = Vec::new();
    let mut recurring_tx_tags = Vec::new();

    for (index, tag) in tag_list.into_iter().enumerate() {
        if let Ok(tag_id) = db_conn.cache().get_tag_id(&tag) {
            recurring_tx_tags.push(RecurringTxTag::new(id, tag_id, index == 0));
            continue;
        }

        let tag_data = NewTag::new(&tag)
            .insert(db_conn)
            .context("Failed on new tag")?;

        new_tags.push(tag_data.clone());
        recurring_tx_tags.push(RecurringTxTag::new(id, tag_data.id, index == 0));
    }

    RecurringTxTag::insert_batch(recurring_tx_tags, db_conn)?;

    Ok(new_tags)
}

pub(crate) fn delete_recurring_tx(id: i32, db_conn: &mut impl ConnCache) -> Result<()> {
    RecurringTx::delete_by_id(id, db_conn)?;

    Ok(())
}

pub(crate) fn process_one_due_tx(
    rule: &FullRecurringTx,
    occurrence_date: NaiveDate,
    db_conn: &mut impl ConnCache,
) -> Result<Vec<Tag>> {
    let local_now = Local::now().naive_local();

    let date = if occurrence_date == local_now.date() {
        local_now
    } else {
        occurrence_date.and_time(NaiveTime::MIN)
    };

    let tx_type_str = rule.tx_type.to_string();

    let tags_str = rule
        .tags
        .iter()
        .map(|t| t.name.as_str())
        .collect::<Vec<&str>>()
        .join(", ");

    let new_tx = NewTx::new(
        date,
        rule.details.as_deref(),
        rule.from_method.id,
        rule.to_method.as_ref().map(|m| m.id),
        rule.amount.value(),
        &tx_type_str,
    );

    let new_tags = add_new_tx(new_tx.clone(), &tags_str, None, db_conn)
        .context("Failed to create recurring tx")?;

    activity_new_tx(&new_tx, &tags_str, db_conn)?;

    Ok(new_tags)
}

/// Returns the last valid day of `year`-`month`, so a stored day like 31 can be clamped
/// down for months that don't have it (e.g. February).
fn last_day_of_month(year: i32, month: u32) -> u32 {
    let next_month_first = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .unwrap();

    (next_month_first - Days::new(1)).day()
}

// If day is set to 31 but the month has 28 or 30 days, work with the minimum of the two so it
// doesn't start pointing to an invalid day of the month.
fn clamp_day(year: i32, month: u32, day: i32) -> NaiveDate {
    let clamped_day = (day as u32).min(last_day_of_month(year, month));

    NaiveDate::from_ymd_opt(year, month, clamped_day).unwrap()
}

fn step_month(year: i32, month: u32, months_to_add: i32) -> (i32, u32) {
    let start = NaiveDate::from_ymd_opt(year, month, 1).unwrap();

    let stepped = start
        .checked_add_months(Months::new(months_to_add as u32))
        .unwrap();

    (stepped.year(), stepped.month())
}

/// Advances one interval forward from an already-valid occurrence date. `recur_value` must
/// be `Some` for Weekly/Monthly/Yearly and `recur_month` must be `Some` for Yearly
pub(crate) fn advance_date(
    current: NaiveDate,
    frequency: RecurrenceFrequency,
    interval: i32,
    recur_value: Option<i32>,
    recur_month: Option<i32>,
) -> NaiveDate {
    match frequency {
        RecurrenceFrequency::Daily => current + Days::new(interval as u64),
        RecurrenceFrequency::Weekly => current + Days::new(7 * interval as u64),
        RecurrenceFrequency::Monthly => {
            let (year, month) = step_month(current.year(), current.month(), interval);
            clamp_day(year, month, recur_value.unwrap())
        }
        RecurrenceFrequency::Yearly => clamp_day(
            current.year() + interval,
            recur_month.unwrap() as u32,
            recur_value.unwrap(),
        ),
    }
}

pub(crate) fn first_occurrence_on_or_after(
    start: NaiveDate,
    frequency: RecurrenceFrequency,
    recur_value: Option<i32>,
    recur_month: Option<i32>,
) -> NaiveDate {
    match frequency {
        RecurrenceFrequency::Daily => start,
        RecurrenceFrequency::Weekly => {
            let target_weekday = recur_value.unwrap();
            let start_weekday = start.weekday().num_days_from_sunday() as i32;
            let days_to_add = (target_weekday - start_weekday).rem_euclid(7);

            start + Days::new(days_to_add as u64)
        }
        RecurrenceFrequency::Monthly => {
            let target_this_month = clamp_day(start.year(), start.month(), recur_value.unwrap());

            if target_this_month >= start {
                target_this_month
            } else {
                let (year, month) = step_month(start.year(), start.month(), 1);
                clamp_day(year, month, recur_value.unwrap())
            }
        }
        RecurrenceFrequency::Yearly => {
            let month = recur_month.unwrap() as u32;
            let target_this_year = clamp_day(start.year(), month, recur_value.unwrap());

            if target_this_year >= start {
                target_this_year
            } else {
                clamp_day(start.year() + 1, month, recur_value.unwrap())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    #[test]
    fn clamp_day_keeps_valid_day() {
        assert_eq!(clamp_day(2026, 1, 15), date(2026, 1, 15));
    }

    #[test]
    fn clamp_day_clamps_to_february_in_non_leap_year() {
        assert_eq!(clamp_day(2026, 2, 31), date(2026, 2, 28));
    }

    #[test]
    fn clamp_day_clamps_to_february_in_leap_year() {
        assert_eq!(clamp_day(2024, 2, 31), date(2024, 2, 29));
    }

    #[test]
    fn advance_date_daily_steps_by_interval() {
        let next = advance_date(date(2026, 1, 30), RecurrenceFrequency::Daily, 5, None, None);
        assert_eq!(next, date(2026, 2, 4));
    }

    #[test]
    fn advance_date_weekly_preserves_weekday_across_month_boundary() {
        // 2026-01-28 is a Wednesday, every-2-weeks should land on the next Wednesday twice over
        let start = date(2026, 1, 28);
        let next = advance_date(
            start,
            RecurrenceFrequency::Weekly,
            2,
            Some(start.weekday().num_days_from_sunday() as i32),
            None,
        );
        assert_eq!(next, date(2026, 2, 11));
        assert_eq!(next.weekday(), start.weekday());
    }

    #[test]
    fn advance_date_monthly_does_not_drift_after_clamping() {
        // 31st of every month: Jan 31 -> Feb 28 (clamped) -> back to Mar 31 (no drift)
        let jan_31 = date(2026, 1, 31);
        let feb = advance_date(jan_31, RecurrenceFrequency::Monthly, 1, Some(31), None);
        assert_eq!(feb, date(2026, 2, 28));

        let mar = advance_date(feb, RecurrenceFrequency::Monthly, 1, Some(31), None);
        assert_eq!(mar, date(2026, 3, 31));
    }

    #[test]
    fn advance_date_monthly_interval_crosses_year_boundary() {
        // Quarterly (every 3 months) starting in November
        let nov = date(2026, 11, 15);
        let next = advance_date(nov, RecurrenceFrequency::Monthly, 3, Some(15), None);
        assert_eq!(next, date(2027, 2, 15));
    }

    #[test]
    fn advance_date_yearly_clamps_leap_day() {
        let leap_day = date(2024, 2, 29);
        let next = advance_date(leap_day, RecurrenceFrequency::Yearly, 1, Some(29), Some(2));
        assert_eq!(next, date(2025, 2, 28));
    }

    #[test]
    fn first_occurrence_weekly_start_already_matches() {
        let start = date(2026, 3, 16);
        let target_weekday = start.weekday().num_days_from_sunday() as i32;

        let first = first_occurrence_on_or_after(
            start,
            RecurrenceFrequency::Weekly,
            Some(target_weekday),
            None,
        );
        assert_eq!(first, start);
    }

    #[test]
    fn first_occurrence_weekly_rolls_forward_to_chosen_weekday() {
        // The start date's own weekday doesn't constrain which day can be chosen - it's
        // just an anchor. Picking a day 2 days ahead in the week should roll forward, not error.
        let start = date(2026, 3, 14);
        let start_weekday = start.weekday().num_days_from_sunday() as i32;
        let target_weekday = (start_weekday + 2).rem_euclid(7);

        let first = first_occurrence_on_or_after(
            start,
            RecurrenceFrequency::Weekly,
            Some(target_weekday),
            None,
        );
        assert_eq!(first, start + Days::new(2));
    }

    #[test]
    fn first_occurrence_weekly_wraps_to_next_week_when_day_already_passed() {
        let start = date(2026, 3, 14);
        let start_weekday = start.weekday().num_days_from_sunday() as i32;
        // A day that's "behind" in the week relative to start wraps all the way to next week
        let target_weekday = (start_weekday + 6).rem_euclid(7);

        let first = first_occurrence_on_or_after(
            start,
            RecurrenceFrequency::Weekly,
            Some(target_weekday),
            None,
        );
        assert_eq!(first, start + Days::new(6));
    }

    #[test]
    fn first_occurrence_monthly_start_already_matches() {
        let start = date(2026, 3, 15);
        let first =
            first_occurrence_on_or_after(start, RecurrenceFrequency::Monthly, Some(15), None);
        assert_eq!(first, start);
    }

    #[test]
    fn first_occurrence_monthly_target_still_ahead_this_month() {
        let start = date(2026, 3, 10);
        let first =
            first_occurrence_on_or_after(start, RecurrenceFrequency::Monthly, Some(20), None);
        assert_eq!(first, date(2026, 3, 20));
    }

    #[test]
    fn first_occurrence_monthly_target_already_passed_rolls_to_next_month() {
        let start = date(2026, 3, 25);
        let first =
            first_occurrence_on_or_after(start, RecurrenceFrequency::Monthly, Some(5), None);
        assert_eq!(first, date(2026, 4, 5));
    }

    #[test]
    fn first_occurrence_yearly_rolls_to_next_year() {
        let start = date(2026, 6, 1);
        let first =
            first_occurrence_on_or_after(start, RecurrenceFrequency::Yearly, Some(1), Some(1));
        assert_eq!(first, date(2027, 1, 1));
    }
}
