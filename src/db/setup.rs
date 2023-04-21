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

/// If the local database is not found, this is executed to create the initial database
/// with the provided transaction methods.
pub fn create_db(tx_methods: Vec<String>, conn: &mut Connection) -> Result<()> {
    // add a save point to reverse commits if failed
    let sp = conn.savepoint()?;

    // tx_all table. Will contain all tx data
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

    // changes_all column. Will contain all balance changes with up and down arrows
    let columns = tx_methods
        .iter()
        .map(|column_name| format!(r#""{}" TEXT DEFAULT 0.00"#, column_name))
        .collect::<Vec<String>>()
        .join(",");

    let query = format!(
        "CREATE TABLE changes_all (
            date TEXT,
            id_num INTEGER NOT NULL PRIMARY KEY,
            {},
            CONSTRAINT changes_all_FK FOREIGN KEY (id_num) REFERENCES tx_all(id_num) ON DELETE CASCADE
        );",
        columns
    );

    create_balances_table(&tx_methods, &sp)?;

    sp.execute(&query, [])?;

    sp.execute("CREATE UNIQUE INDEX all_tx_id_IDX ON tx_all (id_num);", [])?;

    sp.execute(
        "CREATE UNIQUE INDEX changes_all_id_IDX ON changes_all (id_num);",
        [],
    )?;
    sp.execute(
        "CREATE UNIQUE INDEX balance_all_id_num_IDX ON balance_all (id_num);",
        [],
    )?;

    // fill up balance_all table with total year * 12 + 1 rows with 0 balance
    let zero_values = vec!["0.00"; tx_methods.len()];

    let highlighted_tx_methods = tx_methods
        .iter()
        .map(|method| format!("\"{}\"", method))
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

pub fn create_balances_table(tx_methods: &Vec<String>, sp: &Savepoint) -> Result<()> {
    // balance_all table. Will contain tx methods as columns and their balances.
    // each row represents 1 month.
    let tx_methods_str = tx_methods
        .iter()
        .map(|method| format!(r#""{}" REAL DEFAULT 0.00"#, method))
        .collect::<Vec<String>>()
        .join(",");

    let query = format!(
        "CREATE TABLE balance_all (
            id_num INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
            {}
        );",
        tx_methods_str
    );

    sp.execute(&query, [])?;

    Ok(())
}
