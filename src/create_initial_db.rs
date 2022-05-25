use rusqlite::{Connection, Result};

fn _create_db() -> Result<()> {
    //TODO change test to actual db path
    let path = "test.sqlite";
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
        id_num INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
        source_1 TEXT DEFAULT 0,
        source_2 TEXT DEFAULT 0,
        source_3 TEXT DEFAULT 0,
        source_4 TEXT DEFAULT 0,
        CONSTRAINT changes_all_FK FOREIGN KEY (id_num) REFERENCES tx_all(id_num) ON DELETE CASCADE
    );", [])?;

    //TODO change the sources to real values
    conn.execute("CREATE TABLE balance_all (
        id_num INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
        source_1 TEXT DEFAULT 0,
        source_2 TEXT DEFAULT 0,
        source_3 TEXT DEFAULT 0,
        source_4 TEXT DEFAULT 0,
        CONSTRAINT balance_all_FK FOREIGN KEY (id_num) REFERENCES tx_all(id_num) ON DELETE CASCADE
    );", [])?;

    conn.execute("CREATE UNIQUE INDEX all_tx_id_num_IDX ON tx_all (id_num);", [])?;

    conn.execute("CREATE UNIQUE INDEX changes_all_id_num_IDX ON changes_all (id_num);", [])?;

    conn.execute("CREATE UNIQUE INDEX balance_all_id_num_IDX ON balance_all (id_num);", [])?;

    Ok(())
}