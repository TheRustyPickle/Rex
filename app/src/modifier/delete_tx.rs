use anyhow::Result;
use rex_db::ConnCache;
use rex_db::models::{Balance, FullTx, Tx, TxTag, TxType};

use crate::modifier::tidy_balances;

pub(crate) fn delete_tx(tx: &FullTx, db_conn: &mut impl ConnCache) -> Result<()> {
    let mut current_balance = Balance::get_balance_map(tx.date.date(), db_conn)?;

    let mut balance_to_update = Vec::new();

    let final_balance = Balance::get_final_balance(db_conn)?;

    let mut final_balance_updates = Vec::new();

    let from_method = tx.from_method.id;
    let to_method = tx.to_method.as_ref();
    let amount = tx.amount;

    // Reverse the transaction effect on balances.
    match &tx.tx_type {
        TxType::Income => {
            let mut balance = current_balance.remove(&from_method).unwrap();
            let mut final_balance_entry = final_balance.get(&from_method).unwrap().clone();

            balance.balance -= amount;
            final_balance_entry.balance -= amount;

            final_balance_updates.push(final_balance_entry);

            balance_to_update.push(balance);
        }
        TxType::Expense => {
            let mut balance = current_balance.remove(&from_method).unwrap();
            let mut final_balance_entry = final_balance.get(&from_method).unwrap().clone();

            balance.balance += amount;
            final_balance_entry.balance += amount;

            final_balance_updates.push(final_balance_entry);
            balance_to_update.push(balance);
        }
        TxType::Transfer => {
            let to_method_id = to_method.unwrap().id;

            let mut balance_from = current_balance.remove(&from_method).unwrap();
            let mut balance_to = current_balance.remove(&to_method_id).unwrap();

            let mut from_final_balance_entry = final_balance.get(&from_method).unwrap().clone();
            let mut to_final_balance_entry = final_balance.get(&to_method_id).unwrap().clone();

            balance_from.balance += amount;
            balance_to.balance -= amount;

            from_final_balance_entry.balance -= amount;
            to_final_balance_entry.balance += amount;

            balance_to_update.push(balance_from);
            balance_to_update.push(balance_to);

            final_balance_updates.push(from_final_balance_entry);
            final_balance_updates.push(to_final_balance_entry);
        }
    }

    Tx::delete_tx(tx.id, db_conn)?;
    TxTag::delete_by_tx_id(tx.id, db_conn)?;

    for balance in final_balance_updates {
        balance.update_final_balance(db_conn)?;
    }

    for balance in balance_to_update {
        balance.insert(db_conn)?;
    }

    tidy_balances(tx.date.date(), db_conn)?;

    Ok(())
}
