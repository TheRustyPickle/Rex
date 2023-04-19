use rusqlite::{Connection, Result};

/// If the local database is not found, this is executed to create the initial database
/// with the provided transaction methods.
pub fn create_db(file_name: &str, tx_methods: Vec<String>) -> Result<()> {
    let months = vec![
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
    // TODO: turn years into a CONST
    let years = vec!["2022", "2023", "2024", "2025"];

    // add a save point to reverse commits if failed
    let mut conn = Connection::open(file_name)?;
    let sp = conn.savepoint().unwrap();

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

    let mut query = "CREATE TABLE changes_all (
        date TEXT,
        id_num INTEGER NOT NULL PRIMARY KEY,"
        .to_string();
    // we don't know how many tx methods there are, so we have to loop through them
    for i in &tx_methods {
        query.push_str(&format!(r#""{i}" TEXT DEFAULT 0.00,"#))
    }
    query.push_str(
        "CONSTRAINT changes_all_FK FOREIGN KEY (id_num) REFERENCES tx_all(id_num) ON DELETE CASCADE
);",
    );

    sp.execute(&query, [])?;

    let mut query = "CREATE TABLE balance_all (
        id_num INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT"
        .to_string();
    for i in &tx_methods {
        query.push_str(&format!(r#","{i}" REAL DEFAULT 0.00"#))
    }
    query.push_str(");");

    sp.execute(&query, [])?;

    sp.execute("CREATE UNIQUE INDEX all_tx_id_IDX ON tx_all (id_num);", [])
        .unwrap();

    sp.execute(
        "CREATE UNIQUE INDEX changes_all_id_IDX ON changes_all (id_num);",
        [],
    )
    .unwrap();

    sp.execute(
        "CREATE UNIQUE INDEX balance_all_id_num_IDX ON balance_all (id_num);",
        [],
    )
    .unwrap();

    let mut q_marks = vec![];
    for _i in &tx_methods {
        q_marks.push("0.00")
    }

    let mut query = format!(
        "INSERT INTO balance_all ({:?}) VALUES ({:?})",
        tx_methods, q_marks
    );
    // We are using :? to keep the commas inside the string and remove the other unnecessary characters
    query = query.replace(['[', ']'], "");

    for _i in years {
        for _a in 0..months.len() {
            sp.execute(&query, [])?;
        }
    }
    sp.execute(&query, [])?;
    sp.commit()?;
    Ok(())
}
