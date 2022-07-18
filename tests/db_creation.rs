extern crate rex;
use rex::db::{create_db, add_new_tx_methods};
use std::fs;

#[test]
fn test_db_creation() {
    create_db("test_data_1.sqlite", vec!["test1".to_string(),
                                                            "test 2".to_string()]).unwrap();
    let paths = fs::read_dir(".").unwrap();
    let mut db_found = false;
    for path in paths {
        let path = path.unwrap().path().display().to_string();
        if path.contains("test_data_1.sqlite") {
            db_found = true;
        }
    }
    if db_found != true {panic!("db_creation failed!")}
    fs::remove_file("test_data_1.sqlite").unwrap();
}

#[test]
fn test_new_tx_method() {
    create_db("test_data_2.sqlite", vec!["test1".to_string(),
                                                "test 2".to_string()]).unwrap();
    let status = add_new_tx_methods("test_data_2.sqlite", vec!["test3".to_string(),
                                                                    "test 4".to_string()]);
    match status {
        Ok(_)  => {},
        Err(e) => panic!("Failed adding new tx methods {e}")
    }
    fs::remove_file("test_data_2.sqlite").unwrap();
}