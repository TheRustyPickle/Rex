use anyhow::{Result, anyhow};
use chrono::{Days, Local, Months, NaiveDate, NaiveTime};
use db::ConnCache;
use db::models::{Balance, DateNature, FetchNature, NewSearch, NewTx, Tx, TxType};
use shared::models::Dollar;

use crate::utils::parse_amount_nature_cent;

pub(crate) fn tidy_balances(date: NaiveDate, db_conn: &mut impl ConnCache) -> Result<()> {
    let nature = FetchNature::Monthly;

    let txs = Tx::get_txs(date, nature, db_conn)?;

    let current_balance = Balance::get_balance(date, nature, db_conn)?;

    let mut last_balance = Balance::get_last_balance(date, nature, db_conn)?;

    for tx in txs {
        match tx.tx_type.as_str().into() {
            TxType::Income => {
                let method_id = tx.from_method;
                *last_balance.get_mut(&method_id).unwrap() += tx.amount;
            }
            TxType::Expense => {
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
                    .ok_or_else(|| anyhow!("{} is an invalid year", year))?
                    .and_time(NaiveTime::MIN);

                let end_date = NaiveDate::from_ymd_opt(year + 1, 1, 1)
                    .ok_or_else(|| anyhow!("{} is an invalid year", year))?
                    .and_time(NaiveTime::MIN);

                Some(DateNature::ByYear {
                    start_date,
                    end_date,
                })
            }
            2 => {
                let year = split_date[0].parse::<i32>()?;
                let month = split_date[1].parse::<u32>()?;

                let start_date = NaiveDate::from_ymd_opt(year, month, 1)
                    .ok_or_else(|| anyhow!("{year} or {month} value is invalid"))?
                    .and_time(NaiveTime::MIN);

                let end_date = start_date + Months::new(1) - Days::new(1);

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
        let tags = tags.split(',').map(str::trim).collect::<Vec<&str>>();
        let tags = tags
            .iter()
            .map(|t| db_conn.cache().get_tag_id(t))
            .filter_map(Result::ok)
            .collect::<Vec<i32>>();

        Some(tags)
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
