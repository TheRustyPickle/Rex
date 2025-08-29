use anyhow::{Context, Error, Result};
use chrono::{Datelike, Months, NaiveDate};
use db::models::{Balance, NewTag, NewTx, TxTag, TxType};
use db::{ConnCache, DbConn, MutDbConn};
use diesel::sql_types::Text;
use diesel::{prelude::*, sql_query};
use std::io::Write;

use crate::fetcher::{add_new_tx_methods, get_txs_date};

#[derive(QueryableByName)]
struct ColumnInfo {
    #[diesel(sql_type = Text)]
    name: String,
}

pub fn start_migration(
    rows: Vec<Vec<String>>,
    mut old_db_conn: DbConn,
    db_conn: &mut DbConn,
) -> Result<()> {
    let mut columns: Vec<ColumnInfo> = sql_query("PRAGMA table_info(balance_all);")
        .load(old_db_conn.conn())
        .expect("Failed to fetch column info");

    if !columns.is_empty() {
        columns.remove(0);
    }

    let tx_methods = columns.into_iter().map(|c| c.name).collect();

    add_new_tx_methods(&tx_methods, db_conn)?;

    let DbConn { conn, cache } = db_conn;

    let mut start_date = None;

    let mut end_date = None;

    conn.transaction::<_, Error, _>(|conn| {
        let mut mut_db_conn = MutDbConn::new(conn, cache);

        let mut count = 0;
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();

        for row in rows {
            let method = row[2].to_string();

            let from_method;
            let mut to_method = String::new();

            if method.contains(" to ") {
                from_method = method.split(" to ").next().unwrap().to_string();
                to_method = method.split(" to ").last().unwrap().to_string();
            } else {
                from_method = method;
            }

            let date = row[0].parse::<NaiveDate>()?;

            if start_date.is_none() || date < start_date.unwrap() {
                start_date = Some(date);
            }

            if end_date.is_none() || date > end_date.unwrap() {
                end_date = Some(date);
            }

            migrate_tx(
                &row[0],
                &row[1],
                &from_method,
                &to_method,
                &row[3],
                &row[4],
                &row[5],
                &mut mut_db_conn,
            )
            .unwrap();

            count += 1;
            write!(handle, "\rTransaction migrated: {count}").unwrap();
            handle.flush().unwrap();
        }

        Ok(())
    })?;

    db_conn.reload_tags();

    let mut count = 0;
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();

    if let Some(start_date) = start_date
        && let Some(end_date) = end_date
    {
        let total_months = (end_date.year() - start_date.year()) * 12
            + (end_date.month() as i32 - start_date.month() as i32)
            + 1;

        let mut ongoing_date = start_date;

        while ongoing_date <= end_date {
            get_txs_date(ongoing_date, db_conn)?;
            ongoing_date = ongoing_date + Months::new(1);

            count += 1;
            write!(
                handle,
                "\rTidying up balances. Total: {total_months} Completed: {count}"
            )
            .unwrap();
            handle.flush().unwrap();
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn migrate_tx(
    date: &str,
    details: &str,
    from_method: &str,
    to_method: &str,
    amount: &str,
    tx_type: &str,
    tags: &str,
    db_conn: &mut impl ConnCache,
) -> Result<()> {
    let date = date.parse::<NaiveDate>()?;

    let details = if details.is_empty() {
        None
    } else {
        Some(details)
    };

    let amount = (amount.parse::<f64>()? * 100.0).round() as i64;

    let from_method = db_conn.cache().get_method_id(from_method).unwrap();
    let to_method = if to_method.is_empty() {
        None
    } else {
        Some(db_conn.cache().get_method_id(to_method).unwrap())
    };

    let mut tag_list = Vec::new();

    if !tags.is_empty() {
        let split_tags = tags.split(',').collect::<Vec<&str>>();

        for tag in split_tags {
            let trimmed_tag = tag.trim();
            if !trimmed_tag.is_empty() {
                tag_list.push(trimmed_tag.to_string());
            }
        }
    } else {
        tag_list.push("Unknown".to_string());
    }

    let new_tx = NewTx::new(date, details, from_method, to_method, amount, tx_type, None);

    let current_balance = Balance::get_balance(date, db_conn)?;

    let mut balance_to_update = Vec::new();

    for mut balance in current_balance {
        match tx_type.into() {
            TxType::Income => {
                if from_method == balance.method_id {
                    balance.balance += amount;
                    balance_to_update.push(balance);
                }
            }
            TxType::Expense => {
                if from_method == balance.method_id {
                    balance.balance -= amount;
                    balance_to_update.push(balance);
                }
            }
            TxType::Transfer => {
                if from_method == balance.method_id {
                    balance.balance -= amount;
                    balance_to_update.push(balance);
                } else if to_method.unwrap() == balance.method_id {
                    balance.balance += amount;
                    balance_to_update.push(balance);
                }
            }
        }
    }

    // TODO: Add activity txs later

    let added_tx = new_tx.insert(db_conn).context("Failed on new tx")?;

    let mut tx_tags = Vec::new();

    for tag in tag_list {
        let tag_data = NewTag::new(&tag)
            .insert(db_conn)
            .context("Failed on new tag")?;

        let tx_tag = TxTag::new(added_tx.id, tag_data.id);

        tx_tags.push(tx_tag);
    }

    TxTag::insert_batch(tx_tags, db_conn).context("Failed on tx tags")?;

    for balance in balance_to_update {
        balance.insert_conn(db_conn).context("Failed on balance")?;
    }

    Ok(())
}
