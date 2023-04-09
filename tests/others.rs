extern crate rex;
use rex::db::create_db;
use rex::outputs::{AType, NAType, VerifyingOutput};
use rex::utility::traits::DataVerifier;
use rex::utility::*;
use rusqlite::Connection;
use std::fs;

struct Testing {
    data: String,
    result: VerifyingOutput,
}
impl DataVerifier for Testing {}

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
    let test_data = Testing {
        data: "".to_string(),
        result: VerifyingOutput::Nothing(AType::Date),
    };
    let mut to_verify = test_data.data.clone();
    let result = test_data.verify_date(&mut to_verify);
    assert_eq!(result, test_data.result);

    let test_data = Testing {
        data: "2022-01-01".to_string(),
        result: VerifyingOutput::Accepted(AType::Date),
    };
    let mut to_verify = test_data.data.clone();
    let result = test_data.verify_date(&mut to_verify);
    assert_eq!(result, test_data.result);
    assert_eq!(to_verify, "2022-01-01".to_string());

    let test_data = Testing {
        data: "  2022  -  01  -  01  ".to_string(),
        result: VerifyingOutput::Accepted(AType::Date),
    };
    let mut to_verify = test_data.data.clone();
    let result = test_data.verify_date(&mut to_verify);
    assert_eq!(result, test_data.result);
    assert_eq!(to_verify, "2022-01-01".to_string());

    let test_data = Testing {
        data: "  2022-01-01  ".to_string(),
        result: VerifyingOutput::Accepted(AType::Date),
    };
    let mut to_verify = test_data.data.clone();
    let result = test_data.verify_date(&mut to_verify);
    assert_eq!(result, test_data.result);
    assert_eq!(to_verify, "2022-01-01".to_string());

    let test_data = Testing {
        data: "2022-01".to_string(),
        result: VerifyingOutput::NotAccepted(NAType::InvalidDate),
    };
    let mut to_verify = test_data.data.clone();
    let result = test_data.verify_date(&mut to_verify);
    assert_eq!(result, test_data.result);
    assert_eq!(to_verify, "2022-01-01".to_string());
}

#[test]
fn check_verifier_date_2() {
    let test_data = Testing {
        data: "2022-01-0".to_string(),
        result: VerifyingOutput::NotAccepted(NAType::InvalidDay),
    };
    let mut to_verify = test_data.data.clone();
    let result = test_data.verify_date(&mut to_verify);
    assert_eq!(result, test_data.result);
    assert_eq!(to_verify, "2022-01-00".to_string());

    let test_data = Testing {
        data: "2022-0-01".to_string(),
        result: VerifyingOutput::NotAccepted(NAType::InvalidMonth),
    };
    let mut to_verify = test_data.data.clone();
    let result = test_data.verify_date(&mut to_verify);
    assert_eq!(result, test_data.result);
    assert_eq!(to_verify, "2022-00-01".to_string());

    let test_data = Testing {
        data: "01-01-2022".to_string(),
        result: VerifyingOutput::NotAccepted(NAType::InvalidYear),
    };
    let mut to_verify = test_data.data.clone();
    let result = test_data.verify_date(&mut to_verify);
    assert_eq!(result, test_data.result);
    assert_eq!(to_verify, "2022-01-2022".to_string());
}

#[test]
fn check_verifier_date_3() {
    let test_data = Testing {
        data: "2022-01-32".to_string(),
        result: VerifyingOutput::NotAccepted(NAType::DayTooBig),
    };
    let mut to_verify = test_data.data.clone();
    let result = test_data.verify_date(&mut to_verify);
    assert_eq!(result, test_data.result);
    assert_eq!(to_verify, "2022-01-31".to_string());

    let test_data = Testing {
        data: "2022-02-31".to_string(),
        result: VerifyingOutput::NotAccepted(NAType::NonExistingDate),
    };
    let mut to_verify = test_data.data.clone();
    let result = test_data.verify_date(&mut to_verify);
    assert_eq!(result, test_data.result);
    assert_eq!(to_verify, "2022-02-31".to_string());

    let test_data = Testing {
        data: "2022-13-31".to_string(),
        result: VerifyingOutput::NotAccepted(NAType::MonthTooBig),
    };
    let mut to_verify = test_data.data.clone();
    let result = test_data.verify_date(&mut to_verify);
    assert_eq!(result, test_data.result);
    assert_eq!(to_verify, "2022-12-31".to_string());

    let test_data = Testing {
        data: "2026-01-31".to_string(),
        result: VerifyingOutput::NotAccepted(NAType::YearTooBig),
    };
    let mut to_verify = test_data.data.clone();
    let result = test_data.verify_date(&mut to_verify);
    assert_eq!(result, test_data.result);
    assert_eq!(to_verify, "2025-01-31".to_string());

    let test_data = Testing {
        data: "2022-01-".to_string(),
        result: VerifyingOutput::NotAccepted(NAType::ParsingError(AType::Date)),
    };
    let mut to_verify = test_data.data.clone();
    let result = test_data.verify_date(&mut to_verify);
    assert_eq!(result, test_data.result)
}

#[test]
fn check_verifier_amount_1() {
    let test_data = Testing {
        data: "".to_string(),
        result: VerifyingOutput::Nothing(AType::Amount),
    };
    let mut to_verify = test_data.data.clone();
    let result = test_data.verify_amount(&mut to_verify);
    assert_eq!(result, test_data.result);

    let test_data = Testing {
        data: "1".to_string(),
        result: VerifyingOutput::Accepted(AType::Amount),
    };
    let mut to_verify = test_data.data.clone();
    let result = test_data.verify_amount(&mut to_verify);
    assert_eq!(result, test_data.result);
    assert_eq!(to_verify, "1.00".to_string());

    let test_data = Testing {
        data: "1.".to_string(),
        result: VerifyingOutput::Accepted(AType::Amount),
    };
    let mut to_verify = test_data.data.clone();
    let result = test_data.verify_amount(&mut to_verify);
    assert_eq!(result, test_data.result);
    assert_eq!(to_verify, "1.00".to_string());

    let test_data = Testing {
        data: "1.0".to_string(),
        result: VerifyingOutput::Accepted(AType::Amount),
    };
    let mut to_verify = test_data.data.clone();
    let result = test_data.verify_amount(&mut to_verify);
    assert_eq!(result, test_data.result);
    assert_eq!(to_verify, "1.00".to_string());
}

#[test]
fn check_verifier_amount_2() {
    let test_data = Testing {
        data: "   -100     ".to_string(),
        result: VerifyingOutput::Accepted(AType::Amount),
    };
    let mut to_verify = test_data.data.clone();
    let result = test_data.verify_amount(&mut to_verify);
    assert_eq!(result, test_data.result);
    assert_eq!(to_verify, "100.00".to_string());

    let test_data = Testing {
        data: "100+".to_string(),
        result: VerifyingOutput::Accepted(AType::Amount),
    };
    let mut to_verify = test_data.data.clone();
    let result = test_data.verify_amount(&mut to_verify);
    assert_eq!(result, test_data.result);
    assert_eq!(to_verify, "100.00".to_string());

    let test_data = Testing {
        data: "-100 *  ".to_string(),
        result: VerifyingOutput::Accepted(AType::Amount),
    };
    let mut to_verify = test_data.data.clone();
    let result = test_data.verify_amount(&mut to_verify);
    assert_eq!(result, test_data.result);
    assert_eq!(to_verify, "100.00".to_string());
}

#[test]
fn check_verifier_amount_3() {
    let test_data = Testing {
        data: "100-50".to_string(),
        result: VerifyingOutput::Accepted(AType::Amount),
    };
    let mut to_verify = test_data.data.clone();
    let result = test_data.verify_amount(&mut to_verify);
    assert_eq!(result, test_data.result);
    assert_eq!(to_verify, "50.00".to_string());

    let test_data = Testing {
        data: "50 - 100".to_string(),
        result: VerifyingOutput::NotAccepted(NAType::AmountBelowZero),
    };
    let mut to_verify = test_data.data.clone();
    let result = test_data.verify_amount(&mut to_verify);
    assert_eq!(result, test_data.result);
    assert_eq!(to_verify, "50.00".to_string());

    let test_data = Testing {
        data: "100   + 50".to_string(),
        result: VerifyingOutput::Accepted(AType::Amount),
    };
    let mut to_verify = test_data.data.clone();
    let result = test_data.verify_amount(&mut to_verify);
    assert_eq!(result, test_data.result);
    assert_eq!(to_verify, "150.00".to_string());

    let test_data = Testing {
        data: "100*2".to_string(),
        result: VerifyingOutput::Accepted(AType::Amount),
    };
    let mut to_verify = test_data.data.clone();
    let result = test_data.verify_amount(&mut to_verify);
    assert_eq!(result, test_data.result);
    assert_eq!(to_verify, "200.00".to_string());

    let test_data = Testing {
        data: "100/2".to_string(),
        result: VerifyingOutput::Accepted(AType::Amount),
    };
    let mut to_verify = test_data.data.clone();
    let result = test_data.verify_amount(&mut to_verify);
    assert_eq!(result, test_data.result);
    assert_eq!(to_verify, "50.00".to_string());

    let test_data = Testing {
        data: "  2/7  ".to_string(),
        result: VerifyingOutput::Accepted(AType::Amount),
    };
    let mut to_verify = test_data.data.clone();
    let result = test_data.verify_amount(&mut to_verify);
    assert_eq!(result, test_data.result);
    assert_eq!(to_verify, "0.29".to_string());
}

#[test]
fn check_verifier_amount_4() {
    let test_data = Testing {
        data: "   1000000000000000.52   ".to_string(),
        result: VerifyingOutput::Accepted(AType::Amount),
    };
    let mut to_verify = test_data.data.clone();
    let result = test_data.verify_amount(&mut to_verify);
    assert_eq!(result, test_data.result);
    assert_eq!(to_verify, "1000000000.52".to_string());

    let test_data = Testing {
        data: "@%15612".to_string(),
        result: VerifyingOutput::NotAccepted(NAType::ParsingError(AType::Amount)),
    };
    let mut to_verify = test_data.data.clone();
    let result = test_data.verify_amount(&mut to_verify);
    assert_eq!(result, test_data.result)
}

#[test]
fn check_verifier_tx_method() {
    let test_data = Testing {
        data: "".to_string(),
        result: VerifyingOutput::Nothing(AType::TxMethod),
    };
    let file_name = "check_verifier_tx_method.sqlite";
    let conn = create_test_db(file_name);

    let mut to_verify = test_data.data.clone();
    let result = test_data.verify_tx_method(&mut to_verify, &conn);
    assert_eq!(result, test_data.result);
    let test_data = Testing {
        data: "test 2".to_string(),
        result: VerifyingOutput::Accepted(AType::TxMethod),
    };

    let mut to_verify = test_data.data.clone();
    let result = test_data.verify_tx_method(&mut to_verify, &conn);
    assert_eq!(result, test_data.result);

    let test_data = Testing {
        data: "random".to_string(),
        result: VerifyingOutput::NotAccepted(NAType::InvalidTxMethod),
    };
    let mut to_verify = test_data.data.clone();
    let result = test_data.verify_tx_method(&mut to_verify, &conn);
    assert_eq!(result, test_data.result);
    assert_eq!(to_verify, "test1".to_string());

    let test_data = Testing {
        data: "  test 2  ".to_string(),
        result: VerifyingOutput::Accepted(AType::TxMethod),
    };
    let mut to_verify = test_data.data.clone();
    let result = test_data.verify_tx_method(&mut to_verify, &conn);
    assert_eq!(result, test_data.result);
    assert_eq!(to_verify, "test 2".to_string());

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();
}

#[test]
fn check_verifier_tx_type() {
    let test_data = Testing {
        data: "".to_string(),
        result: VerifyingOutput::Nothing(AType::TxType),
    };
    let mut to_verify = test_data.data.clone();
    let result = test_data.verify_tx_type(&mut to_verify);
    assert_eq!(result, test_data.result);

    let test_data = Testing {
        data: "e".to_string(),
        result: VerifyingOutput::Accepted(AType::TxType),
    };
    let mut to_verify = test_data.data.clone();
    let result = test_data.verify_tx_type(&mut to_verify);
    assert_eq!(result, test_data.result);
    assert_eq!(to_verify, "Expense".to_string());

    let test_data = Testing {
        data: "i".to_string(),
        result: VerifyingOutput::Accepted(AType::TxType),
    };
    let mut to_verify = test_data.data.clone();
    let result = test_data.verify_tx_type(&mut to_verify);
    assert_eq!(result, test_data.result);
    assert_eq!(to_verify, "Income".to_string());

    let test_data = Testing {
        data: "w".to_string(),
        result: VerifyingOutput::NotAccepted(NAType::InvalidTxType),
    };
    let mut to_verify = test_data.data.clone();
    let result = test_data.verify_tx_type(&mut to_verify);
    assert_eq!(result, test_data.result);

    let test_data = Testing {
        data: "   i".to_string(),
        result: VerifyingOutput::Accepted(AType::TxType),
    };
    let mut to_verify = test_data.data.clone();
    let result = test_data.verify_tx_type(&mut to_verify);
    assert_eq!(result, test_data.result);
    assert_eq!(to_verify, "Income".to_string());

    let test_data = Testing {
        data: "  i   ".to_string(),
        result: VerifyingOutput::Accepted(AType::TxType),
    };
    let mut to_verify = test_data.data.clone();
    let result = test_data.verify_tx_type(&mut to_verify);
    assert_eq!(result, test_data.result);
    assert_eq!(to_verify, "Income".to_string());
}
