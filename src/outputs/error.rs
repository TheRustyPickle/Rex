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
