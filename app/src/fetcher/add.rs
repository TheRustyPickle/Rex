use anyhow::{Context, Error, Result};
use chrono::NaiveDate;
use db::models::{Balance, FetchNature, NewTag, NewTx, NewTxMethod, Tx, TxMethod, TxTag, TxType};
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

        Balance::insert_batch_final_balance(final_balances, &mut mut_db_conn)?;

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

    let mut current_balance = Balance::get_balance_map(date, db_conn)?;

    let mut balance_to_update = Vec::new();

    let final_balance = Balance::get_final_balance(db_conn)?;

    let mut final_balance_updates = Vec::new();

    match tx_type.into() {
        TxType::Income => {
            let mut balance = current_balance.remove(&from_method).unwrap();
            let mut final_balance_entry = final_balance.get(&from_method).unwrap().clone();

            balance.balance += amount;
            final_balance_entry.balance += amount;

            final_balance_updates.push(final_balance_entry);

            balance_to_update.push(balance);
        }
        TxType::Expense => {
            let mut balance = current_balance.remove(&from_method).unwrap();
            let mut final_balance_entry = final_balance.get(&from_method).unwrap().clone();

            balance.balance -= amount;
            final_balance_entry.balance -= amount;

            balance_to_update.push(balance);
        }
        TxType::Transfer => {
            let mut balance_from = current_balance.remove(&from_method).unwrap();
            let mut balance_to = current_balance.remove(&to_method.unwrap()).unwrap();

            let mut from_final_balance_entry = final_balance.get(&from_method).unwrap().clone();
            let mut to_final_balance_entry =
                final_balance.get(&to_method.unwrap()).unwrap().clone();

            balance_from.balance -= amount;
            balance_to.balance += amount;

            from_final_balance_entry.balance -= amount;
            to_final_balance_entry.balance += amount;

            balance_to_update.push(balance_from);
            balance_to_update.push(balance_to);
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

        TxTag::insert_batch(tx_tags, &mut mut_db_conn)?;

        for balance in final_balance_updates {
            balance.update_final_balance(&mut mut_db_conn)?;
        }

        for balance in balance_to_update {
            balance.insert(&mut mut_db_conn)?;
        }

        tidy_balances(date, &mut mut_db_conn)?;

        Ok(())
    })?;

    db_conn.cache.new_tags(new_tags);

    Ok(())
}

fn tidy_balances(date: NaiveDate, db_conn: &mut impl ConnCache) -> Result<()> {
    let nature = FetchNature::Monthly;

    let txs = Tx::get_txs(date, nature, db_conn)?;

    let current_balance = Balance::get_balance(date, db_conn)?;

    let mut last_balance = Balance::get_last_balance(date, db_conn)?;

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
            balance.balance = last_balance;
            to_insert_balance.push(balance);
        }
    }

    for to_insert in to_insert_balance {
        to_insert.insert(db_conn)?;
    }

    Ok(())
}
