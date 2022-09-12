extern crate rex;
use rex::db::*;
use rusqlite::{Connection};
use std::fs;
//use std::collections::HashMap;

//fn create_test_db(file_name: &str) -> Connection {
//    create_db(file_name, vec!["test1".to_string(), "test 2".to_string()]).unwrap();
//    return Connection::open(file_name).unwrap();
//}

struct Testing {
    data: String,
}
impl StatusChecker for Testing {}

fn create_test_db(file_name: &str) -> Connection {
    create_db(file_name, vec!["test1".to_string(), "test 2".to_string()]).unwrap();
    Connection::open(file_name).unwrap()
}


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
        data: "2022-01-01".to_string(),
    };
    let mut to_verify = test_struct.data.clone();

    let result = test_struct.verify_date(&mut to_verify).unwrap();
    let expected_data = "Date: Date Accepted".to_string();
    assert_eq!(result, expected_data);
    assert_eq!(to_verify, "2022-01-01".to_string());

    let test_struct = Testing {
        data: "2022-01".to_string(),
    };
    let mut to_verify = test_struct.data.clone();

    let result = test_struct.verify_date(&mut to_verify).unwrap();
    let expected_data = "Date: Unknown date".to_string();
    assert_eq!(result, expected_data);
    assert_eq!(to_verify, "2022-01-01".to_string());
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
    assert_eq!(to_verify, "2022-01-00".to_string());

    let test_struct = Testing {
        data: "2022-0-01".to_string(),
    };
    let mut to_verify = test_struct.data.clone();

    let result = test_struct.verify_date(&mut to_verify).unwrap();
    let expected_data = "Date: Month length not acceptable. Example Date: 2022-05-01".to_string();
    assert_eq!(result, expected_data);
    assert_eq!(to_verify, "2022-00-01".to_string());

    let test_struct = Testing {
        data: "01-01-2022".to_string(),
    };
    let mut to_verify = test_struct.data.clone();

    let result = test_struct.verify_date(&mut to_verify).unwrap();
    let expected_data = "Date: Year length not acceptable. Example Date: 2022-05-01".to_string();
    assert_eq!(result, expected_data);
    assert_eq!(to_verify, "2022-01-2022".to_string());
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
    assert_eq!(to_verify, "2022-01-31".to_string());

    let test_struct = Testing {
        data: "2022-02-31".to_string(),
    };
    let mut to_verify = test_struct.data.clone();

    let result = test_struct.verify_date(&mut to_verify).unwrap();
    let expected_data =
        "Date: Date not acceptable and possibly non-existing. Error: input is out of range"
            .to_string();

    assert_eq!(result, expected_data);
    assert_eq!(to_verify, "2022-02-31".to_string());

    let test_struct = Testing {
        data: "2022-13-31".to_string(),
    };
    let mut to_verify = test_struct.data.clone();

    let result = test_struct.verify_date(&mut to_verify).unwrap();
    let expected_data = "Date: Month must be between 01-12".to_string();

    assert_eq!(result, expected_data);
    assert_eq!(to_verify, "2022-12-31".to_string());

    let test_struct = Testing {
        data: "2026-01-31".to_string(),
    };
    let mut to_verify = test_struct.data.clone();

    let result = test_struct.verify_date(&mut to_verify).unwrap();
    let expected_data = "Date: Year must be between 2022-2025".to_string();

    assert_eq!(result, expected_data);
    assert_eq!(to_verify, "2025-01-31".to_string());
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

#[test]
fn check_verifier_amount_1() {
    let test_struct = Testing {
        data: "".to_string(),
    };
    let mut to_verify = test_struct.data.clone();

    let result = test_struct.verify_amount(&mut to_verify).unwrap();
    let expected_data = "Amount: Nothing to check".to_string();
    assert_eq!(result, expected_data);

    let test_struct = Testing {
        data: "1".to_string(),
    };
    let mut to_verify = test_struct.data.clone();

    let result = test_struct.verify_amount(&mut to_verify).unwrap();
    let expected_data = "Amount: Amount Accepted".to_string();
    assert_eq!(result, expected_data);
    assert_eq!(to_verify, "1.00".to_string());

    let test_struct = Testing {
        data: "1.".to_string(),
    };
    let mut to_verify = test_struct.data.clone();

    let result = test_struct.verify_amount(&mut to_verify).unwrap();
    let expected_data = "Amount: Amount Accepted".to_string();
    assert_eq!(result, expected_data);
    assert_eq!(to_verify, "1.00".to_string());

    let test_struct = Testing {
        data: "1.0".to_string(),
    };
    let mut to_verify = test_struct.data.clone();

    let result = test_struct.verify_amount(&mut to_verify).unwrap();
    let expected_data = "Amount: Amount Accepted".to_string();
    assert_eq!(result, expected_data);
    assert_eq!(to_verify, "1.00".to_string());
}

#[test]
fn check_verifier_amount_2() {
    let test_struct = Testing {
        data: "   -100     ".to_string(),
    };
    let mut to_verify = test_struct.data.clone();

    let result = test_struct.verify_amount(&mut to_verify).unwrap();
    let expected_data = "Amount: Amount Accepted".to_string();
    assert_eq!(result, expected_data);
    assert_eq!(to_verify, "100.00".to_string());

    let test_struct = Testing {
        data: "100+".to_string(),
    };
    let mut to_verify = test_struct.data.clone();

    let result = test_struct.verify_amount(&mut to_verify).unwrap();
    let expected_data = "Amount: Amount Accepted".to_string();
    assert_eq!(result, expected_data);
    assert_eq!(to_verify, "100.00".to_string());

    let test_struct = Testing {
        data: "-100 *  ".to_string(),
    };
    let mut to_verify = test_struct.data.clone();

    let result = test_struct.verify_amount(&mut to_verify).unwrap();
    let expected_data = "Amount: Amount Accepted".to_string();
    assert_eq!(result, expected_data);
    assert_eq!(to_verify, "100.00".to_string());


}

#[test]
fn check_verifier_amount_3() {
    let test_struct = Testing {
        data: "100-50".to_string(),
    };
    let mut to_verify = test_struct.data.clone();

    let result = test_struct.verify_amount(&mut to_verify).unwrap();
    let expected_data = "Amount: Amount Accepted".to_string();
    assert_eq!(result, expected_data);
    assert_eq!(to_verify, "50.00".to_string());

    let test_struct = Testing {
        data: "50 - 100".to_string(),
    };
    let mut to_verify = test_struct.data.clone();

    let result = test_struct.verify_amount(&mut to_verify).unwrap();
    let expected_data = "Amount: Value must be bigger than zero".to_string();
    assert_eq!(result, expected_data);
    assert_eq!(to_verify, "50.00".to_string());

    let test_struct = Testing {
        data: "100   + 50".to_string(),
    };
    let mut to_verify = test_struct.data.clone();

    let result = test_struct.verify_amount(&mut to_verify).unwrap();
    let expected_data = "Amount: Amount Accepted".to_string();
    assert_eq!(result, expected_data);
    assert_eq!(to_verify, "150.00".to_string());

    let test_struct = Testing {
        data: "100*2".to_string(),
    };
    let mut to_verify = test_struct.data.clone();

    let result = test_struct.verify_amount(&mut to_verify).unwrap();
    let expected_data = "Amount: Amount Accepted".to_string();
    assert_eq!(result, expected_data);
    assert_eq!(to_verify, "200.00".to_string());

    let test_struct = Testing {
        data: "100/2".to_string(),
    };
    let mut to_verify = test_struct.data.clone();

    let result = test_struct.verify_amount(&mut to_verify).unwrap();
    let expected_data = "Amount: Amount Accepted".to_string();
    assert_eq!(result, expected_data);
    assert_eq!(to_verify, "50.00".to_string());

    let test_struct = Testing {
        data: "  2/7  ".to_string(),
    };
    let mut to_verify = test_struct.data.clone();

    let result = test_struct.verify_amount(&mut to_verify).unwrap();
    let expected_data = "Amount: Amount Accepted".to_string();
    assert_eq!(result, expected_data);
    assert_eq!(to_verify, "0.29".to_string());
}

#[test]
fn check_verifier_amount_4() {
    let test_struct = Testing {
        data: "   1000000000000000.52   ".to_string(),
    };
    let mut to_verify = test_struct.data.clone();

    let result = test_struct.verify_amount(&mut to_verify).unwrap();
    let expected_data = "Amount: Amount Accepted".to_string();
    assert_eq!(result, expected_data);
    assert_eq!(to_verify, "1000000000.52".to_string());
}

#[test]
#[should_panic]
fn check_verifier_amount_5() {
    let test_struct = Testing {
        data: "@%15612".to_string(),
    };
    let mut to_verify = test_struct.data.clone();

    test_struct.verify_amount(&mut to_verify).unwrap();
}

#[test]
fn check_verifier_tx_method() {

    let test_struct = Testing {
        data: "".to_string(),
    };
    let file_name = "check_verifier_tx_method.sqlite";
    let conn = create_test_db(file_name);

    let mut to_verify = test_struct.data.clone();

    let result = test_struct.verify_tx_method(&mut to_verify, &conn).unwrap();
    let expected_data = "TX Method: Nothing to check".to_string();
    assert_eq!(result, expected_data);

    let test_struct = Testing {
        data: "test 2".to_string(),
    };

    let mut to_verify = test_struct.data.clone();

    let result = test_struct.verify_tx_method(&mut to_verify, &conn).unwrap();
    let expected_data = "TX Method: Transaction Method Accepted".to_string();
    assert_eq!(result, expected_data);

    let test_struct = Testing {
        data: "random".to_string(),
    };

    let mut to_verify = test_struct.data.clone();

    let result = test_struct.verify_tx_method(&mut to_verify, &conn).unwrap();
    let expected_data = "TX Method: Transaction Method not found".to_string();
    assert_eq!(result, expected_data);
    assert_eq!(to_verify, "test1".to_string());

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();
}

#[test]
fn check_verifier_tx_type() {
    let test_struct = Testing {
        data: "".to_string(),
    };
    let mut to_verify = test_struct.data.clone();

    let result = test_struct.verify_tx_type(&mut to_verify).unwrap();
    let expected_data = "TX Type: Nothing to check".to_string();
    assert_eq!(result, expected_data);

    let test_struct = Testing {
        data: "e".to_string(),
    };
    let mut to_verify = test_struct.data.clone();

    let result = test_struct.verify_tx_type(&mut to_verify).unwrap();
    let expected_data = "TX Type: Transaction Type Accepted".to_string();
    assert_eq!(result, expected_data);
    assert_eq!(to_verify, "Expense".to_string());

    let test_struct = Testing {
        data: "i".to_string(),
    };
    let mut to_verify = test_struct.data.clone();

    let result = test_struct.verify_tx_type(&mut to_verify).unwrap();
    let expected_data = "TX Type: Transaction Type Accepted".to_string();
    assert_eq!(result, expected_data);
    assert_eq!(to_verify, "Income".to_string());

    let test_struct = Testing {
        data: "w".to_string(),
    };
    let mut to_verify = test_struct.data.clone();

    let result = test_struct.verify_tx_type(&mut to_verify).unwrap();
    let expected_data = "TX Type: Transaction Type not acceptable. Values: Expense/Income/E/I".to_string();
    assert_eq!(result, expected_data);

    let test_struct = Testing {
        data: "   i".to_string(),
    };
    let mut to_verify = test_struct.data.clone();

    let result = test_struct.verify_tx_type(&mut to_verify).unwrap();
    let expected_data = "TX Type: Transaction Type Accepted".to_string();
    assert_eq!(result, expected_data);
    assert_eq!(to_verify, "Income".to_string());

    let test_struct = Testing {
        data: "  i   ".to_string(),
    };
    let mut to_verify = test_struct.data.clone();

    let result = test_struct.verify_tx_type(&mut to_verify).unwrap();
    let expected_data = "TX Type: Transaction Type Accepted".to_string();
    assert_eq!(result, expected_data);
    assert_eq!(to_verify, "Income".to_string());
}