use rusqlite::{Connection, Result};

pub fn create_db() -> Result<()> {
    let months = vec!["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];
    let years = vec!["2022", "2023", "2024", "2025"];
    //TODO change test to actual db path
    let path = "data.sqlite";
    let conn = Connection::open(path)?;

    conn.execute("CREATE TABLE tx_all (
        date TEXT,
        details TEXT,
        tx_method TEXT,
        amount TEXT,
        tx_type TEXT,
        id_num INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT
    );", [])?;

    conn.execute("CREATE TABLE changes_all (
        date TEXT,
        id_num INTEGER NOT NULL PRIMARY KEY,
        source_1 TEXT DEFAULT 0.00,
        source_2 TEXT DEFAULT 0.00,
        source_3 TEXT DEFAULT 0.00,
        source_4 TEXT DEFAULT 0.00,
        CONSTRAINT changes_all_FK FOREIGN KEY (id_num) REFERENCES tx_all(id_num) ON DELETE CASCADE
    );", [])?;

    //TODO change the sources to real values
    conn.execute("CREATE TABLE balance_all (
        id_num INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
        source_1 TEXT DEFAULT 0.00,
        source_2 TEXT DEFAULT 0.00,
        source_3 TEXT DEFAULT 0.00,
        source_4 TEXT DEFAULT 0.00
    );", [])?;

    conn.execute("CREATE UNIQUE INDEX all_tx_date_IDX ON tx_all (id_num);", []).unwrap();

    conn.execute("CREATE UNIQUE INDEX changes_all_date_IDX ON changes_all (id_num);", []).unwrap();

    conn.execute("CREATE UNIQUE INDEX balance_all_id_num_IDX ON balance_all (id_num);", []).unwrap();

    for _i in years {
        for _a in 0..months.len() {
            conn.execute("INSERT INTO balance_all (source_1, source_2, source_3, source_4) VALUES (?, ?, ?, ?)",
        [0.0, 0.0, 0.0, 0.0])?;
        }
    }
    conn.execute("INSERT INTO balance_all (source_1, source_2, source_3, source_4) VALUES (?, ?, ?, ?)",
        [0.0, 0.0, 0.0, 0.0])?;

    Ok(())
}