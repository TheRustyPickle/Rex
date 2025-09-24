use rusqlite::{Result, Savepoint};

/// Creates the `balance_all` table of the DB
pub fn create_balances_table(tx_methods: &[String], sp: &Savepoint) -> Result<()> {
    // Balance_all table. Will contain tx methods as columns and their balances.
    // Each row represents 1 month.
    let tx_methods_str = tx_methods
        .iter()
        .map(|method| format!(r#""{method}" REAL DEFAULT 0.00"#))
        .collect::<Vec<String>>()
        .join(",");

    let query = format!(
        "CREATE TABLE balance_all (
            id_num INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
            {tx_methods_str}
        );"
    );

    sp.execute(&query, [])?;

    Ok(())
}

/// Create the `changes_all` table of the DB
pub fn create_changes_table(tx_methods: &[String], sp: &Savepoint) -> Result<()> {
    // Changes_all column. Will contain all balance changes with up and down arrows
    let columns = tx_methods
        .iter()
        .map(|column_name| format!(r#""{column_name}" TEXT DEFAULT 0.00"#))
        .collect::<Vec<String>>()
        .join(",");

    let query = format!(
        "CREATE TABLE changes_all (
            date TEXT,
            id_num INTEGER NOT NULL PRIMARY KEY,
            {columns},
            CONSTRAINT changes_all_FK FOREIGN KEY (id_num) REFERENCES tx_all(id_num) ON DELETE CASCADE
        );"
    );

    sp.execute(&query, [])?;

    Ok(())
}
