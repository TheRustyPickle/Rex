use std::fmt;
use std::io::Error;
use std::process::Output;

#[derive(Debug)]
pub enum TerminalExecutionError {
    NotFound(Output),
    ExecutionFailed(Error),
}

impl fmt::Display for TerminalExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TerminalExecutionError::NotFound(output) => write!(f, "Error while trying to run any console/terminal. Use a terminal/console to run the app. Output:\n\n{:?}", output),
            TerminalExecutionError::ExecutionFailed(error) => write!(f, "Error while processing commands. Use a terminal/console to run the app. Output: {}", error),
        }
    }
}

impl std::error::Error for TerminalExecutionError {}

#[derive(Debug)]
pub enum UiHandlingError {
    DrawingError(Error),
    PollingError(Error),
}

impl fmt::Display for UiHandlingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UiHandlingError::DrawingError(err) => {
                write!(f, "Error while trying to draw widgets. Error: {}", err)
            }
            UiHandlingError::PollingError(err) => {
                write!(f, "Error while polling for keyboard input. {err}")
            }
        }
    }
}

impl std::error::Error for UiHandlingError {}

#[derive(Debug)]
pub enum CheckingError {
    EmptyDate,
    EmptyMethod,
    EmptyAmount,
    EmptyTxType,
    SameTxMethod,
}

impl fmt::Display for CheckingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

impl std::error::Error for CheckingError {}

pub enum SteppingError {
    InvalidDate,
    InvalidTxMethod,
    InvalidAmount,
    InvalidTxType,
    InvalidTags,
}

impl fmt::Display for SteppingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
        }
    }
}
