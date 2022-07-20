extern crate rex;
use rex::db::{add_new_tx_methods, create_db};
use std::fs;

#[test]
fn check_db_creation() {
    create_db(
        "test_data_1.sqlite",
        vec!["test1".to_string(), "test 2".to_string()],
    )
    .unwrap();
    let paths = fs::read_dir(".").unwrap();
    let mut db_found = false;
    for path in paths {
        let path = path.unwrap().path().display().to_string();
        if path.contains("test_data_1.sqlite") {
            db_found = true;
        }
    }
    fs::remove_file("test_data_1.sqlite").unwrap();

    if db_found != true {
        panic!("db_creation failed!")
    }
}

#[test]
fn check_adding_new_tx_method() {
    create_db(
        "test_data_2.sqlite",
        vec!["test1".to_string(), "test 2".to_string()],
    )
    .unwrap();
    let status = add_new_tx_methods(
        "test_data_2.sqlite",
        vec!["test3".to_string(), "test 4".to_string()],
    );
    fs::remove_file("test_data_2.sqlite").unwrap();

    match status {
        Ok(_) => {}
        Err(e) => panic!("Failed adding new tx methods {e}"),
    }
}
