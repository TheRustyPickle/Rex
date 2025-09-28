use rusqlite::Error as sqlError;
use std::error::Error;
use std::fmt::{self, Display, Result};
use std::io::Error as ioError;

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

pub enum TxUpdateError {
    Add(sqlError),
    Edit(sqlError),
    Delete(String),
}

impl Display for TxUpdateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result {
        match self {
            TxUpdateError::Add(e) => {
                write!(
                    f,
                    "Delete Transaction: Something went wrong. Failed to delete transaction. Error: \n{e}",
                )
            }
            TxUpdateError::Edit(e) => write!(
                f,
                "Edit Transaction: Something went wrong. Failed to edit transaction. Error: {e}",
            ),
            TxUpdateError::Delete(e) => {
                write!(
                    f,
                    "Add Transaction: Something went wrong. Failed to add transaction. Error: {e}",
                )
            }
        }
    }
}
