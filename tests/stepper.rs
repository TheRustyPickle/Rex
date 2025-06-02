extern crate rex_tui;
use rex_tui::db::create_db;
use rex_tui::outputs::{StepType, SteppingError};
use rex_tui::page_handler::DateType;
use rex_tui::tx_handler::*;
use rex_tui::utility::traits::{DataVerifier, FieldStepper};
use rusqlite::Connection;
use std::fs;

struct Testing {
    data: Vec<String>,
    expected: Vec<String>,
    result: Vec<Result<(), SteppingError>>,
}
impl FieldStepper for Testing {}
impl DataVerifier for Testing {}

fn create_test_db(file_name: &str) -> Connection {
    if let Ok(metadata) = fs::metadata(file_name) {
        if metadata.is_file() {
            fs::remove_file(file_name).expect("Failed to delete existing file");
        }
    }

    let mut conn = Connection::open(file_name).unwrap();
    create_db(
        &[
            "Super Special Bank".to_string(),
            "Cash Cow".to_string(),
            "Danger Cash".to_string(),
        ],
        &mut conn,
    )
    .unwrap();
    conn
}

fn add_dummy_tx(conn: &mut Connection) {
    add_tx(
        "2022-08-19",
        "Car expense",
        "Super Special Bank",
        "100.00",
        "Expense",
        "Car",
        None,
        conn,
    )
    .unwrap();

    add_tx(
        "2023-07-19",
        "Food cost",
        "Cash Cow",
        "100.00",
        "Expense",
        "Food",
        None,
        conn,
    )
    .unwrap();

    add_tx(
        "2023-07-25",
        "Selling goods",
        "Super Special Bank",
        "200.00",
        "Income",
        "Goods",
        None,
        conn,
    )
    .unwrap();
}

#[test]
fn test_stepper_date() {
    let data = vec![
        "",
        "2020-05-01",
        "2025-05-15",
        "2025-13-01",
        "2025-05-35",
        "2040-05-01",
        "2037-12-31",
        "2022-01-01",
    ]
    .into_iter()
    .map(std::string::ToString::to_string)
    .collect::<Vec<String>>();

    let expected = vec![
        "2022-01-01",
        "2022-05-01",
        "2025-05-16",
        "2025-12-01",
        "2025-05-31",
        "2037-05-01",
        "2037-12-31",
        "2022-01-02",
    ]
    .into_iter()
    .map(std::string::ToString::to_string)
    .collect::<Vec<String>>();

    let result = vec![
        Ok(()),
        Err(SteppingError::InvalidDate),
        Ok(()),
        Err(SteppingError::InvalidDate),
        Err(SteppingError::InvalidDate),
        Err(SteppingError::InvalidDate),
        Ok(()),
        Ok(()),
    ];

    let test_data = Testing {
        data: data.clone(),
        expected,
        result,
    };

    for i in 0..test_data.data.len() {
        let mut to_verify = test_data.data[i].clone();
        let result = test_data.step_date(&mut to_verify, StepType::StepUp, &DateType::Exact);

        assert_eq!(to_verify, test_data.expected[i]);
        assert_eq!(result, test_data.result[i]);
    }

    let expected = vec![
        "2022-01-01",
        "2022-05-01",
        "2025-05-14",
        "2025-12-01",
        "2025-05-31",
        "2037-05-01",
        "2037-12-30",
        "2022-01-01",
    ]
    .into_iter()
    .map(std::string::ToString::to_string)
    .collect::<Vec<String>>();

    let result = vec![
        Ok(()),
        Err(SteppingError::InvalidDate),
        Ok(()),
        Err(SteppingError::InvalidDate),
        Err(SteppingError::InvalidDate),
        Err(SteppingError::InvalidDate),
        Ok(()),
        Ok(()),
    ];

    let test_data = Testing {
        data: data.clone(),
        expected,
        result,
    };

    for i in 0..test_data.data.len() {
        let mut to_verify = test_data.data[i].clone();
        let result = test_data.step_date(&mut to_verify, StepType::StepDown, &DateType::Exact);

        assert_eq!(to_verify, test_data.expected[i]);
        assert_eq!(result, test_data.result[i]);
    }

    let data = vec!["", "2022-01", "2022-13", "2040-01", "2037-12"]
        .into_iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<String>>();

    let expected = vec!["2022-01", "2022-02", "2022-12", "2037-01", "2037-12"]
        .into_iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<String>>();

    let result = vec![
        Ok(()),
        Ok(()),
        Err(SteppingError::InvalidDate),
        Err(SteppingError::InvalidDate),
        Ok(()),
    ];

    let test_data = Testing {
        data: data.clone(),
        expected,
        result,
    };

    for i in 0..test_data.data.len() {
        let mut to_verify = test_data.data[i].clone();
        let result = test_data.step_date(&mut to_verify, StepType::StepUp, &DateType::Monthly);

        assert_eq!(to_verify, test_data.expected[i]);
        assert_eq!(result, test_data.result[i]);
    }

    let expected = vec!["2022-01", "2022-01", "2022-12", "2037-01", "2037-11"]
        .into_iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<String>>();

    let result = vec![
        Ok(()),
        Ok(()),
        Err(SteppingError::InvalidDate),
        Err(SteppingError::InvalidDate),
        Ok(()),
    ];

    let test_data = Testing {
        data: data.clone(),
        expected,
        result,
    };

    for i in 0..test_data.data.len() {
        let mut to_verify = test_data.data[i].clone();
        let result = test_data.step_date(&mut to_verify, StepType::StepDown, &DateType::Monthly);

        assert_eq!(to_verify, test_data.expected[i]);
        assert_eq!(result, test_data.result[i]);
    }

    let data = vec!["", "2022", "2037", "2040"]
        .into_iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<String>>();

    let expected = vec!["2022", "2023", "2037", "2037"]
        .into_iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<String>>();

    let result = vec![Ok(()), Ok(()), Ok(()), Err(SteppingError::InvalidDate)];

    let test_data = Testing {
        data: data.clone(),
        expected,
        result,
    };

    for i in 0..test_data.data.len() {
        let mut to_verify = test_data.data[i].clone();
        let result = test_data.step_date(&mut to_verify, StepType::StepUp, &DateType::Yearly);

        assert_eq!(to_verify, test_data.expected[i]);
        assert_eq!(result, test_data.result[i]);
    }

    let expected = vec!["2022", "2022", "2036", "2037"]
        .into_iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<String>>();

    let result = vec![Ok(()), Ok(()), Ok(()), Err(SteppingError::InvalidDate)];

    let test_data = Testing {
        data: data.clone(),
        expected,
        result,
    };

    for i in 0..test_data.data.len() {
        let mut to_verify = test_data.data[i].clone();
        let result = test_data.step_date(&mut to_verify, StepType::StepDown, &DateType::Yearly);

        assert_eq!(to_verify, test_data.expected[i]);
        assert_eq!(result, test_data.result[i]);
    }
}

#[test]
fn test_stepper_tx_method() {
    let file_name = "stepper_tx_method.sqlite";
    let mut conn = create_test_db(file_name);
    add_dummy_tx(&mut conn);

    let data = vec!["", "Super", "Super Special Bank", "Cash Cow", "Danger Cash"]
        .into_iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<String>>();

    let expected = vec![
        "Super Special Bank",
        "Super Special Bank",
        "Cash Cow",
        "Danger Cash",
        "Super Special Bank",
    ]
    .into_iter()
    .map(std::string::ToString::to_string)
    .collect::<Vec<String>>();

    let result = vec![
        Ok(()),
        Err(SteppingError::InvalidTxMethod),
        Ok(()),
        Ok(()),
        Ok(()),
    ];

    let test_data = Testing {
        data: data.clone(),
        expected,
        result,
    };

    for i in 0..test_data.data.len() {
        let mut to_verify = test_data.data[i].clone();
        let result = test_data.step_tx_method(&mut to_verify, StepType::StepUp, &conn);

        assert_eq!(to_verify, test_data.expected[i]);
        assert_eq!(result, test_data.result[i]);
    }

    let expected = vec![
        "Super Special Bank",
        "Super Special Bank",
        "Danger Cash",
        "Super Special Bank",
        "Cash Cow",
    ]
    .into_iter()
    .map(std::string::ToString::to_string)
    .collect::<Vec<String>>();

    let result = vec![
        Ok(()),
        Err(SteppingError::InvalidTxMethod),
        Ok(()),
        Ok(()),
        Ok(()),
    ];

    let test_data = Testing {
        data,
        expected,
        result,
    };

    for i in 0..test_data.data.len() {
        let mut to_verify = test_data.data[i].clone();
        let result = test_data.step_tx_method(&mut to_verify, StepType::StepDown, &conn);

        assert_eq!(to_verify, test_data.expected[i]);
        assert_eq!(result, test_data.result[i]);
    }

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();
}

#[test]
fn test_stepper_amount() {
    let data = vec!["0", "99999999999.99", "123456", "-123456", ""]
        .into_iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<String>>();

    let expected = vec!["1.00", "9999999999.99", "123457.00", "123457.00", "1.00"]
        .into_iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<String>>();

    let result = vec![Ok(()), Ok(()), Ok(()), Ok(()), Ok(())];

    let test_data = Testing {
        data: data.clone(),
        expected,
        result,
    };

    for i in 0..test_data.data.len() {
        let mut to_verify = test_data.data[i].clone();
        let result = test_data.step_amount(&mut to_verify, StepType::StepUp);

        assert_eq!(to_verify, test_data.expected[i]);
        assert_eq!(result, test_data.result[i]);
    }

    let expected = vec!["0.00", "9999999998.99", "123455.00", "123455.00", "1.00"]
        .into_iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<String>>();

    let result = vec![Ok(()), Ok(()), Ok(()), Ok(()), Ok(())];

    let test_data = Testing {
        data,
        expected,
        result,
    };

    for i in 0..test_data.data.len() {
        let mut to_verify = test_data.data[i].clone();
        let result = test_data.step_amount(&mut to_verify, StepType::StepDown);

        assert_eq!(to_verify, test_data.expected[i]);
        assert_eq!(result, test_data.result[i]);
    }
}

#[test]
fn test_stepper_tx_type() {
    let data = vec![
        "", "e", "E", "t", "T", "i", "I", "v", "Expense", "Income", "Transfer",
    ]
    .into_iter()
    .map(std::string::ToString::to_string)
    .collect::<Vec<String>>();

    let expected = vec![
        "Income", "Transfer", "Transfer", "Income", "Income", "Expense", "Expense", "", "Transfer",
        "Expense", "Income",
    ]
    .into_iter()
    .map(std::string::ToString::to_string)
    .collect::<Vec<String>>();

    let result = vec![
        Ok(()),
        Ok(()),
        Ok(()),
        Ok(()),
        Ok(()),
        Ok(()),
        Ok(()),
        Err(SteppingError::InvalidTxType),
        Ok(()),
        Ok(()),
        Ok(()),
    ];

    let test_data = Testing {
        data: data.clone(),
        expected,
        result,
    };

    for i in 0..test_data.data.len() {
        let mut to_verify = test_data.data[i].clone();
        let result = test_data.step_tx_type(&mut to_verify, StepType::StepUp);

        assert_eq!(to_verify, test_data.expected[i]);
        assert_eq!(result, test_data.result[i]);
    }

    let expected = vec![
        "Income", "Income", "Income", "Expense", "Expense", "Transfer", "Transfer", "", "Income",
        "Transfer", "Expense",
    ]
    .into_iter()
    .map(std::string::ToString::to_string)
    .collect::<Vec<String>>();

    let result = vec![
        Ok(()),
        Ok(()),
        Ok(()),
        Ok(()),
        Ok(()),
        Ok(()),
        Ok(()),
        Err(SteppingError::InvalidTxType),
        Ok(()),
        Ok(()),
        Ok(()),
    ];

    let test_data = Testing {
        data,
        expected,
        result,
    };

    for i in 0..test_data.data.len() {
        let mut to_verify = test_data.data[i].clone();
        let result = test_data.step_tx_type(&mut to_verify, StepType::StepDown);

        assert_eq!(to_verify, test_data.expected[i]);
        assert_eq!(result, test_data.result[i]);
    }
}

#[test]
fn test_stepper_tags() {
    let file_name = "tag_stepper_test.sqlite";
    let mut conn = create_test_db(file_name);

    let data = vec!["Hmm", "", "123"]
        .into_iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<String>>();
    let expected = vec!["", "", ""]
        .into_iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<String>>();
    let result = vec![
        Err(SteppingError::InvalidTags),
        Err(SteppingError::InvalidTags),
        Err(SteppingError::InvalidTags),
    ];

    let test_data = Testing {
        data,
        expected,
        result,
    };

    for i in 0..test_data.data.len() {
        let mut to_verify = test_data.data[i].clone();
        let result = test_data.step_tags(&mut to_verify, "", StepType::StepUp, &conn);

        assert_eq!(to_verify, test_data.expected[i]);
        assert_eq!(result, test_data.result[i]);
    }

    add_dummy_tx(&mut conn);

    let data = vec!["", "car", "fOoD", "Goods", "Car,", "Car, Food", "Boom"]
        .into_iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<String>>();
    let expected = vec!["Car", "Food", "Goods", "Car", "Car, Car", "Car, Goods", ""]
        .into_iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<String>>();
    let result = vec![
        Ok(()),
        Ok(()),
        Ok(()),
        Ok(()),
        Ok(()),
        Ok(()),
        Err(SteppingError::InvalidTags),
    ];

    let test_data = Testing {
        data: data.clone(),
        expected,
        result,
    };

    for i in 0..test_data.data.len() {
        let mut to_verify = test_data.data[i].clone();
        let result = test_data.step_tags(&mut to_verify, "", StepType::StepUp, &conn);

        assert_eq!(to_verify, test_data.expected[i]);
        assert_eq!(result, test_data.result[i]);
    }

    let expected = vec!["Car", "Goods", "Car", "Food", "Car, Car", "Car, Car", ""]
        .into_iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<String>>();
    let result = vec![
        Ok(()),
        Ok(()),
        Ok(()),
        Ok(()),
        Ok(()),
        Ok(()),
        Err(SteppingError::InvalidTags),
    ];

    let test_data = Testing {
        data: data.clone(),
        expected,
        result,
    };

    for i in 0..test_data.data.len() {
        let mut to_verify = test_data.data[i].clone();
        let result = test_data.step_tags(&mut to_verify, "", StepType::StepDown, &conn);

        assert_eq!(to_verify, test_data.expected[i]);
        assert_eq!(result, test_data.result[i]);
    }

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();
}
