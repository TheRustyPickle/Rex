use anyhow::{Context, Error, Result};
use chrono::NaiveDate;
use db::models::{Balance, NewTag, NewTx, NewTxMethod, TxMethod, TxTag, TxType};
use db::{ConnCache, DbConn, MutDbConn};
use diesel::prelude::*;

pub fn add_new_tx_methods(method_list: &Vec<String>, db_conn: &mut DbConn) -> Result<()> {
    let mut last_position = TxMethod::get_last_position(db_conn)?;

    let mut methods = Vec::new();

    for method in method_list {
        let new_method = NewTxMethod::new(method, last_position + 1);

        methods.push(new_method);

        last_position += 1;
    }

    let DbConn { conn, cache } = db_conn;

    let mut new_methods = Vec::new();

    conn.transaction::<_, Error, _>(|conn| {
        let mut mut_db_conn = MutDbConn::new(conn, cache);

        let mut final_balances = Vec::new();

        for method in methods {
            let new_method = method.insert(&mut mut_db_conn)?;

            new_methods.push(new_method.clone());

            let new_balance = Balance::new(new_method.id, 0, 0, 0, true);
            final_balances.push(new_balance);
        }

        Balance::insert_batch_final_balance(final_balances, conn)?;

        Ok(())
    })?;

    db_conn.cache.new_tx_methods(new_methods);

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn add_new_tx(
    date: &str,
    details: &str,
    from_method: &str,
    to_method: &str,
    amount: &str,
    tx_type: &str,
    tags: &str,
    db_conn: &mut DbConn,
) -> Result<()> {
    let date = date.parse::<NaiveDate>()?;

    let details = if details.is_empty() {
        None
    } else {
        Some(details)
    };

    let amount = (amount.parse::<f64>()? * 100.0).round() as i64;

    let from_method = db_conn.get_method_id(from_method).unwrap();
    let to_method = if to_method.is_empty() {
        None
    } else {
        Some(db_conn.get_method_id(to_method).unwrap())
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

    let DbConn { conn, cache } = db_conn;

    let mut new_tags = Vec::new();

    // TODO: Add activity txs later
    conn.transaction::<_, Error, _>(|conn| {
        let mut mut_db_conn = MutDbConn::new(conn, cache);

        let added_tx = new_tx
            .insert(&mut mut_db_conn)
            .context("Failed on new tx")?;

        let mut tx_tags = Vec::new();

        for tag in tag_list {
            if let Some(tag_id) = mut_db_conn.cache().get_tag_id(&tag) {
                let tx_tag = TxTag::new(added_tx.id, tag_id);
                tx_tags.push(tx_tag);
                continue;
            } else {
                let tag_data = NewTag::new(&tag)
                    .insert(&mut mut_db_conn)
                    .context("Failed on new tag")?;

                new_tags.push(tag_data.clone());

                let tx_tag = TxTag::new(added_tx.id, tag_data.id);

                tx_tags.push(tx_tag);
            }
        }

        TxTag::insert_batch(tx_tags, &mut mut_db_conn).context("Failed on tx tags")?;

        for balance in balance_to_update {
            balance
                .insert_conn(&mut mut_db_conn)
                .context("Failed on balance")?;
        }

        Ok(())
    })?;

    db_conn.cache.new_tags(new_tags);

    Ok(())
}
