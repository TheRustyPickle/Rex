use app::conn::{DbConn, get_conn_old};
use app::migration::start_migration;
use rusqlite::{Connection, Result};

use crate::db::{create_balances_table, create_changes_table};

/// Adds new tx methods as columns on `balance_all` and `changes_all` tables. Gets called after
/// successful handling of 'J' from the app
pub fn add_new_tx_methods(tx_methods: &[String], conn: &mut Connection) -> Result<()> {
    // Add a save point to reverse commits if failed
    let sp = conn.savepoint()?;

    for i in tx_methods {
        let query = format!(r#"ALTER TABLE balance_all ADD COLUMN "{i}" REAL DEFAULT 0.00"#);
        sp.execute(&query, [])?;
    }

    for i in tx_methods {
        let query = format!(r#"ALTER TABLE changes_all ADD COLUMN "{i}" TEXT DEFAULT 0.00"#);
        sp.execute(&query, [])?;
    }
    sp.commit()?;
    Ok(())
}

/// Updates the DB with the new tx method name
pub fn rename_column(old_name: &str, new_name: &str, conn: &mut Connection) -> Result<()> {
    let sp = conn.savepoint()?;
    let query = format!(r#"ALTER TABLE balance_all RENAME COLUMN "{old_name}" TO "{new_name}""#);
    sp.execute(&query, [])?;

    let query = format!(r#"ALTER TABLE changes_all RENAME COLUMN "{old_name}" TO "{new_name}""#);
    sp.execute(&query, [])?;

    // Follows 3 cases
    // 1. Old_name == new_name. Replace old name with the new name
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

/// Repositions tx method positions in the db
pub fn reposition_column(tx_methods: &[String], conn: &mut Connection) -> Result<()> {
    let sp = conn.savepoint()?;

    let query = "ALTER TABLE balance_all RENAME TO balance_all_old";
    sp.execute(query, [])?;

    let query = "ALTER TABLE changes_all RENAME TO changes_all_old";
    sp.execute(query, [])?;

    create_balances_table(tx_methods, &sp)?;
    create_changes_table(tx_methods, &sp)?;

    let columns = tx_methods
        .iter()
        .map(|column_name| format!(r#""{column_name}""#,))
        .collect::<Vec<String>>()
        .join(",");

    let query = format!(
        "INSERT INTO balance_all (id_num, {columns}) SELECT id_num, {columns} FROM balance_all_old"
    );
    sp.execute(&query, [])?;

    let query = format!(
        "INSERT INTO changes_all (date, id_num, {columns}) SELECT date, id_num, {columns} FROM changes_all_old"
    );
    sp.execute(&query, [])?;

    sp.execute("DROP TABLE balance_all_old", [])?;
    sp.execute("DROP TABLE changes_all_old", [])?;

    sp.commit()?;

    Ok(())
}

pub fn migrate_to_new_schema(conn: &mut Connection, new_conn: &mut DbConn) -> Result<()> {
    let mut statement = conn
        .prepare("SELECT * FROM tx_all ORDER BY date, id_num")
        .expect("could not prepare statement");

    let rows = statement
        .query_map([], |row| {
            let id_num: i32 = row.get(5).unwrap();

            Ok(vec![
                row.get(0).unwrap(),
                row.get(1).unwrap(),
                row.get(2).unwrap(),
                row.get(3).unwrap(),
                row.get(4).unwrap(),
                row.get(6).unwrap(),
                id_num.to_string(),
            ])
        })
        .unwrap();

    let mut row_list = Vec::new();
    for row in rows {
        let row = row.unwrap();
        row_list.push(row);
    }

    let db_path = conn.path().unwrap();

    let old_conn = get_conn_old(db_path);

    start_migration(row_list, old_conn, new_conn).unwrap();

    Ok(())
}
