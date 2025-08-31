use rusqlite::Error as sqlError;
use std::error::Error;
use std::fmt::{self, Display, Result};
use std::io::Error as ioError;
use std::process::Output;

#[derive(Debug)]
pub enum TerminalExecutionError {
    NotFound(Output),
    ExecutionFailed(ioError),
}

impl Display for TerminalExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result {
        match self {
            TerminalExecutionError::NotFound(output) => write!(
                f,
                "Error while trying to run any console/terminal. Use a terminal/console to run the app. Output:\n\n{output:?}",
            ),
            TerminalExecutionError::ExecutionFailed(error) => write!(
                f,
                "Error while processing commands. Use a terminal/console to run the app. Output: {error}",
            ),
        }
    }
}

impl Error for TerminalExecutionError {}

#[derive(Debug)]
pub enum UiHandlingError {
    DrawingError(ioError),
    PollingError(ioError),
}

impl Display for UiHandlingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result {
        match self {
            UiHandlingError::DrawingError(err) => {
                write!(f, "Error while trying to draw widgets. Error: {err}",)
            }
            UiHandlingError::PollingError(err) => {
                write!(f, "Error while polling for keyboard input. {err}")
            }
        }
    }
}

impl Error for UiHandlingError {}

#[derive(Debug, PartialEq)]
pub enum CheckingError {
    EmptyDate,
    EmptyMethod,
    EmptyAmount,
    EmptyTxType,
    SameTxMethod,
}

impl Display for CheckingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result {
        match self {
            CheckingError::EmptyDate => write!(f, "Date: Date cannot be empty"),
            CheckingError::EmptyMethod => write!(f, "Tx Method: TX Method cannot be empty"),
            CheckingError::EmptyAmount => write!(f, "Amount: Amount cannot be empty"),
            CheckingError::EmptyTxType => write!(f, "Tx Type: Transaction Type cannot be empty"),
            CheckingError::SameTxMethod => write!(
                f,
                "Tx Method: From and To methods cannot be the same for Transfer"
            ),
        }
    }
}

impl Error for CheckingError {}

#[derive(Debug, PartialEq)]
pub enum SteppingError {
    InvalidDate,
    InvalidTxMethod,
    InvalidAmount,
    InvalidTxType,
    InvalidTags,
    UnknownBValue,
}

impl Display for SteppingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result {
        match self {
            SteppingError::InvalidDate => {
                write!(f, "Date: Failed to step due to invalid date format")
            }
            SteppingError::InvalidTxMethod => write!(
                f,
                "Tx Method: Failed to step as the tx method does not exists"
            ),
            SteppingError::InvalidAmount => {
                write!(f, "Amount: Failed to step due to invalid amount format")
            }
            SteppingError::InvalidTxType => {
                write!(f, "Tx Type: Failed to step due to invalid tx type")
            }
            SteppingError::InvalidTags => {
                write!(f, "Tags: Failed to step as the tag does not exists")
            }
            SteppingError::UnknownBValue => write!(
                f,
                "Amount: Failed to step value. Value of B cannot be determined"
            ),
        }
    }
}

pub enum TxUpdateError {
    FailedAddTx(sqlError),
    FailedEditTx(sqlError),
    FailedDeleteTx(String),
}

impl Display for TxUpdateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result {
        match self {
            TxUpdateError::FailedAddTx(e) => {
                write!(
                    f,
                    "Delete Transaction: Something went wrong. Failed to delete transaction. Error: \n{e}",
                )
            }
            TxUpdateError::FailedEditTx(e) => write!(
                f,
                "Edit Transaction: Something went wrong. Failed to edit transaction. Error: {e}",
            ),
            TxUpdateError::FailedDeleteTx(e) => {
                write!(
                    f,
                    "Add Transaction: Something went wrong. Failed to add transaction. Error: {e}",
                )
            }
        }
    }
}
