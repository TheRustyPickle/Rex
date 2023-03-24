use rusqlite::{Connection, Result};

/// This function is used for adding new column to the database when adding new
/// Transaction Methods. Takes vector with transaction method names and commits them.
pub fn add_new_tx_methods(file_name: &str, tx_methods: Vec<String>) -> Result<()> {
    // add a save point to reverse commits if failed
    let mut conn = Connection::open(file_name)?;
    let sp = conn.savepoint().unwrap();

    for i in &tx_methods {
        let query = format!(r#"ALTER TABLE balance_all ADD COLUMN "{i}" TEXT DEFAULT 0.00"#);
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
pub fn add_tags_column(file_name: &str) -> Result<()> {
    let mut conn = Connection::open(file_name)?;
    let sp = conn.savepoint().unwrap();
    sp.execute("ALTER TABLE tx_all ADD tags TEXT DEFAULT Unknown;", [])?;
    sp.commit()?;
    Ok(())
}
