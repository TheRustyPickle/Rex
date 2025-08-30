use anyhow::{Error, Result};
use db::models::{Balance, NewTxMethod, TxMethod};
use db::{DbConn, MutDbConn};
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
