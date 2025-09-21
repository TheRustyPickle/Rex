use anyhow::{Context, Result};
use db::ConnCache;
use db::models::{Balance, NewTag, NewTx, Tag, Tx, TxTag, TxType};

use crate::modifier::tidy_balances;

pub(crate) fn add_new_tx(
    tx: NewTx,
    tags: &str,
    maintain_id: Option<i32>,
    db_conn: &mut impl ConnCache,
) -> Result<Vec<Tag>> {
    let date = tx.date;
    let from_method = tx.from_method;
    let to_method = tx.to_method;
    let amount = tx.amount;
    let tx_type = tx.tx_type;
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

            final_balance_updates.push(final_balance_entry);
            balance_to_update.push(balance);
        }
        TxType::Transfer => {
            let to_method_id = to_method.unwrap();

            let mut balance_from = current_balance.remove(&from_method).unwrap();
            let mut balance_to = current_balance.remove(&to_method_id).unwrap();

            let mut from_final_balance_entry = final_balance.get(&from_method).unwrap().clone();
            let mut to_final_balance_entry = final_balance.get(&to_method_id).unwrap().clone();

            balance_from.balance -= amount;
            balance_to.balance += amount;

            from_final_balance_entry.balance -= amount;
            to_final_balance_entry.balance += amount;

            balance_to_update.push(balance_from);
            balance_to_update.push(balance_to);

            final_balance_updates.push(from_final_balance_entry);
            final_balance_updates.push(to_final_balance_entry);
        }
    }

    let mut new_tags = Vec::new();

    // TODO: Add activity txs later

    let added_tx = if let Some(id) = maintain_id {
        Tx::from_new_tx(tx, id).insert(db_conn)?
    } else {
        tx.insert(db_conn).context("Failed on new tx")?
    };

    let mut tx_tags = Vec::new();

    for tag in tag_list {
        if let Ok(tag_id) = db_conn.cache().get_tag_id(&tag) {
            let tx_tag = TxTag::new(added_tx.id, tag_id);
            tx_tags.push(tx_tag);
            continue;
        } else {
            let tag_data = NewTag::new(&tag)
                .insert(db_conn)
                .context("Failed on new tag")?;

            new_tags.push(tag_data.clone());

            let tx_tag = TxTag::new(added_tx.id, tag_data.id);

            tx_tags.push(tx_tag);
        }
    }

    TxTag::insert_batch(tx_tags, db_conn)?;

    for balance in final_balance_updates {
        balance.update_final_balance(db_conn)?;
    }

    for balance in balance_to_update {
        balance.insert(db_conn)?;
    }

    tidy_balances(date, db_conn)?;

    Ok(new_tags)
}
