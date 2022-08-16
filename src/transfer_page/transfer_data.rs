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
        tx_type: &str,
        id_num: i32,
    ) -> Self {
        let splitted = date.split("-");
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
            tx_type: tx_type.to_string(),
            tx_status: Vec::new(),
            editing_tx: true,
            id_num: id_num,
        }
    }

    /// Sends out all the collected data that has been inputted into the Add Transaction widgets
    ///  that is going to be used for creating a new transaction
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

    /// Used to add a new character to the date value being inputted by the
    /// user following each key press. Takes a bool value to represent backspace pressing.
    pub fn edit_date(&mut self, text: char, pop_last: bool) {
        match pop_last {
            true => {
                if self.date.len() > 0 {
                    self.date.pop().unwrap();
                }
            }
            false => self.date = format!("{}{text}", self.date),
        }
    }

    /// Used to add a new character to the details value being inputted by the
    /// user following each key press. Takes a bool value to represent backspace pressing.
    pub fn edit_details(&mut self, text: char, pop_last: bool) {
        match pop_last {
            true => {
                if self.details.len() > 0 {
                    self.details.pop().unwrap();
                }
            }
            false => self.details = format!("{}{text}", self.details),
        }
    }

    /// Used to add a new character to the tx method value being inputted by the
    /// user following each key press. Takes a bool value to represent backspace pressing.
    pub fn edit_from(&mut self, text: char, pop_last: bool) {
        match pop_last {
            true => {
                if self.from.len() > 0 {
                    self.from.pop().unwrap();
                }
            }
            false => self.from = format!("{}{text}", self.from),
        }
    }

    pub fn edit_to(&mut self, text: char, pop_last: bool) {
        match pop_last {
            true => {
                if self.to.len() > 0 {
                    self.to.pop().unwrap();
                }
            }
            false => self.to = format!("{}{text}", self.to),
        }
    }

    /// Used to add a new character to the amount value being inputted by the
    /// user following each key press. Takes a bool value to represent backspace pressing.
    pub fn edit_amount(&mut self, text: char, pop_last: bool) {
        match pop_last {
            true => {
                if self.amount.len() > 0 {
                    self.amount.pop().unwrap();
                }
            }
            false => {
                let data = format!("{}{text}", self.amount);
                self.amount = data;
            }
        }
    }
    //TODO fix saving transfer tx
    /// Collects all the data for the transaction and calls the function
    /// that pushes them to the database.
    /*pub fn add_tx(&mut self) -> String {
        if &self.date == "" {
            return format!("Date: Date cannot be empty");
        } else if &self.details == "" {
            return format!("Details: Details cannot be empty");
        } else if &self.tx_method == "" {
            return format!("Tx Method: Transaction method cannot be empty");
        } else if &self.amount == "" {
            return format!("Amount: Amount cannot be empty");
        } else if &self.tx_type == "" {
            return format!("Tx Type: Transaction Type cannot be empty");
        }

        if self.editing_tx == true {
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
                "data.sqlite",
                Some(&self.id_num.to_string()),
            );

            match status_add {
                Ok(_) => return format!(""),
                Err(e) => return format!("Edit Transaction: Something went wrong {}", e),
            }
        } else {
            let status = add_new_tx(
                &self.date,
                &self.details,
                &self.tx_method,
                &self.amount,
                &self.tx_type,
                "data.sqlite",
                None,
            );
            match status {
                Ok(_) => return format!(""),
                Err(e) => return format!("Add Transaction: Something went wrong {}", e),
            }
        }
    }*/

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
    pub fn check_from(&mut self, conn: &Connection) -> Result<String, Box<dyn Error>> {
        let mut cu_method = self.from.clone();

        let status = self.verify_tx_method(&mut cu_method, conn)?;

        self.from = cu_method;
        Ok(status)
    }

    pub fn check_to(&mut self, conn: &Connection) -> Result<String, Box<dyn Error>> {
        let mut cu_method = self.to.clone();

        let status = self.verify_tx_method(&mut cu_method, conn)?;

        self.to = cu_method;
        Ok(status)
    }

    /// Checks the inputted Transaction Method by the user upon pressing Enter/Esc for various error.
    pub fn check_amount(&mut self) -> Result<String, Box<dyn Error>> {
        let mut user_amount = self.amount.clone();

        let status = self.verify_amount(&mut user_amount)?;

        self.amount = user_amount;
        Ok(status)
    }
}
