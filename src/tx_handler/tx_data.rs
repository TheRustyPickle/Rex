use crate::outputs::{NAType, SavingError, VerifyingOutput};
use crate::tx_handler::{add_tx, delete_tx};
use crate::utility::traits::DataVerifier;
use crate::utility::{get_all_tx_methods, get_last_balances};
use chrono::prelude::Local;
use rusqlite::Connection;
pub struct TxData {
    date: String,
    details: String,
    from_method: String,
    to_method: String,
    amount: String,
    tx_type: String,
    tags: String,
    pub tx_status: Vec<String>,
    editing_tx: bool,
    id_num: i32,
}

impl DataVerifier for TxData {}

impl TxData {
    /// Creates an instance of the struct however the date field is
    /// edited with the current local date of the device.
    pub fn new() -> Self {
        let cu_date = Local::now().to_string();
        let formatted_cu_date = &cu_date[0..10];
        TxData {
            date: formatted_cu_date.to_string(),
            details: String::new(),
            from_method: String::new(),
            to_method: String::new(),
            amount: String::new(),
            tx_type: String::new(),
            tags: String::new(),
            tx_status: Vec::new(),
            editing_tx: false,
            id_num: 0,
        }
    }

    pub fn new_transfer() -> Self {
        let cu_date = Local::now().to_string();
        let formatted_cu_date = &cu_date[0..10];
        TxData {
            date: formatted_cu_date.to_string(),
            details: String::new(),
            from_method: String::new(),
            to_method: String::new(),
            amount: String::new(),
            tx_type: "Transfer".to_string(),
            tags: String::new(),
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
        from_method: &str,
        to_method: &str,
        amount: &str,
        tx_type: &str,
        tags: &str,
        id_num: i32,
    ) -> Self {
        let data = date.split('-').collect::<Vec<&str>>();
        let year = data[2];
        let month = data[1];
        let day = data[0];

        let new_date = format!("{}-{}-{}", year, month, day);
        TxData {
            date: new_date,
            details: details.to_string(),
            from_method: from_method.to_string(),
            to_method: to_method.to_string(),
            amount: amount.to_string(),
            tx_type: tx_type.to_string(),
            tags: tags.to_string(),
            tx_status: Vec::new(),
            editing_tx: true,
            id_num,
        }
    }

    pub fn get_all_texts(&self) -> Vec<&str> {
        vec![
            &self.date,
            &self.details,
            &self.from_method,
            &self.to_method,
            &self.amount,
            &self.tx_type,
            &self.tags,
        ]
    }

    pub fn get_tx_method(&self) -> String {
        if self.tx_type == "Transfer" {
            format!("{} to {}", self.from_method, self.to_method)
        } else {
            self.from_method.to_string()
        }
    }

    pub fn edit_date(&mut self, to_add: Option<char>) {
        match to_add {
            Some(ch) => self.date.push(ch),
            None => {
                if !self.date.is_empty() {
                    self.date.pop().unwrap();
                }
            }
        }
    }

    pub fn edit_details(&mut self, to_add: Option<char>) {
        match to_add {
            Some(ch) => self.details.push(ch),
            None => {
                if !self.details.is_empty() {
                    self.details.pop().unwrap();
                }
            }
        }
    }

    pub fn edit_from_method(&mut self, to_add: Option<char>) {
        match to_add {
            Some(ch) => self.from_method.push(ch),
            None => {
                if !self.from_method.is_empty() {
                    self.from_method.pop().unwrap();
                }
            }
        }
    }

    pub fn edit_to_method(&mut self, to_add: Option<char>) {
        match to_add {
            Some(ch) => self.to_method.push(ch),
            None => {
                if !self.to_method.is_empty() {
                    self.to_method.pop().unwrap();
                }
            }
        }
    }

    pub fn edit_amount(&mut self, to_add: Option<char>) {
        match to_add {
            Some(ch) => self.amount.push(ch),
            None => {
                if !self.amount.is_empty() {
                    self.amount.pop().unwrap();
                }
            }
        }
    }

    pub fn edit_tx_type(&mut self, to_add: Option<char>) {
        match to_add {
            Some(ch) => self.tx_type.push(ch),
            None => {
                if !self.tx_type.is_empty() {
                    self.tx_type.pop().unwrap();
                }
            }
        }
    }

    pub fn edit_tags(&mut self, to_add: Option<char>) {
        match to_add {
            Some(ch) => self.tags.push(ch),
            None => {
                if !self.tags.is_empty() {
                    self.tags.pop().unwrap();
                }
            }
        }
    }

    pub fn add_tx(&mut self) -> String {
        if let Some(output) = self.check_all_fields() {
            return output.to_string();
        }

        let tx_method = self.get_tx_method();

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

            let status_add = add_tx(
                &self.date,
                &self.details,
                &tx_method,
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
            let status = add_tx(
                &self.date,
                &self.details,
                &tx_method,
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

    pub fn add_tx_status(&mut self, data: String) {
        if self.tx_status.len() == 30 {
            self.tx_status.remove(0);
        }
        self.tx_status.push(data);
    }

    /// Checks the inputted Date by the user upon pressing Enter/Esc for various error.
    pub fn check_date(&mut self) -> VerifyingOutput {
        let mut user_date = self.date.clone();
        let status = self.verify_date(&mut user_date);

        self.date = user_date;
        status
    }

    /// Checks the inputted Transaction Method by the user upon pressing Enter/Esc for various error.
    pub fn check_from_method(&mut self, conn: &Connection) -> VerifyingOutput {
        let mut cu_method = self.from_method.clone();

        let status = self.verify_tx_method(&mut cu_method, conn);

        self.from_method = cu_method;
        status
    }

    /// Checks the inputted Transaction Method by the user upon pressing Enter/Esc for various error.
    pub fn check_to_method(&mut self, conn: &Connection) -> VerifyingOutput {
        let mut cu_method = self.to_method.clone();

        let status = self.verify_tx_method(&mut cu_method, conn);

        self.to_method = cu_method;
        status
    }

    /// Checks the inputted Transaction Method by the user upon pressing Enter/Esc for various error.
    pub fn check_amount(&mut self, conn: &Connection) -> VerifyingOutput {
        let mut user_amount = self.amount.clone().to_lowercase();

        // 'b' represents the current balance of the original tx method
        if user_amount.contains('b') && !self.from_method.is_empty() {
            let all_methods = get_all_tx_methods(conn);

            // get all the method's final balance, loop through the balances and match the tx method name
            let last_balances = get_last_balances(conn, &all_methods);

            for x in 0..all_methods.len() {
                if all_methods[x] == self.from_method {
                    user_amount = user_amount.replace('b', &last_balances[x]);
                    break;
                }
            }
        } else if user_amount.contains('b') && self.from_method.is_empty() {
            return VerifyingOutput::NotAccepted(NAType::InvalidBValue);
        }

        let status = self.verify_amount(&mut user_amount);

        self.amount = user_amount;
        status
    }

    /// Checks the inputted Transaction Method by the user upon pressing Enter/Esc for various error.
    pub fn check_tx_type(&mut self) -> VerifyingOutput {
        let mut tx_type = self.tx_type.clone();

        let status = self.verify_tx_type(&mut tx_type);

        self.tx_type = tx_type;
        status
    }

    pub fn check_all_fields(&mut self) -> Option<SavingError> {
        if self.date.is_empty() {
            return Some(SavingError::EmptyDate);
        } else if self.from_method.is_empty() && self.tx_type != "Transfer" {
            return Some(SavingError::EmptyMethod);
        } else if self.amount.is_empty() {
            return Some(SavingError::EmptyAmount);
        } else if self.tx_type.is_empty() {
            return Some(SavingError::EmptyTxType);
        } else if self.tx_type == "Transfer" && self.from_method == self.to_method {
            return Some(SavingError::SameTxMethod);
        } else if self.tx_type == "Transfer"
            && (self.from_method.is_empty() || self.to_method.is_empty())
        {
            return Some(SavingError::EmptyMethod);
        }
        if self.tags.is_empty() {
            self.tags = "Unknown".to_string();
        }
        None
    }
}

impl TxData {}
