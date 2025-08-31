use anyhow::Result;
use chrono::NaiveDate;
use db::ConnCache;
use db::models::{Balance, FetchNature, NewTx, Tx, TxType};

pub(crate) fn tidy_balances(date: NaiveDate, db_conn: &mut impl ConnCache) -> Result<()> {
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

    let new_tx = NewTx::new(date, details, from_method, to_method, amount, tx_type, None);
    Ok(new_tx)
}
