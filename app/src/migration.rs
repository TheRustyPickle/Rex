use anyhow::{Error, Result};
use chrono::{Datelike, Months, NaiveDate, NaiveTime};
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{Integer, Text};
use rex_db::ConnCache;
use rex_db::models::{
    Activity, ActivityNature, ActivityTx, ActivityTxTag, Balance, FetchNature, NewTag, NewTx, Tag,
    TxMethod, TxTag, TxType,
};
use std::collections::{HashMap, HashSet};
use std::io::Write;

use crate::conn::{DbConn, MutDbConn};
use crate::modifier::add_new_tx_methods;
use crate::utils::parse_amount_nature_cent;

#[derive(QueryableByName)]
struct ColumnInfo {
    #[diesel(sql_type = Text)]
    name: String,
}

#[derive(QueryableByName)]
struct OldTx {
    #[diesel(sql_type = Text)]
    date: String,
    #[diesel(sql_type = Text)]
    details: String,
    #[diesel(sql_type = Text)]
    tx_method: String,
    #[diesel(sql_type = Text)]
    amount: String,
    #[diesel(sql_type = Text)]
    tx_type: String,
    #[diesel(sql_type = Text)]
    tags: String,
}

#[derive(QueryableByName)]
struct OldActivity {
    #[diesel(sql_type = Text)]
    date: String,
    #[diesel(sql_type = Text)]
    activity_type: String,
    #[diesel(sql_type = Text)]
    #[allow(dead_code)]
    description: String,
    #[diesel(sql_type = Integer)]
    activity_num: i32,
}

#[derive(QueryableByName)]
struct OldActivityTx {
    #[diesel(sql_type = Text)]
    date: String,
    #[diesel(sql_type = Text)]
    details: String,
    #[diesel(sql_type = Text)]
    tx_method: String,
    #[diesel(sql_type = Text)]
    amount: String,
    #[diesel(sql_type = Text)]
    tx_type: String,
    #[diesel(sql_type = Text)]
    tags: String,
    #[diesel(sql_type = Text)]
    #[allow(dead_code)]
    id_num: String,
    #[diesel(sql_type = Integer)]
    activity_num: i32,
    #[diesel(sql_type = Integer)]
    insertion_id: i32,
}

pub fn start_migration(mut old_db_conn: DbConn, db_conn: &mut DbConn) -> Result<()> {
    let mut columns: Vec<ColumnInfo> = sql_query("PRAGMA table_info(balance_all);")
        .load(old_db_conn.conn())
        .expect("Failed to fetch column info");

    let rows: Vec<OldTx> = sql_query("SELECT * FROM tx_all ORDER BY date, id_num;")
        .load(old_db_conn.conn())
        .expect("Failed to fetch tx info");

    let activities: Vec<OldActivity> = sql_query("SELECT * FROM activities;")
        .load(old_db_conn.conn())
        .expect("Failed to fetch activity info");

    let activity_txs: Vec<OldActivityTx> = sql_query("SELECT * FROM activity_txs;")
        .load(old_db_conn.conn())
        .expect("Failed to fetch activity info");

    if !columns.is_empty() {
        columns.remove(0);
    }

    let tx_methods = columns.into_iter().map(|c| c.name).collect();

    let new_methods = add_new_tx_methods(&tx_methods, db_conn)?;
    db_conn.cache.new_tx_methods(new_methods);

    let DbConn { conn, cache } = db_conn;

    let mut start_date = None;

    let mut end_date = None;

    let mut count = 0;
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();

    conn.transaction::<_, Error, _>(|conn| {
        let mut mut_db_conn = MutDbConn::new(conn, cache);

        let mut final_balance = Balance::get_final_balance(&mut mut_db_conn)?;

        let mut skipped_activity_id = HashSet::new();

        for row in rows {
            let method = row.tx_method.to_string();

            let from_method;
            let mut to_method = String::new();

            if method.contains(" to ") {
                from_method = method.split(" to ").next().unwrap().to_string();
                to_method = method.split(" to ").last().unwrap().to_string();
            } else {
                from_method = method;
            }

            let date = row.date.parse::<NaiveDate>()?;

            if start_date.is_none() || date < start_date.unwrap() {
                start_date = Some(date);
            }

            if end_date.is_none() || date > end_date.unwrap() {
                end_date = Some(date);
            }

            migrate_tx(
                &row.date,
                &row.details,
                &from_method,
                &to_method,
                &row.amount,
                &row.tx_type,
                &row.tags,
                &mut final_balance,
                &mut mut_db_conn,
            )
            .unwrap();

            count += 1;
            write!(handle, "\rTransaction migrated: {count}").unwrap();
            handle.flush().unwrap();
        }

        for balance in final_balance.values() {
            balance.update_final_balance(&mut mut_db_conn).unwrap();
        }

        println!("\nAll transactions migrated successfully");

        count = 0;

        for activity in activities {
            let skipped = migrate_activity(
                &activity.date,
                &activity.activity_type,
                activity.activity_num,
                &mut mut_db_conn,
            )
            .unwrap();

            if skipped {
                skipped_activity_id.insert(activity.activity_num);
            }

            count += 1;
            write!(
                handle,
                "\rActivity migrated: {count} Activity skipped: {}",
                skipped_activity_id.len()
            )
            .unwrap();
        }

        println!("\nAll activities migrated successfully. Incompatible old activities are skipped");

        count = 0;
        let mut skipped_tx = 0;

        for tx in activity_txs {
            if skipped_activity_id.contains(&tx.activity_num) {
                skipped_tx += 1;
                continue;
            }
            migrate_activity_tx(tx, &mut mut_db_conn).unwrap();

            count += 1;
            write!(
                handle,
                "\rActivity transactions migrated: {count} Activity transactions skipped: {skipped_tx}"
            )
            .unwrap();
        }

        write!(
            handle,
            "\rActivity transactions migrated: {count} Activity transactions skipped: {skipped_tx}"
        )
        .unwrap();

        Ok(())
    })?;

    println!(
        "\nAll activity transactions migrated successfully. Incompatible old activity transactions are skipped"
    );

    db_conn.reload_tags();

    count = 0;

    if let Some(start_date) = start_date
        && let Some(end_date) = end_date
    {
        let total_months = (end_date.year() - start_date.year()) * 12
            + (end_date.month() as i32 - start_date.month() as i32)
            + 1;

        let mut ongoing_date = start_date - Months::new(1);

        while ongoing_date <= end_date {
            db_conn.fetch_txs_with_date(ongoing_date, FetchNature::Monthly)?;
            ongoing_date = ongoing_date + Months::new(1);

            count += 1;
            write!(
                handle,
                "\rTidying up balances. Total: {total_months} Completed: {count}"
            )
            .unwrap();
            handle.flush().unwrap();
        }
        println!("\nBalances migrated successfully");
    } else {
        println!(
            "\nLooks like no balance to tidy up. This should usually not happen. If it's a bug, report it on github"
        );
    }

    Ok(())
}

fn migrate_tx(
    date: &str,
    details: &str,
    from_method: &str,
    to_method: &str,
    amount: &str,
    tx_type: &str,
    tags: &str,
    final_balance: &mut HashMap<i32, Balance>,
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

    if tags.is_empty() {
        tag_list.push("Unknown".to_string());
    } else {
        let split_tags = tags.split(',').collect::<Vec<&str>>();

        for tag in split_tags {
            let trimmed_tag = tag.trim();
            if !trimmed_tag.is_empty() {
                tag_list.push(trimmed_tag.to_string());
            }
        }
    }

    let new_tx = NewTx::new(
        date.and_time(NaiveTime::MIN),
        details,
        from_method,
        to_method,
        amount,
        tx_type,
    );

    let mut current_balance = Balance::get_balance_map(date, db_conn)?;

    let mut balance_to_update = Vec::new();

    match tx_type.into() {
        TxType::Income => {
            let mut target_balance = current_balance.remove(&from_method).unwrap();

            target_balance.balance += amount;

            final_balance.get_mut(&from_method).unwrap().balance += amount;
            balance_to_update.push(target_balance);
        }
        TxType::Expense => {
            let mut target_balance = current_balance.remove(&from_method).unwrap();
            target_balance.balance -= amount;
            final_balance.get_mut(&from_method).unwrap().balance -= amount;
            balance_to_update.push(target_balance);
        }
        TxType::Transfer => {
            let mut target_balance_from = current_balance.remove(&from_method).unwrap();
            let mut target_balance_to = current_balance.remove(&to_method.unwrap()).unwrap();

            target_balance_from.balance -= amount;
            target_balance_to.balance += amount;

            balance_to_update.push(target_balance_from);
            balance_to_update.push(target_balance_to);

            final_balance.get_mut(&from_method).unwrap().balance -= amount;
            final_balance.get_mut(&to_method.unwrap()).unwrap().balance += amount;
        }
        TxType::Borrow | TxType::Lend | TxType::BorrowRepay | TxType::LendRepay => {
            panic!("This type of transaction should not exist in the database");
        }
    }

    let added_tx = new_tx.insert(db_conn)?;

    let mut tx_tags = Vec::new();

    for (index, tag) in tag_list.into_iter().enumerate() {
        let tag_data = NewTag::new(&tag).insert(db_conn)?;

        let tx_tag = TxTag::new(added_tx.id, tag_data.id, index == 0);

        tx_tags.push(tx_tag);
    }

    TxTag::insert_batch(tx_tags, db_conn)?;

    for balance in balance_to_update {
        balance.insert(db_conn)?;
    }

    Ok(())
}

fn migrate_activity(
    date: &str,
    activity: &str,
    activity_num: i32,
    conn: &mut impl ConnCache,
) -> Result<bool> {
    let activity_type = match activity {
        "Add TX" => ActivityNature::AddTx,
        "Edit TX" => ActivityNature::EditTx,
        "Delete TX" => ActivityNature::DeleteTx,
        "Search TX" => ActivityNature::SearchTx,
        "TX Position Swap" => ActivityNature::PositionSwap,
        _ => panic!("Invalid activity type {activity} found"),
    };

    if let ActivityNature::PositionSwap = activity_type {
        return Ok(true);
    }

    let date = date.parse::<NaiveDate>()?;
    let date = date.and_time(NaiveTime::MIN);

    Activity::new(date, activity_type, activity_num).insert(conn)?;

    Ok(false)
}

fn migrate_activity_tx(tx: OldActivityTx, conn: &mut impl ConnCache) -> Result<()> {
    let date = if tx.date.is_empty() {
        None
    } else {
        Some(tx.date)
    };

    let details = if tx.details.is_empty() {
        None
    } else {
        Some(tx.details)
    };

    let tx_type = if tx.tx_type.is_empty() {
        None
    } else {
        Some(tx.tx_type)
    };

    let mut to_method = None;

    let from_method = if tx.tx_method.is_empty() {
        None
    } else {
        let split_method = tx.tx_method.split("to").collect::<Vec<&str>>();

        if split_method.len() > 1 {
            let to_method_name = split_method[1].trim();
            let from_method_name = split_method[0].trim();

            if to_method_name != "?" && !to_method_name.is_empty() {
                let method = TxMethod::get_by_name(conn, to_method_name)?;
                to_method = Some(method.id);
            }

            if from_method_name != "?" && !from_method_name.is_empty() {
                let method = TxMethod::get_by_name(conn, from_method_name)?;
                Some(method.id)
            } else {
                None
            }
        } else {
            let method = TxMethod::get_by_name(conn, split_method[0])?;
            Some(method.id)
        }
    };

    let mut amount = None;
    let mut amount_type = None;

    let amount_nature = parse_amount_nature_cent(tx.amount.as_str())?;

    if let Some(nature) = amount_nature {
        amount_type = Some(nature.to_type().into());
        amount = Some(nature.extract().value());
    }

    ActivityTx::new(
        date,
        details,
        from_method,
        to_method,
        amount,
        amount_type,
        tx_type,
        None,
        tx.activity_num,
        tx.insertion_id,
    )
    .insert(conn)?;

    if !tx.tags.is_empty() {
        let split_tags = tx.tags.split(',').collect::<Vec<&str>>();

        let mut new_activity_tags = Vec::new();

        for tag in split_tags {
            let tag = tag.trim();
            let db_tag = Tag::get_by_name(conn, tag)?;

            let tag_id = if let Some(t) = db_tag {
                t.id
            } else {
                let new_tag = NewTag::new(tag).insert(conn)?;
                new_tag.id
            };

            let activity_tx_tag = ActivityTxTag::new(tx.insertion_id, tag_id);

            new_activity_tags.push(activity_tx_tag);
        }

        ActivityTxTag::insert_batch(new_activity_tags, conn)?;
    }

    Ok(())
}
