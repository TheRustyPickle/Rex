use crate::db::{add_new_tx, delete_tx};
use crate::db::{get_all_tx_methods, get_last_balances, StatusChecker};
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
pub struct AddTxData {
    date: String,
    details: String,
    tx_method: String,
    amount: String,
    tx_type: String,
    tags: String,
    pub tx_status: Vec<String>,
    editing_tx: bool,
    id_num: i32,
}

impl StatusChecker for AddTxData {}

impl AddTxData {
    /// Creates an instance of the struct however the date field is
    /// edited with the current local date of the device.
    pub fn new() -> Self {
        let cu_date = Local::today().to_string();
        let formatted_cu_date = &cu_date[0..10];
        AddTxData {
            date: formatted_cu_date.to_string(),
            details: "".to_string(),
            tx_method: "".to_string(),
            amount: "".to_string(),
            tx_type: "".to_string(),
            tags: "".to_string(),
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
        tx_method: &str,
        amount: &str,
        tx_type: &str,
        tags: &str,
        id_num: i32,
    ) -> Self {
        let splitted = date.split('-');
        let data = splitted.collect::<Vec<&str>>();
        let year = data[2];
        let month = data[1];
        let day = data[0];

        let new_date = format!("{}-{}-{}", year, month, day);
        AddTxData {
            date: new_date,
            details: details.to_string(),
            tx_method: tx_method.to_string(),
            amount: amount.to_string(),
            tx_type: tx_type.to_string(),
            tags: tags.to_string(),
            tx_status: Vec::new(),
            editing_tx: true,
            id_num,
        }
    }

    /// Sends out all the collected data that has been inputted into the Add Transaction widgets
    ///  that is going to be used for creating a new transaction
    pub fn get_all_texts(&self) -> Vec<&str> {
        vec![
            &self.date,
            &self.details,
            &self.tx_method,
            &self.amount,
            &self.tx_type,
            &self.tags,
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

    /// Used to add a new character to the tx method value that is being inputted by the
    /// user following each key press. Takes a bool value to represent backspace pressing.
    pub fn edit_tx_method(&mut self, text: char, pop_last: bool) {
        match pop_last {
            true => {
                if !self.tx_method.is_empty() {
                    self.tx_method.pop().unwrap();
                }
            }
            false => self.tx_method = format!("{}{text}", self.tx_method),
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

    /// Used to add a new character to the tx type value that is being inputted by the
    /// user following each key press. Takes a bool value to represent backspace pressing.
    pub fn edit_tx_type(&mut self, text: char, pop_last: bool) {
        match pop_last {
            true => {
                if !self.tx_type.is_empty() {
                    self.tx_type.pop().unwrap();
                }
            }
            false => self.tx_type = format!("{}{text}", self.tx_type),
        }
    }

    pub fn edit_tags(&mut self, text: char, pop_last: bool) {
        match pop_last {
            true => {
                if !self.tags.is_empty() {
                    self.tags.pop().unwrap();
                }
            }
            false => self.tags = format!("{}{text}", self.tags),
        }
    }

    /// Collects all the data for the transaction and calls the function
    /// that pushes them to the database.
    pub fn add_tx(&mut self) -> String {
        if self.date.is_empty() {
            return "Date: Date cannot be empty".to_string();
        } else if self.details.is_empty() {
            return "Details: Details cannot be empty".to_string();
        } else if self.tx_method.is_empty() {
            return "Tx Method: Transaction method cannot be empty".to_string();
        } else if self.amount.is_empty() {
            return "Amount: Amount cannot be empty".to_string();
        } else if self.tx_type.is_empty() {
            return "Tx Type: Transaction Type cannot be empty".to_string();
        }
        if self.tags == "" {
            self.tags = "Unknown".to_string();
        }

        if self.editing_tx {
            self.editing_tx = false;
            let status = delete_tx(self.id_num as usize, "data.sqlite");
            match status {
                Ok(_) => {}
                Err(e) => {
                    return format!(
                        "Edit Transaction: Something went wrong while editing transaction {}",
                        e
                    )
                }
            }

            let status_add = add_new_tx(
                &self.date,
                &self.details,
                &self.tx_method,
                &self.amount,
                &self.tx_type,
                &self.tags,
                "data.sqlite",
                Some(&self.id_num.to_string()),
            );

            match status_add {
                Ok(_) => String::new(),
                Err(e) => format!("Edit Transaction: Something went wrong {}", e),
            }
        } else {
            let status = add_new_tx(
                &self.date,
                &self.details,
                &self.tx_method,
                &self.amount,
                &self.tx_type,
                &self.tags,
                "data.sqlite",
                None,
            );
            match status {
                Ok(_) => String::new(),
                Err(e) => format!("Add Transaction: Something went wrong {}", e),
            }
        }
    }

    /// Adds a status after a checking is complete. Used for the Status widget
    /// on Add Transaction page and called upon on Enter/Esc presses.
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
    pub fn check_tx_method(&mut self, conn: &Connection) -> Result<String, Box<dyn Error>> {
        let mut cu_method = self.tx_method.clone();

        let status = self.verify_tx_method(&mut cu_method, conn)?;

        self.tx_method = cu_method;
        Ok(status)
    }

    /// Checks the inputted Transaction Method by the user upon pressing Enter/Esc for various error.
    pub fn check_amount(&mut self, conn: &Connection) -> Result<String, Box<dyn Error>> {
        let mut user_amount = self.amount.clone().to_lowercase();
        if user_amount.contains("b") && !self.tx_method.is_empty() {
            let all_methods = get_all_tx_methods(&conn);

            if !all_methods.contains(&self.tx_method) {
                return Ok(String::from(
                    "Amount: TX Method not found. B value cannot be accepted",
                ));
            }

            let last_balances = get_last_balances(&conn, &all_methods);

            for x in 0..all_methods.len() {
                if all_methods[x] == self.tx_method {
                    user_amount = user_amount.replace("b", &last_balances[x]);
                    self.amount = self.amount.replace("b", &last_balances[x]);
                    break;
                }
            }
        } else if user_amount.contains("b") && self.tx_method.is_empty() {
            return Ok(String::from(
                "Amount: TX Method cannot be empty. Value of B cannot be determined",
            ));
        }

        let status = self.verify_amount(&mut user_amount)?;

        self.amount = user_amount;
        Ok(status)
    }

    /// Checks the inputted Transaction Method by the user upon pressing Enter/Esc for various error.
    pub fn check_tx_type(&mut self) -> Result<String, Box<dyn Error>> {
        let mut tx_type = self.tx_type.clone();

        let status = self.verify_tx_type(&mut tx_type)?;

        self.tx_type = tx_type;
        Ok(status)
    }
}
