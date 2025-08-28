use rusqlite::{Connection, Result, Savepoint};

pub const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

pub const YEARS: [&str; 16] = [
    "2022", "2023", "2024", "2025", "2026", "2027", "2028", "2029", "2030", "2031", "2032", "2033",
    "2034", "2035", "2036", "2037",
];

pub const MODES: [&str; 3] = ["Monthly", "Yearly", "All Time"];

/// Creates the db that is used by this app
pub fn create_db(tx_methods: &[String], conn: &mut Connection) -> Result<()> {
    // Add a save point to reverse commits if failed
    let sp = conn.savepoint()?;

    // tx_all table. Will contain all tx data
    // Amount should have been REAL, old mistake, can't be bothered to migrate to REAL at this point
    // Same for date column
    // Also not too frequently searched by amount field anyway. Only when searching txs. Not a priority
    sp.execute(
        "CREATE TABLE tx_all (
        date TEXT,
        details TEXT,
        tx_method TEXT,
        amount TEXT,
        tx_type TEXT,
        id_num INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
        tags TEXT
    );",
        [],
    )?;

    create_balances_table(tx_methods, &sp)?;

    create_changes_table(tx_methods, &sp)?;

    create_activities_table(&sp)?;

    create_activity_txs_table(&sp)?;

    create_missing_indexes(&sp)?;

    sp.execute("CREATE UNIQUE INDEX all_tx_id_IDX ON tx_all (id_num);", [])?;

    sp.execute(
        "CREATE UNIQUE INDEX changes_all_id_IDX ON changes_all (id_num);",
        [],
    )?;
    sp.execute(
        "CREATE UNIQUE INDEX balance_all_id_num_IDX ON balance_all (id_num);",
        [],
    )?;

    // Fill up balance_all table with total year * 12 + 1 rows with 0 balance
    let zero_values = vec!["0.00"; tx_methods.len()];

    let highlighted_tx_methods = tx_methods
        .iter()
        .map(|method| format!("\"{method}\""))
        .collect::<Vec<String>>()
        .join(",");

    let query = format!(
        "INSERT INTO balance_all ({}) VALUES ({})",
        highlighted_tx_methods,
        zero_values.join(",")
    );

    for _i in YEARS {
        for _a in 0..MONTHS.len() {
            sp.execute(&query, [])?;
        }
    }

    sp.execute(&query, [])?;
    sp.commit()?;
    Ok(())
}

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

pub fn create_activities_table(sp: &Savepoint) -> Result<()> {
    sp.execute(
        "CREATE TABLE activities (
        date TEXT,
        activity_type TEXT,
        description TEXT,
        activity_num INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT
    );",
        [],
    )?;

    Ok(())
}

pub fn create_activity_txs_table(sp: &Savepoint) -> Result<()> {
    sp.execute(
        "CREATE TABLE activity_txs (
        date TEXT,
        details TEXT,
        tx_method TEXT,
        amount TEXT,
        tx_type TEXT,
        tags TEXT,
        id_num TEXT,
        activity_num INTEGER NOT NULL,
        insertion_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
        CONSTRAINT activity_tx_FK FOREIGN KEY (activity_num) REFERENCES activities(activity_num) ON DELETE CASCADE
    );",
        [],
    )?;
    Ok(())
}

pub fn create_missing_indexes(sp: &Savepoint) -> Result<()> {
    sp.execute("CREATE INDEX activities_date_idx ON activities(date);", [])?;

    sp.execute("CREATE INDEX tx_all_date_idx ON tx_all(date);", [])?;

    sp.execute(
        "CREATE INDEX activity_txs_date_idx ON activity_txs(date);",
        [],
    )?;

    sp.execute(
        "CREATE UNIQUE INDEX activities_activity_num_idx ON activities(activity_num);",
        [],
    )?;

    sp.execute(
        "CREATE INDEX activity_txs_activity_num_idx ON activity_txs(activity_num);",
        [],
    )?;

    sp.execute(
        "CREATE UNIQUE INDEX activity_txs_insertion_id_idx ON activity_txs(insertion_id);",
        [],
    )?;

    Ok(())
}
