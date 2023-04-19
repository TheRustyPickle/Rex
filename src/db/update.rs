use crate::utility::get_all_tx_methods;
use rusqlite::{Connection, Result};

/// This function is used for adding new column to the database when adding new
/// Transaction Methods. Takes vector with transaction method names and commits them.
pub fn add_new_tx_methods(file_name: &str, tx_methods: Vec<String>) -> Result<()> {
    // add a save point to reverse commits if failed
    let mut conn = Connection::open(file_name)?;
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
    let query = "ALTER TABLE balance_all RENAME TO balance_all_old";
    sp.execute(&query, [])?;

    let mut query = "CREATE TABLE balance_all (
        id_num INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT"
        .to_string();
    for i in &all_methods {
        query.push_str(&format!(r#","{i}" REAL DEFAULT 0.00"#))
    }
    query.push_str(");");
    sp.execute(&query, [])?;

    let mut query = "INSERT INTO balance_all (id_num".to_string();

    for i in &all_methods {
        query.push_str(&format!(r#", "{i}""#))
    }
    query.push_str(") SELECT id_num");

    for i in &all_methods {
        query.push_str(&format!(r#", CAST("{i}" as REAL)"#))
    }
    query.push_str("FROM balance_all_old");

    sp.execute(&query, [])?;
    sp.execute("DROP TABLE balance_all_old", [])?;
    sp.execute(
        "CREATE UNIQUE INDEX balance_all_id_num_IDX ON balance_all (id_num);",
        [],
    )
    .unwrap();
    sp.commit()?;
    Ok(())
}
