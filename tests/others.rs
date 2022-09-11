extern crate rex;
use rex::db::*;
//use rusqlite::{Connection};
//use std::fs;
//use std::collections::HashMap;

//fn create_test_db(file_name: &str) -> Connection {
//    create_db(file_name, vec!["test1".to_string(), "test 2".to_string()]).unwrap();
//    return Connection::open(file_name).unwrap();
//}

struct Testing {
    data: String,
}
impl StatusChecker for Testing {}

#[test]
fn check_sql_dates() {
    let data = get_sql_dates(11, 2);
    let expected_data = ("2024-11-01".to_string(), "2024-11-31".to_string());
    assert_eq!(data, expected_data);
}

#[test]
fn check_verifier_date_1() {
    let test_struct = Testing {
        data: "".to_string(),
    };
    let mut to_verify = test_struct.data.clone();

    let result = test_struct.verify_date(&mut to_verify).unwrap();
    let expected_data = "Date: Nothing to check".to_string();
    assert_eq!(result, expected_data);

    let test_struct = Testing {
        data: "01-01-2022".to_string(),
    };
    let mut to_verify = test_struct.data.clone();

    let result = test_struct.verify_date(&mut to_verify).unwrap();
    let expected_data = "Date: Year length not acceptable. Example Date: 2022-05-01".to_string();
    assert_eq!(result, expected_data);

    let test_struct = Testing {
        data: "2022-01-01".to_string(),
    };
    let mut to_verify = test_struct.data.clone();

    let result = test_struct.verify_date(&mut to_verify).unwrap();
    let expected_data = "Date: Date Accepted".to_string();
    assert_eq!(result, expected_data);

    let test_struct = Testing {
        data: "2022-01".to_string(),
    };
    let mut to_verify = test_struct.data.clone();

    let result = test_struct.verify_date(&mut to_verify).unwrap();
    let expected_data = "Date: Unknown date".to_string();
    assert_eq!(result, expected_data);
}

#[test]
fn check_verifier_date_2() {
    let test_struct = Testing {
        data: "2022-01-0".to_string(),
    };
    let mut to_verify = test_struct.data.clone();

    let result = test_struct.verify_date(&mut to_verify).unwrap();
    let expected_data = "Date: Day length not acceptable. Example Date: 2022-05-01".to_string();
    assert_eq!(result, expected_data);

    let test_struct = Testing {
        data: "2022-0-01".to_string(),
    };
    let mut to_verify = test_struct.data.clone();

    let result = test_struct.verify_date(&mut to_verify).unwrap();
    let expected_data = "Date: Month length not acceptable. Example Date: 2022-05-01".to_string();
    assert_eq!(result, expected_data);

    let test_struct = Testing {
        data: "202-01-01".to_string(),
    };
    let mut to_verify = test_struct.data.clone();

    let result = test_struct.verify_date(&mut to_verify).unwrap();
    let expected_data = "Date: Year length not acceptable. Example Date: 2022-05-01".to_string();
    assert_eq!(result, expected_data);
}

#[test]
fn check_verifier_date_3() {
    let test_struct = Testing {
        data: "2022-01-32".to_string(),
    };
    let mut to_verify = test_struct.data.clone();

    let result = test_struct.verify_date(&mut to_verify).unwrap();
    let expected_data = "Date: Day must be between 01-31".to_string();
    assert_eq!(result, expected_data);

    let test_struct = Testing {
        data: "2022-02-31".to_string(),
    };
    let mut to_verify = test_struct.data.clone();

    let result = test_struct.verify_date(&mut to_verify).unwrap();
    let expected_data =
        "Date: Date not acceptable and possibly non-existing. Error: input is out of range"
            .to_string();
    assert_eq!(result, expected_data);

    let test_struct = Testing {
        data: "2022-13-31".to_string(),
    };
    let mut to_verify = test_struct.data.clone();

    let result = test_struct.verify_date(&mut to_verify).unwrap();
    let expected_data = "Date: Month must be between 01-12".to_string();
    assert_eq!(result, expected_data);

    let test_struct = Testing {
        data: "2026-01-31".to_string(),
    };
    let mut to_verify = test_struct.data.clone();

    let result = test_struct.verify_date(&mut to_verify).unwrap();
    let expected_data = "Date: Year must be between 2022-2025".to_string();
    assert_eq!(result, expected_data);
}

#[test]
#[should_panic]
fn check_verifier_date_4() {
    let test_struct = Testing {
        data: "2022-01-".to_string(),
    };
    let mut to_verify = test_struct.data.clone();

    test_struct.verify_date(&mut to_verify).unwrap();
}
