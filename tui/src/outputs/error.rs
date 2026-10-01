use std::error::Error;
use std::io::Error as ioError;
use strum_macros::Display;

#[derive(Debug, Display)]
pub enum UiHandlingError {
    #[strum(to_string = "Error while trying to draw widgets. Error: {0}")]
    Drawing(String),
    #[strum(to_string = "Error while polling for keyboard input. {0}")]
    Polling(ioError),
}

impl Error for UiHandlingError {}

#[derive(Debug, PartialEq, Display)]
pub enum CheckingError {
    #[strum(to_string = "Date: Date cannot be empty")]
    EmptyDate,
    #[strum(to_string = "Tx Method: TX Method cannot be empty")]
    EmptyMethod,
    #[strum(to_string = "Amount: Amount cannot be empty")]
    EmptyAmount,
    #[strum(to_string = "Tx Type: Transaction Type cannot be empty")]
    EmptyTxType,
    #[strum(to_string = "Tx Method: From and To methods cannot be the same for Transfer")]
    SameTxMethod,
    #[strum(to_string = "Frequency: Frequency cannot be empty")]
    EmptyFrequency,
    #[strum(to_string = "Recur Interval: Recur Interval cannot be empty")]
    EmptyRecurInterval,
    #[strum(to_string = "Recur Value: Recur Value cannot be empty for this frequency")]
    EmptyRecurValue,
    #[strum(to_string = "Recur Month: Recur Month cannot be empty for Yearly frequency")]
    EmptyRecurMonth,
}

impl Error for CheckingError {}
