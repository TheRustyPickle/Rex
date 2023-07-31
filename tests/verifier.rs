extern crate rex_tui;
use rex_tui::db::create_db;
use rex_tui::outputs::{AType, NAType, VerifyingOutput};
use rex_tui::page_handler::DateType;
use rex_tui::utility::traits::DataVerifier;
use rex_tui::utility::*;
use rusqlite::Connection;
use std::fs;

struct Testing {
    data: Vec<String>,
    expected: Vec<String>,
    result: Vec<VerifyingOutput>,
}
impl DataVerifier for Testing {}

fn create_test_db(file_name: &str) -> Connection {
    if let Ok(metadata) = fs::metadata(file_name) {
        if metadata.is_file() {
            fs::remove_file(file_name).expect("Failed to delete existing file");
        }
    }

    let mut conn = Connection::open(file_name).unwrap();
    create_db(vec!["test1".to_string(), "test 2".to_string()], &mut conn).unwrap();
    conn
}

#[test]
fn check_sql_dates() {
    let data = get_sql_dates(11, 2, &DateType::Monthly);
    let expected_data = ("2024-12-01".to_string(), "2024-12-31".to_string());
    assert_eq!(data, expected_data);
}

#[test]
fn check_verifier_date() {
    let test_data = Testing {
        data: vec![
            "".to_string(),
            "2022-01-01".to_string(),
            "  2022  -  01  -  01  ".to_string(),
            "2022-01".to_string(),
            "2022-01-0".to_string(),
            "2022-0-01".to_string(),
            "01-01-2022".to_string(),
            "2022-01-32".to_string(),
            "2022-02-31".to_string(),
            "2022-13-31".to_string(),
            "2038-01-31".to_string(),
            "2022-01-".to_string(),
            "20222-01-01".to_string(),
            "2022-015-01".to_string(),
            "2022-01-311".to_string(),
        ],
        expected: vec![
            "".to_string(),
            "2022-01-01".to_string(),
            "2022-01-01".to_string(),
            "2022-01-01".to_string(),
            "2022-01-00".to_string(),
            "2022-00-01".to_string(),
            "2022-01-2022".to_string(),
            "2022-01-31".to_string(),
            "2022-02-31".to_string(),
            "2022-12-31".to_string(),
            "2037-01-31".to_string(),
            "2022-01-".to_string(),
            "2022-01-01".to_string(),
            "2022-12-01".to_string(),
            "2022-01-31".to_string(),
        ],
        result: vec![
            VerifyingOutput::Nothing(AType::Date),
            VerifyingOutput::Accepted(AType::Date),
            VerifyingOutput::Accepted(AType::Date),
            VerifyingOutput::NotAccepted(NAType::InvalidDate),
            VerifyingOutput::NotAccepted(NAType::InvalidDay),
            VerifyingOutput::NotAccepted(NAType::InvalidMonth),
            VerifyingOutput::NotAccepted(NAType::InvalidYear),
            VerifyingOutput::NotAccepted(NAType::DayTooBig),
            VerifyingOutput::NotAccepted(NAType::NonExistingDate),
            VerifyingOutput::NotAccepted(NAType::MonthTooBig),
            VerifyingOutput::NotAccepted(NAType::YearTooBig),
            VerifyingOutput::NotAccepted(NAType::ParsingError(AType::Date)),
            VerifyingOutput::NotAccepted(NAType::InvalidYear),
            VerifyingOutput::NotAccepted(NAType::InvalidMonth),
            VerifyingOutput::NotAccepted(NAType::InvalidDay),
        ],
    };

    for i in 0..test_data.data.len() {
        let mut to_verify = test_data.data[i].clone();
        let result = test_data.verify_date(&mut to_verify, &DateType::Exact);
        assert_eq!(result, test_data.result[i]);
        assert_eq!(to_verify, test_data.expected[i]);
    }
}

#[test]
fn check_verifier_amount() {
    let test_data = Testing {
        data: vec![
            "".to_string(),
            "1".to_string(),
            "1.".to_string(),
            "1.0".to_string(),
            "   -100     ".to_string(),
            "100+".to_string(),
            "-100 *  ".to_string(),
            "100-50".to_string(),
            "50 - 100".to_string(),
            "100   + 50".to_string(),
            "100*2".to_string(),
            "100/2".to_string(),
            "  2/7  ".to_string(),
            "   1000000000000000.52   ".to_string(),
            "@%15612".to_string(),
            " 5 + 2 * 3 - 5".to_string(),
            "1.0000".to_string(),
        ],
        expected: vec![
            "".to_string(),
            "1.00".to_string(),
            "1.00".to_string(),
            "1.00".to_string(),
            "100.00".to_string(),
            "100.00".to_string(),
            "100.00".to_string(),
            "50.00".to_string(),
            "50.00".to_string(),
            "150.00".to_string(),
            "200.00".to_string(),
            "50.00".to_string(),
            "0.29".to_string(),
            "1000000000.52".to_string(),
            "15612.00".to_string(),
            "6.00".to_string(),
            "1.00".to_string(),
        ],

        result: vec![
            VerifyingOutput::Nothing(AType::Amount),
            VerifyingOutput::Accepted(AType::Amount),
            VerifyingOutput::Accepted(AType::Amount),
            VerifyingOutput::Accepted(AType::Amount),
            VerifyingOutput::Accepted(AType::Amount),
            VerifyingOutput::Accepted(AType::Amount),
            VerifyingOutput::Accepted(AType::Amount),
            VerifyingOutput::Accepted(AType::Amount),
            VerifyingOutput::NotAccepted(NAType::AmountBelowZero),
            VerifyingOutput::Accepted(AType::Amount),
            VerifyingOutput::Accepted(AType::Amount),
            VerifyingOutput::Accepted(AType::Amount),
            VerifyingOutput::Accepted(AType::Amount),
            VerifyingOutput::Accepted(AType::Amount),
            VerifyingOutput::Accepted(AType::Amount),
            VerifyingOutput::Accepted(AType::Amount),
            VerifyingOutput::Accepted(AType::Amount),
        ],
    };
    for i in 0..test_data.data.len() {
        let mut to_verify = test_data.data[i].clone();
        let result = test_data.verify_amount(&mut to_verify);
        assert_eq!(result, test_data.result[i]);
        assert_eq!(to_verify, test_data.expected[i]);
    }
}

#[test]
fn check_verifier_tx_method() {
    let test_data = Testing {
        data: vec![
            "".to_string(),
            "test 2".to_string(),
            "random".to_string(),
            "  test 2  ".to_string(),
            "te".to_string(),
        ],
        expected: vec![
            "".to_string(),
            "test 2".to_string(),
            "test1".to_string(),
            "test 2".to_string(),
            "test1".to_string(),
        ],
        result: vec![
            VerifyingOutput::Nothing(AType::TxMethod),
            VerifyingOutput::Accepted(AType::TxMethod),
            VerifyingOutput::NotAccepted(NAType::InvalidTxMethod),
            VerifyingOutput::Accepted(AType::TxMethod),
            VerifyingOutput::NotAccepted(NAType::InvalidTxMethod),
        ],
    };
    let file_name = "check_verifier_tx_method.sqlite";
    let conn = create_test_db(file_name);

    for i in 0..test_data.data.len() {
        let mut to_verify = test_data.data[i].clone();
        let result = test_data.verify_tx_method(&mut to_verify, &conn);
        assert_eq!(result, test_data.result[i]);
        assert_eq!(to_verify, test_data.expected[i]);
    }
    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();
}

#[test]
fn check_verifier_tx_type() {
    let test_data = Testing {
        data: vec![
            "".to_string(),
            "e".to_string(),
            "i".to_string(),
            "w".to_string(),
            "   i".to_string(),
            "  i   ".to_string(),
        ],
        expected: vec![
            "".to_string(),
            "Expense".to_string(),
            "Income".to_string(),
            "".to_string(),
            "Income".to_string(),
            "Income".to_string(),
        ],
        result: vec![
            VerifyingOutput::Nothing(AType::TxType),
            VerifyingOutput::Accepted(AType::TxType),
            VerifyingOutput::Accepted(AType::TxType),
            VerifyingOutput::NotAccepted(NAType::InvalidTxType),
            VerifyingOutput::Accepted(AType::TxType),
            VerifyingOutput::Accepted(AType::TxType),
        ],
    };

    for i in 0..test_data.data.len() {
        let mut to_verify = test_data.data[i].clone();
        let result = test_data.verify_tx_type(&mut to_verify);
        assert_eq!(result, test_data.result[i]);
        assert_eq!(to_verify, test_data.expected[i]);
    }
}

#[test]
fn check_verifier_tags() {
    let test_data = Testing {
        data: vec![
            "".to_string(),
            "tag1,".to_string(),
            "tag1,    , , , ".to_string(),
            "tag1,tag2,tag3".to_string(),
            "tag1,tag1,tag1".to_string(),
            "tag1, Tag1, tAg1".to_string(),
        ],
        expected: vec![
            "".to_string(),
            "tag1".to_string(),
            "tag1".to_string(),
            "tag1, tag2, tag3".to_string(),
            "tag1".to_string(),
            "tag1, Tag1, tAg1".to_string(),
        ],
        result: Vec::new(),
    };

    for i in 0..test_data.data.len() {
        let mut to_verify = test_data.data[i].clone();
        test_data.verify_tags(&mut to_verify);
        assert_eq!(to_verify, test_data.expected[i]);
    }
}
