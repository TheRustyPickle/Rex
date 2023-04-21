use crate::db::create_balances_table;
use crate::utility::get_all_tx_methods;
use rusqlite::{Connection, Result, Savepoint};

/// This function is used for adding new column to the database when adding new
/// Transaction Methods. Takes vector with transaction method names and commits them.
pub fn add_new_tx_methods(tx_methods: Vec<String>, conn: &mut Connection) -> Result<()> {
    // add a save point to reverse commits if failed
    let sp = conn.savepoint().unwrap();

    for i in &tx_methods {
        let query = format!(r#"ALTER TABLE balance_all ADD COLUMN "{i}" REAL DEFAULT 0.00"#);
        sp.execute(&query, [])?;
    }

    for i in &tx_methods {
        let query = format!(r#"ALTER TABLE changes_all ADD COLUMN "{i}" TEXT DEFAULT 0.00"#);
        sp.execute(&query, [])?;
    }
    sp.commit()?;
    Ok(())
}

/// Adds the tags column inside the database. Used when the old database without the tags column is detected
pub fn add_tags_column(conn: &mut Connection) -> Result<()> {
    let sp = conn.savepoint().unwrap();
    sp.execute("ALTER TABLE tx_all ADD tags TEXT DEFAULT Unknown;", [])?;
    sp.commit()?;
    Ok(())
}

pub fn update_balance_type(conn: &mut Connection) -> Result<()> {
    let all_methods = get_all_tx_methods(conn);

    let sp = conn.savepoint().unwrap();
    let old_last_balance = get_last_balance(&sp, &all_methods);

    let query = "ALTER TABLE balance_all RENAME TO balance_all_old";
    sp.execute(query, [])?;

    create_balances_table(&all_methods, &sp)?;

    let columns = all_methods
        .iter()
        .map(|column_name| format!(r#""{}""#, column_name))
        .collect::<Vec<String>>()
        .join(",");

    let values = all_methods
        .iter()
        .map(|method| format!(r#"CAST("{}" as REAL)"#, method))
        .collect::<Vec<_>>()
        .join(",");

    let query = format!(
        "INSERT INTO balance_all ({}, {}) SELECT id_num, {} FROM balance_all_old",
        "id_num", columns, values
    );

    sp.execute(&query, [])?;
    sp.execute("DROP TABLE balance_all_old", [])?;
    sp.execute(
        "CREATE UNIQUE INDEX balance_all_id_num_IDX ON balance_all (id_num);",
        [],
    )?;

    // fill up balance_all table with total year * 12 + 1 rows with 0 balance
    let zero_values = vec!["0.00"; all_methods.len()];

    let highlighted_tx_methods = all_methods
        .iter()
        .map(|method| format!("\"{}\"", method))
        .collect::<Vec<String>>()
        .join(",");

    let query = format!(
        "INSERT INTO balance_all ({}) VALUES ({})",
        highlighted_tx_methods,
        zero_values.join(",")
    );

    for _a in 0..144 {
        sp.execute(&query, [])?;
    }

    let new_values = old_last_balance
        .iter()
        .enumerate()
        .map(|(index, data)| format!(r#""{}" = {data}"#, all_methods[index]))
        .collect::<Vec<_>>()
        .join(",");

    let query = format!("UPDATE balance_all SET {} WHERE id_num = 193", new_values);
    sp.execute(&query, [])?;

    let new_values = all_methods
        .iter()
        .map(|data| format!(r#""{}" = 0.00"#, data))
        .collect::<Vec<_>>()
        .join(",");

    let query = format!("UPDATE balance_all SET {} WHERE id_num = 49", new_values);

    sp.execute(&query, [])?;
    sp.commit()?;
    Ok(())
}

fn get_last_balance(sp: &Savepoint, all_methods: &Vec<String>) -> Vec<String> {
    let mut query = format!(
        "SELECT {:?} FROM balance_all ORDER BY id_num DESC LIMIT 1",
        all_methods
    );
    query = query.replace('[', "");
    query = query.replace(']', "");

    let final_balance = sp.query_row(&query, [], |row| {
        let mut final_data: Vec<String> = Vec::new();
        for i in 0..all_methods.len() {
            let row_data: String = row.get(i).unwrap();
            final_data.push(row_data.to_string());
        }
        Ok(final_data)
    });
    final_balance.unwrap()
}
