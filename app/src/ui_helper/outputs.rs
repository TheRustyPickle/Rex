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
    #[error("TX Type: Transaction Type not acceptable. Values: Expense/Income/E/I")]
    InvalidTxType,
    #[error("{0}: Error acquired while validating input")]
    ParsingError(Field),
    #[error("Amount: TX Method cannot be empty. Value of B cannot be determined")]
    InvalidBValue,
    #[error("Tags: Non-existing tags cannot be accepted")]
    NonExistingTag,
}
