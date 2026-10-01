use std::fmt;
use thiserror::Error;

pub enum Output {
    Nothing(Field),
    Accepted(Field),
}

impl fmt::Display for Output {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Output::Nothing(value) => write!(f, "{value}: Nothing to check"),
            Output::Accepted(value) => write!(f, "{value}: Accepted"),
        }
    }
}

#[derive(PartialEq, Debug, Error)]
pub enum Field {
    #[error("Date")]
    Date,
    #[error("Tx Method")]
    TxMethod,
    #[error("Amount")]
    Amount,
    #[error("Tx Type")]
    TxType,
    #[error("Tags")]
    Tags,
    #[error("Frequency")]
    Frequency,
    #[error("Recur Interval")]
    RecurInterval,
    #[error("Recur Value")]
    RecurValue,
    #[error("Recur Month")]
    RecurMonth,
}

#[derive(Debug, Error)]
pub enum VerifierError {
    #[error("Date: Unknown date")]
    InvalidDate,
    #[error("Date: Year length not acceptable. Example Date: 2022-05-01")]
    InvalidYear,
    #[error("Date: Month length not acceptable. Example Date: 2022-05-01")]
    InvalidMonth,
    #[error("Date: Day length not acceptable. Example Date: 2022-05-01")]
    InvalidDay,
    #[error("Date: Month must be between 01-12")]
    MonthTooBig,
    #[error("Date: Day must be between 01-31")]
    DayTooBig,
    #[error("Date: Date not acceptable and possibly non-existing")]
    NonExistingDate,
    #[error("Amount: Value must be bigger than zero")]
    AmountBelowZero,
    #[error("TX Method: Transaction Method not found")]
    InvalidTxMethod,
    #[error(
        "TX Type: Transaction Type not acceptable. Values: Expense/Income/Transfer/Borrow/Borrow Repay/Lend/Lend Repay/I/E/T/B/BR/L/LR"
    )]
    InvalidTxType,
    #[error("{0}: Error acquired while validating input")]
    ParsingError(Field),
    #[error("Amount: TX Method cannot be empty. Value of B cannot be determined")]
    InvalidBValue,
    #[error("Tags: Non-existing tags cannot be accepted")]
    NonExistingTag,
    #[error("Frequency: Value not acceptable. Values: Daily/Weekly/Monthly/Yearly")]
    InvalidFrequency,
    #[error("Recur Interval: Value must be a whole number of at least 1")]
    InvalidRecurInterval,
    #[error("Recur Value: Day of month must be between 1-31")]
    InvalidRecurValueMonthly,
    #[error("Recur Value: Day of week not acceptable. Values: Sunday-Saturday")]
    InvalidRecurValueWeekly,
    #[error("Recur Month: Value not acceptable. Values: January-December")]
    InvalidRecurMonth,
    #[error("Others: Something went wrong while verifying input. Error: {0}")]
    Others(String),
}

#[derive(Debug, Error)]
pub enum SteppingError {
    #[error("Date: Failed to step due to invalid date format")]
    InvalidDate,
    #[error("Tx Method: Failed to step as the tx method does not exists")]
    InvalidTxMethod,
    #[error("Amount: Failed to step due to invalid amount format")]
    InvalidAmount,
    #[error("Tx Type: Failed to step due to invalid tx type")]
    InvalidTxType,
    #[error("Tags: Failed to step as the tag does not exists")]
    InvalidTags,
    #[error("Amount: Failed to step value. Value of B cannot be determined")]
    UnknownBValue,
    #[error("Frequency: Failed to step as the frequency is invalid")]
    InvalidFrequency,
    #[error("Recur Interval: Failed to step due to invalid interval format")]
    InvalidRecurInterval,
    #[error("Recur Value: Failed to step as the value does not exist for the current frequency")]
    InvalidRecurValue,
    #[error("Recur Month: Failed to step as the month does not exists")]
    InvalidRecurMonth,
    #[error("{0}: Error acquired while validating input")]
    ParsingError(Field),
}
