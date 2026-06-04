use anyhow::Result;
use rex_db::ConnCache;
use rex_db::models::{Balance, NewTxMethod, TxMethod};

pub(crate) fn add_new_tx_methods(
    method_list: &[String],
    db_conn: &mut impl ConnCache,
) -> Result<Vec<TxMethod>> {
    let mut methods = Vec::new();

    for (last_position, method) in (TxMethod::get_last_position(db_conn)?..).zip(method_list.iter())
    {
        let new_method = NewTxMethod::new(method, last_position + 1);

        methods.push(new_method);
    }

    let mut new_methods = Vec::new();

    let mut final_balances = Vec::new();

    for method in methods {
        let new_method = method.insert(db_conn)?;

        new_methods.push(new_method.clone());

        let new_balance = Balance::new(new_method.id, 0, 0, 0, true);
        final_balances.push(new_balance);
    }

    Balance::insert_batch_final_balance(final_balances, db_conn)?;

    Ok(new_methods)
}
