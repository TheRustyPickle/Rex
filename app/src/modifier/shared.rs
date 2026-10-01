use anyhow::{Result, anyhow};
use chrono::{Days, Local, Months, NaiveDate, NaiveTime};
use rex_db::ConnCache;
use rex_db::models::{
    Balance, DateNature, FetchNature, NewRecurringTx, NewSearch, NewTx, RecurrenceFrequency, Tx,
    TxType,
};
use rex_shared::models::{Dollar, LAST_POSSIBLE_TIME};

use crate::modifier::first_occurrence_on_or_after;
use crate::utils::{month_name_to_num, parse_amount_nature_cent, weekday_name_to_num};

pub(crate) fn tidy_balances(date: NaiveDate, db_conn: &mut impl ConnCache) -> Result<()> {
    let tx = Balance::get_highest_date(db_conn)?;

    let max_date = NaiveDate::from_ymd_opt(tx.year, tx.month as u32, 1).unwrap();

    tidy_recursive(max_date, date, db_conn)?;

    // A bad delete can corrupt the final balance while the monthly ones stay
    // correct, so whenever the two disagree, trust the latest monthly balance.
    // This issue is from an older version which has since been fixed and should not occur again.
    // This is just as a safety net.
    let mut final_balance = Balance::get_final_balance(db_conn)?;
    let balance_highest_date = Balance::get_balance_highest_date(db_conn)?;

    for balance in balance_highest_date {
        let mut final_balance_entry = final_balance.get_mut(&balance.method_id).unwrap().clone();

        if final_balance_entry.balance != balance.balance {
            final_balance_entry.balance = balance.balance;
            final_balance_entry.insert(db_conn)?;
        }
    }

    Ok(())
}

/// Recursively tidy balances from the given date to the max date
fn tidy_recursive(
    max_date: NaiveDate,
    date: NaiveDate,
    db_conn: &mut impl ConnCache,
) -> Result<()> {
    let nature = FetchNature::Monthly;

    let txs = Tx::get_txs(date, nature, db_conn)?;

    let current_balance = Balance::get_balance(date, nature, db_conn)?;

    let mut last_balance = Balance::get_last_balance(date, nature, db_conn)?;

    for tx in txs {
        match tx.tx_type.as_str().into() {
            TxType::Income | TxType::Borrow | TxType::LendRepay => {
                let method_id = tx.from_method;
                *last_balance.get_mut(&method_id).unwrap() += tx.amount;
            }
            TxType::Expense | TxType::Lend | TxType::BorrowRepay => {
                let method_id = tx.from_method;
                *last_balance.get_mut(&method_id).unwrap() -= tx.amount;
            }

            TxType::Transfer => {
                let from_method_id = tx.from_method;
                let to_method_id = tx.to_method.as_ref().unwrap();

                *last_balance.get_mut(&from_method_id).unwrap() -= tx.amount;
                *last_balance.get_mut(to_method_id).unwrap() += tx.amount;
            }
        }
    }

    let mut to_insert_balance = Vec::new();

    for mut balance in current_balance {
        let method_id = balance.method_id;
        let last_balance = *last_balance.get(&method_id).unwrap();

        if balance.balance != last_balance {
            balance.balance = last_balance.value();
            to_insert_balance.push(balance);
        }
    }

    for to_insert in to_insert_balance {
        to_insert.insert(db_conn)?;
    }

    if date <= max_date {
        let next_date = date + Months::new(1);
        tidy_recursive(max_date, next_date, db_conn)?;
    }

    Ok(())
}

pub fn parse_tx_fields<'a>(
    date: &'a str,
    details: &'a str,
    from_method: &'a str,
    to_method: &'a str,
    amount: &'a str,
    tx_type: &'a str,
    db_conn: &impl ConnCache,
) -> Result<NewTx<'a>> {
    let date = date.parse::<NaiveDate>()?;

    let local_now = Local::now().naive_local();

    let new_date = if date == local_now.date() {
        local_now
    } else {
        date.and_time(NaiveTime::MIN)
    };

    let details = if details.is_empty() {
        None
    } else {
        Some(details)
    };

    let amount = Dollar::new(amount.parse()?).cent().value();

    let from_method = db_conn.cache().get_method_id(from_method)?;
    let to_method = if to_method.is_empty() {
        None
    } else {
        Some(db_conn.cache().get_method_id(to_method)?)
    };

    let new_tx = NewTx::new(new_date, details, from_method, to_method, amount, tx_type);
    Ok(new_tx)
}

#[allow(clippy::too_many_arguments)]
pub fn parse_recurring_tx_fields<'a>(
    date: &'a str,
    details: &'a str,
    from_method: &'a str,
    to_method: &'a str,
    amount: &'a str,
    tx_type: &'a str,
    frequency: &'a str,
    recur_interval: &'a str,
    recur_value: &'a str,
    recur_month: &'a str,
    end_date: &'a str,
    db_conn: &impl ConnCache,
) -> Result<NewRecurringTx<'a>> {
    let start_date = date.parse::<NaiveDate>()?;

    let details = if details.is_empty() {
        None
    } else {
        Some(details)
    };

    let amount = Dollar::new(amount.parse()?).cent().value();

    let from_method = db_conn.cache().get_method_id(from_method)?;
    let to_method = if to_method.is_empty() {
        None
    } else {
        Some(db_conn.cache().get_method_id(to_method)?)
    };

    let frequency_value: RecurrenceFrequency = frequency.into();
    let interval = recur_interval.parse::<i32>()?;

    let (value, month) = match frequency_value {
        RecurrenceFrequency::Daily => (None, None),
        RecurrenceFrequency::Weekly => {
            let weekday = weekday_name_to_num(recur_value)?;

            (Some(weekday), None)
        }
        RecurrenceFrequency::Monthly => (Some(recur_value.parse::<i32>()?), None),
        RecurrenceFrequency::Yearly => (
            Some(recur_value.parse::<i32>()?),
            Some(month_name_to_num(recur_month)? as i32),
        ),
    };

    let next_recurring_date =
        first_occurrence_on_or_after(start_date, frequency_value, value, month);

    let end_date = if end_date.is_empty() {
        None
    } else {
        Some(end_date.parse::<NaiveDate>()?)
    };

    let created_at = Local::now().naive_local();

    Ok(NewRecurringTx::new(
        created_at,
        details,
        from_method,
        to_method,
        amount,
        tx_type,
        frequency,
        interval,
        value,
        month,
        next_recurring_date,
        end_date,
    ))
}

pub fn parse_search_fields<'a>(
    date: &'a str,
    details: &'a str,
    from_method: &'a str,
    to_method: &'a str,
    amount: &'a str,
    tx_type: &'a str,
    tags: &'a str,
    db_conn: &impl ConnCache,
) -> Result<NewSearch<'a>> {
    let date_nature = if date.is_empty() {
        None
    } else {
        let split_date = date.trim().split('-').collect::<Vec<&str>>();

        match split_date.len() {
            1 => {
                let year = split_date[0].parse::<i32>()?;

                let start_date = NaiveDate::from_ymd_opt(year, 1, 1)
                    .ok_or_else(|| anyhow!("{year} is an invalid year"))?
                    .and_time(NaiveTime::MIN);

                let end_date = NaiveDate::from_ymd_opt(year + 1, 1, 1)
                    .ok_or_else(|| anyhow!("{year} is an invalid year"))?
                    .and_time(LAST_POSSIBLE_TIME);

                Some(DateNature::ByYear {
                    start_date,
                    end_date,
                })
            }
            2 => {
                let year = split_date[0].parse::<i32>()?;
                let month = split_date[1].parse::<u32>()?;

                let start_date = NaiveDate::from_ymd_opt(year, month, 1)
                    .ok_or_else(|| anyhow!("{year} or {month} value is invalid"))?;

                let end_date = start_date + Months::new(1) - Days::new(1);

                let start_date = start_date.and_time(NaiveTime::MIN);
                let end_date = end_date.and_time(LAST_POSSIBLE_TIME);

                Some(DateNature::ByMonth {
                    start_date,
                    end_date,
                })
            }
            3 => {
                let date = date.parse::<NaiveDate>()?.and_time(NaiveTime::MIN);
                Some(DateNature::Exact(date))
            }
            _ => None,
        }
    };

    let details = if details.is_empty() {
        None
    } else {
        Some(details)
    };

    let from_method = if from_method.is_empty() {
        None
    } else {
        Some(db_conn.cache().get_method_id(from_method)?)
    };

    let to_method = if to_method.is_empty() {
        None
    } else {
        Some(db_conn.cache().get_method_id(to_method)?)
    };

    let amount = if amount.is_empty() {
        None
    } else {
        parse_amount_nature_cent(amount)?
    };

    let tx_type = if tx_type.is_empty() {
        None
    } else {
        Some(tx_type)
    };

    let tags = if tags.is_empty() {
        None
    } else {
        let tag_names = tags.split(',').map(str::trim).filter(|s| !s.is_empty());

        let mut tag_ids = Vec::new();

        for t in tag_names {
            tag_ids.push(db_conn.cache().get_tag_id(t)?);
        }

        Some(tag_ids)
    };

    let search_tx = NewSearch::new(
        date_nature,
        details,
        tx_type,
        from_method,
        to_method,
        amount,
        tags,
    );

    Ok(search_tx)
}
