use crate::db::StatusChecker;
use crate::db::{add_new_tx, delete_tx};
use chrono::prelude::Local;
use rusqlite::Connection;
use std::error::Error;

/// The struct maintains the data that has been entered by the
/// user to the relevant field in order to create a new transaction and push it
/// to the database. This is also designed to parse and validate the data that
/// is being passed by the user. tx_status values contains the status comment whether
/// if the user inputted value was accepted or rejected and shown in the Status widget on
/// the Add Transaction page.
///
/// tx_status : `["Date: Date Accepted", "Tx Method: Transaction Method Not Found"]`
pub struct TransferData {
    date: String,
    details: String,
    from: String,
    to: String,
    amount: String,
    tx_type: String,
    pub tx_status: Vec<String>,
    editing_tx: bool,
    id_num: i32,
}

impl StatusChecker for TransferData {}

impl TransferData {
    /// Creates an instance of the struct however the date field is
    /// edited with the current local date of the device.
    pub fn new() -> Self {
        let cu_date = Local::today().to_string();
        let formatted_cu_date = &cu_date[0..10];
        TransferData {
            date: formatted_cu_date.to_string(),
            details: "".to_string(),
            from: "".to_string(),
            to: "".to_string(),
            amount: "".to_string(),
            tx_type: "Transfer".to_string(),
            tx_status: Vec::new(),
            editing_tx: false,
            id_num: 0,
        }
    }

    /// Used to adding custom pre-defined data inside the widgets of Add Transaction Page.
    /// Currently used on Editing transaction.
    pub fn custom(
        date: &str,
        details: &str,
        from: &str,
        to: &str,
        amount: &str,
        id_num: i32,
    ) -> Self {
        let splitted = date.split('-');
        let data = splitted.collect::<Vec<&str>>();
        let year = data[2];
        let month = data[1];
        let day = data[0];

        let new_date = format!("{}-{}-{}", year, month, day);

        TransferData {
            date: new_date,
            details: details.to_string(),
            from: from.to_string(),
            to: to.to_string(),
            amount: amount.to_string(),
            tx_type: "Transfer".to_string(),
            tx_status: Vec::new(),
            editing_tx: true,
            id_num,
        }
    }

    /// Sends out all the collected data that has been inputted into the Add Transaction widgets
    /// that is going to be used for creating a new transaction
    pub fn get_all_texts(&self) -> Vec<&str> {
        vec![
            &self.date,
            &self.details,
            &self.from,
            &self.to,
            &self.amount,
            &self.tx_type,
        ]
    }

    /// Used to add a new character to the date value that is being inputted by the
    /// user following each key press. Takes a bool value to represent backspace pressing.
    pub fn edit_date(&mut self, text: char, pop_last: bool) {
        match pop_last {
            true => {
                if !self.date.is_empty() {
                    self.date.pop().unwrap();
                }
            }
            false => self.date = format!("{}{text}", self.date),
        }
    }

    /// Used to add a new character to the details value that is being inputted by the
    /// user following each key press. Takes a bool value to represent backspace pressing.
    pub fn edit_details(&mut self, text: char, pop_last: bool) {
        match pop_last {
            true => {
                if !self.details.is_empty() {
                    self.details.pop().unwrap();
                }
            }
            false => self.details = format!("{}{text}", self.details),
        }
    }

    /// Used to add a new character to the From TX Method value that is being inputted by the
    /// user following each key press. Takes a bool value to represent backspace pressing.
    pub fn edit_from(&mut self, text: char, pop_last: bool) {
        match pop_last {
            true => {
                if !self.from.is_empty() {
                    self.from.pop().unwrap();
                }
            }
            false => self.from = format!("{}{text}", self.from),
        }
    }

    /// Used to add a new character to the To TX Method value that is being inputted by the
    /// user following each key press. Takes a bool value to represent backspace pressing.
    pub fn edit_to(&mut self, text: char, pop_last: bool) {
        match pop_last {
            true => {
                if !self.to.is_empty() {
                    self.to.pop().unwrap();
                }
            }
            false => self.to = format!("{}{text}", self.to),
        }
    }

    /// Used to add a new character to the amount value that is being inputted by the
    /// user following each key press. Takes a bool value to represent backspace pressing.
    pub fn edit_amount(&mut self, text: char, pop_last: bool) {
        match pop_last {
            true => {
                if !self.amount.is_empty() {
                    self.amount.pop().unwrap();
                }
            }
            false => {
                let data = format!("{}{text}", self.amount);
                self.amount = data;
            }
        }
    }

    /// Collects all the data, verifies that all fields are complete for the transaction and calls the function
    /// that pushes them to the database.
    pub fn add_tx(&mut self) -> String {
        // Checks that none of the ui fields are not empty
        if self.date.is_empty() {
            return "Date: Date cannot be empty".to_string();
        } else if self.details.is_empty() {
            return "Details: Details cannot be empty".to_string();
        } else if self.from.is_empty() {
            return "From TX Method: Transaction method cannot be empty".to_string();
        } else if self.to.is_empty() {
            return "To TX Method: Transaction method cannot be empty".to_string();
        } else if self.from == self.to && !&self.from.is_empty() && !&self.to.is_empty() {
            return "Tx Method: Transaction method From and To cannot be the same".to_string();
        } else if self.amount.is_empty() {
            return "Amount: Amount cannot be empty".to_string();
        } else if self.tx_type.is_empty() {
            return "Tx Type: Transaction Type cannot be empty".to_string();
        }

        let tx_method = format!("{} to {}", self.from, self.to);

        if self.editing_tx {
            // if we are editing a tx delete the selected transaction so we can create it again
            // with the new details
            self.editing_tx = false;
            let status = delete_tx(self.id_num as usize, "data.sqlite");
            match status {
                Ok(_) => {}
                Err(e) => {
                    return format!(
                        "Edit Transfer: Something went wrong while editing transaction {}",
                        e
                    )
                }
            }
            let status_add = add_new_tx(
                &self.date,
                &self.details,
                &tx_method,
                &self.amount,
                &self.tx_type,
                "data.sqlite",
                Some(&self.id_num.to_string()),
            );

            match status_add {
                Ok(_) => String::new(),
                Err(e) => format!("Edit Transfer: Something went wrong {}", e),
            }
        } else {
            let status = add_new_tx(
                &self.date,
                &self.details,
                &tx_method,
                &self.amount,
                &self.tx_type,
                "data.sqlite",
                None,
            );
            match status {
                Ok(_) => String::new(),
                Err(e) => format!("Add Transfer: Something went wrong {}", e),
            }
        }
    }

    /// Adds a status after a checking is complete. Used for the Status widget
    /// on Add Transaction/Transfer page and called upon on Enter/Esc presses.
    /// Removes the earliest status if total status number passes 20.
    pub fn add_tx_status(&mut self, data: &str) {
        if self.tx_status.len() == 20 {
            self.tx_status.remove(0);
        }
        self.tx_status.push(data.to_string());
    }

    /// Checks the inputted Date by the user upon pressing Enter/Esc for various error.
    pub fn check_date(&mut self) -> Result<String, Box<dyn Error>> {
        let mut user_date = self.date.clone();

        let status = self.verify_date(&mut user_date)?;

        self.date = user_date;
        Ok(status)
    }

    /// Checks the inputted Transaction Method by the user upon pressing Enter/Esc for various error.
    pub fn check_from(&mut self, conn: &Connection) -> Result<String, Box<dyn Error>> {
        let mut cu_method = self.from.clone();

        let mut status = self.verify_tx_method(&mut cu_method, conn)?;
        if cu_method == self.to && !self.to.is_empty() && !self.from.is_empty() {
            return Ok(
                "From TX Method: To and From Transaction Methods cannot be the same".to_string(),
            );
        }

        status = status.replace("TX Method", "From TX Method");

        self.from = cu_method;
        Ok(status)
    }

    /// Checks the inputted Transaction Method by the user upon pressing Enter/Esc for various error.
    pub fn check_to(&mut self, conn: &Connection) -> Result<String, Box<dyn Error>> {
        let mut cu_method = self.to.clone();

        let mut status = self.verify_tx_method(&mut cu_method, conn)?;
        if cu_method == self.from && !self.to.is_empty() && !self.from.is_empty() {
            return Ok(
                "To TX Method: To and From Transaction Methods cannot be the same".to_string(),
            );
        }

        status = status.replace("TX Method", "To TX Method");

        self.to = cu_method;
        Ok(status)
    }

    /// Checks the inputted amount by the user upon pressing Enter/Esc for various error.
    pub fn check_amount(&mut self) -> Result<String, Box<dyn Error>> {
        let mut user_amount = self.amount.clone();

        let status = self.verify_amount(&mut user_amount)?;

        self.amount = user_amount;
        Ok(status)
    }
}
