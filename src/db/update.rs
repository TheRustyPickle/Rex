use crate::db::{create_balances_table, create_changes_table};
use crate::utility::get_all_tx_methods;
use rusqlite::{Connection, Result, Savepoint};

/// adds new tx methods as columns on balance_all and changes_all tables. Gets called after
/// successful handling of 'J' from the app
pub fn add_new_tx_methods(tx_methods: Vec<String>, conn: &mut Connection) -> Result<()> {
    // add a save point to reverse commits if failed
    let sp = conn.savepoint()?;

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
    let sp = conn.savepoint()?;
    sp.execute("ALTER TABLE tx_all ADD tags TEXT DEFAULT Unknown;", [])?;
    sp.commit()?;
    Ok(())
}

/// Migrates existing database's balance_all column's data type from TEXT to REAL
pub fn update_balance_type(conn: &mut Connection) -> Result<()> {
    let all_methods = get_all_tx_methods(conn);

    let sp = conn.savepoint()?;
    let old_last_balance = get_last_balance(&sp, &all_methods);

    // rename table
    let query = "ALTER TABLE balance_all RENAME TO balance_all_old";
    sp.execute(query, [])?;

    // create the new updated balance_all table
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

    // insert everything from old table to the new balance_all
    let query = format!(
        "INSERT INTO balance_all (id_num, {}) SELECT id_num, {} FROM balance_all_old",
        columns, values
    );

    sp.execute(&query, [])?;
    sp.execute("DROP TABLE balance_all_old", [])?;
    sp.execute(
        "CREATE UNIQUE INDEX balance_all_id_num_IDX ON balance_all (id_num);",
        [],
    )?;

    // fill up balance_all table with total year * 12 + 1 rows with 0 balance for all columns
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

    // the old db had 49 rows. So add 144 more to match total year * 12 + 1 rows
    for _a in 0..144 {
        sp.execute(&query, [])?;
    }

    // swap the values from 49 row of the old table to the new table at 193
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

/// return the last balance from the db
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

/// Updates the DB with the new tx method name
pub fn rename_column(old_name: &str, new_name: &str, conn: &mut Connection) -> Result<()> {
    let sp = conn.savepoint()?;
    let query = format!(r#"ALTER TABLE balance_all RENAME COLUMN "{old_name}" TO "{new_name}""#);
    sp.execute(&query, [])?;

    let query = format!(r#"ALTER TABLE changes_all RENAME COLUMN "{old_name}" TO "{new_name}""#);
    sp.execute(&query, [])?;

    // Follows 3 cases
    // 1. old_name == new_name. Replace old name with the new name
    // 2. If the tx method = old_name to tx_method. Replace the old name part but keep to tx_method
    // 3. If the tx method = tx_method to old_name. Replace the old name part but keep tx_method to
    // last 2 are used for transfer tx
    let query = format!(
        r#"UPDATE tx_all SET tx_method =
            CASE
                WHEN tx_method = "{old_name}" THEN "{new_name}"
                WHEN tx_method LIKE "{old_name} %" THEN REPLACE(tx_method, "{old_name}", "{new_name}")
                WHEN tx_method LIKE "% {old_name}" THEN REPLACE(tx_method, " {old_name}", " {new_name}")
                ELSE tx_method
            END
        WHERE tx_method LIKE "%{old_name}%""#
    );
    sp.execute(&query, [])?;

    sp.commit()?;
    Ok(())
}

/// repositions tx method positions in the db
pub fn reposition_column(tx_methods: Vec<String>, conn: &mut Connection) -> Result<()> {
    let sp = conn.savepoint()?;

    let query = "ALTER TABLE balance_all RENAME TO balance_all_old";
    sp.execute(query, [])?;

    let query = "ALTER TABLE changes_all RENAME TO changes_all_old";
    sp.execute(query, [])?;

    create_balances_table(&tx_methods, &sp)?;
    create_changes_table(&tx_methods, &sp)?;

    let columns = tx_methods
        .iter()
        .map(|column_name| format!(r#""{}""#, column_name))
        .collect::<Vec<String>>()
        .join(",");

    let query = format!(
        "INSERT INTO balance_all (id_num, {}) SELECT id_num, {} FROM balance_all_old",
        columns, columns
    );
    sp.execute(&query, [])?;

    let query = format!(
        "INSERT INTO changes_all (date, id_num, {}) SELECT date, id_num, {} FROM changes_all_old",
        columns, columns
    );
    sp.execute(&query, [])?;

    sp.execute("DROP TABLE balance_all_old", [])?;
    sp.execute("DROP TABLE changes_all_old", [])?;

    sp.commit()?;

    Ok(())
}
