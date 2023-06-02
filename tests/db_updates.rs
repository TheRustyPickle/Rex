extern crate rex_tui;
use rex_tui::db::{add_tags_column, update_balance_type};
use rex_tui::utility::{check_old_balance_sql, get_all_tx_columns, get_last_balance_id};
use rusqlite::Connection;
use std::fs;

fn check_test_db(file_name: &str) {
    if let Ok(metadata) = fs::metadata(file_name) {
        if metadata.is_file() {
            fs::remove_file(file_name).expect("Failed to delete existing file");
        }
    }
}

#[test]
fn check_tags_migration() {
    let file_name = "db_update_1.sqlite";
    check_test_db(file_name);
    let mut conn = Connection::open(file_name).unwrap();

    conn.execute(
        "CREATE TABLE tx_all (
        date TEXT,
        details TEXT,
        tx_method TEXT,
        amount TEXT,
        tx_type TEXT,
        id_num INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT
    );",
        [],
    )
    .unwrap();

    let old_columns = get_all_tx_columns(&conn);
    add_tags_column(&mut conn).unwrap();
    let new_columns = get_all_tx_columns(&conn);

    let expected_columns = vec![
        "date".to_string(),
        "details".to_string(),
        "tx_method".to_string(),
        "amount".to_string(),
        "tx_type".to_string(),
        "id_num".to_string(),
        "tags".to_string(),
    ];

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();

    assert_eq!(old_columns.contains(&"tags".to_string()), false);
    assert_eq!(new_columns, expected_columns);
}

#[test]
fn check_balance_migration() {
    let file_name = "db_update_2.sqlite";
    check_test_db(file_name);
    let mut conn = Connection::open(file_name).unwrap();

    conn.execute(
        r#"CREATE TABLE balance_all (
        id_num INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
        test1 TEXT DEFAULT 0.00,
        "test 2" TEXT DEFAULT 0.00
    );"#,
        [],
    )
    .unwrap();

    let query = format!(r#"INSERT INTO balance_all ("test1", "test 2") VALUES ("0.00", "0.00")"#);
    for _i in 0..49 {
        conn.execute(&query, []).unwrap();
    }

    let old_db_status = check_old_balance_sql(&mut conn);
    let old_last_balance_id = get_last_balance_id(&conn).unwrap();

    update_balance_type(&mut conn).unwrap();

    let db_status = check_old_balance_sql(&mut conn);
    let last_balance_id = get_last_balance_id(&conn).unwrap();

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();

    assert_eq!(old_db_status, true);
    assert_eq!(old_last_balance_id, 49);

    assert_eq!(db_status, false);
    assert_eq!(last_balance_id, 193);
}
