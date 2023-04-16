use crate::outputs::{NAType, SavingError, SteppingError, VerifyingOutput};
use crate::page_handler::{AddTxTab, TransferTab};
use crate::tx_handler::{add_tx, delete_tx};
use crate::utility::traits::DataVerifier;
use crate::utility::{get_all_tx_methods, get_last_balances};
use chrono::prelude::Local;
use chrono::{Duration, NaiveDate};
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
    current_index: usize,
}

impl DataVerifier for TxData {}

impl TxData {
    /// Creates an instance of the struct however the date field is
    /// edited with the current local date of the device.
    pub fn new() -> Self {
        let current_date = Local::now().to_string();
        let formatted_current_date = &current_date[0..10];
        TxData {
            date: formatted_current_date.to_string(),
            details: String::new(),
            from_method: String::new(),
            to_method: String::new(),
            amount: String::new(),
            tx_type: String::new(),
            tags: String::new(),
            tx_status: Vec::new(),
            editing_tx: false,
            id_num: 0,
            current_index: 0,
        }
    }

    pub fn new_transfer() -> Self {
        let current_date = Local::now().to_string();
        let formatted_current_date = &current_date[0..10];
        TxData {
            date: formatted_current_date.to_string(),
            details: String::new(),
            from_method: String::new(),
            to_method: String::new(),
            amount: String::new(),
            tx_type: "Transfer".to_string(),
            tags: String::new(),
            tx_status: Vec::new(),
            editing_tx: false,
            id_num: 0,
            current_index: 0,
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
            current_index: 0,
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

    fn get_tx_method(&self) -> String {
        if self.tx_type == "Transfer" {
            format!("{} to {}", self.from_method, self.to_method)
        } else {
            self.from_method.to_string()
        }
    }

    pub fn edit_date(&mut self, to_add: Option<char>) {
        if self.current_index > self.date.len() {
            self.current_index = self.date.len();
        } else {
            match to_add {
                Some(ch) => {
                    self.date.insert(self.current_index, ch);
                    self.current_index += 1
                }
                None => {
                    if !self.date.is_empty() && self.current_index != 0 {
                        self.date.remove(self.current_index - 1);
                        self.current_index -= 1;
                    }
                }
            }
        }
    }

    pub fn edit_details(&mut self, to_add: Option<char>) {
        if self.current_index > self.details.len() {
            self.current_index = self.details.len();
        } else {
            match to_add {
                Some(ch) => {
                    self.details.insert(self.current_index, ch);
                    self.current_index += 1
                }
                None => {
                    if !self.details.is_empty() && self.current_index != 0 {
                        self.details.remove(self.current_index - 1);
                        self.current_index -= 1;
                    }
                }
            }
        }
    }

    pub fn edit_from_method(&mut self, to_add: Option<char>) {
        if self.current_index > self.from_method.len() {
            self.current_index = self.from_method.len();
        } else {
            match to_add {
                Some(ch) => {
                    self.from_method.insert(self.current_index, ch);
                    self.current_index += 1
                }
                None => {
                    if !self.from_method.is_empty() && self.current_index != 0 {
                        self.from_method.remove(self.current_index - 1);
                        self.current_index -= 1;
                    }
                }
            }
        }
    }

    pub fn edit_to_method(&mut self, to_add: Option<char>) {
        if self.current_index > self.to_method.len() {
            self.current_index = self.to_method.len();
        } else {
            match to_add {
                Some(ch) => {
                    self.to_method.insert(self.current_index, ch);
                    self.current_index += 1
                }
                None => {
                    if !self.to_method.is_empty() && self.current_index != 0 {
                        self.to_method.remove(self.current_index - 1);
                        self.current_index -= 1;
                    }
                }
            }
        }
    }

    pub fn edit_amount(&mut self, to_add: Option<char>) {
        if self.current_index > self.amount.len() {
            self.current_index = self.amount.len();
        } else {
            match to_add {
                Some(ch) => {
                    self.amount.insert(self.current_index, ch);
                    self.current_index += 1
                }
                None => {
                    if !self.amount.is_empty() && self.current_index != 0 {
                        self.amount.remove(self.current_index - 1);
                        self.current_index -= 1;
                    }
                }
            }
        }
    }

    pub fn edit_tx_type(&mut self, to_add: Option<char>) {
        if self.current_index > self.tx_type.len() {
            self.current_index = self.tx_type.len();
        } else {
            match to_add {
                Some(ch) => {
                    self.tx_type.insert(self.current_index, ch);
                    self.current_index += 1
                }
                None => {
                    if !self.tx_type.is_empty() && self.current_index != 0 {
                        self.tx_type.remove(self.current_index - 1);
                        self.current_index -= 1;
                    }
                }
            }
        }
    }

    pub fn edit_tags(&mut self, to_add: Option<char>) {
        if self.current_index > self.tags.len() {
            self.current_index = self.tags.len();
        } else {
            match to_add {
                Some(ch) => {
                    self.tags.insert(self.current_index, ch);
                    self.current_index += 1
                }
                None => {
                    if !self.tags.is_empty() && self.current_index != 0 {
                        self.tags.remove(self.current_index - 1);
                        self.current_index -= 1;
                    }
                }
            }
        }
    }

    // TODO: Handle errors
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
        let mut current_method = self.from_method.clone();

        let status = self.verify_tx_method(&mut current_method, conn);

        self.from_method = current_method;
        status
    }

    /// Checks the inputted Transaction Method by the user upon pressing Enter/Esc for various error.
    pub fn check_to_method(&mut self, conn: &Connection) -> VerifyingOutput {
        let mut current_method = self.to_method.clone();

        let status = self.verify_tx_method(&mut current_method, conn);

        self.to_method = current_method;
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

    pub fn get_current_index(&self) -> usize {
        self.current_index
    }

    fn get_add_tx_data_len(&self, current_tab: &AddTxTab) -> usize {
        match current_tab {
            AddTxTab::Date => self.date.len(),
            AddTxTab::Details => self.details.len(),
            AddTxTab::TxMethod => self.from_method.len(),
            AddTxTab::Amount => self.amount.len(),
            AddTxTab::TxType => self.tx_type.len(),
            AddTxTab::Tags => self.tags.len(),
            AddTxTab::Nothing => 0,
        }
    }

    fn get_transfer_data_len(&self, current_tab: &TransferTab) -> usize {
        match current_tab {
            TransferTab::Date => self.date.len(),
            TransferTab::Details => self.details.len(),
            TransferTab::From => self.from_method.len(),
            TransferTab::To => self.to_method.len(),
            TransferTab::Amount => self.amount.len(),
            TransferTab::Tags => self.tags.len(),
            TransferTab::Nothing => 0,
        }
    }

    pub fn add_tx_move_index_left(&mut self, current_tab: &AddTxTab) {
        let data_len = self.get_add_tx_data_len(current_tab);

        if self.current_index > data_len {
            self.current_index = data_len
        } else if self.current_index > 0 {
            self.current_index -= 1
        }
    }

    pub fn add_tx_move_index_right(&mut self, current_tab: &AddTxTab) {
        let data_len = self.get_add_tx_data_len(current_tab);

        if self.current_index > data_len {
            self.current_index = data_len
        } else if data_len > self.current_index {
            self.current_index += 1
        }
    }

    pub fn transfer_move_index_left(&mut self, current_tab: &TransferTab) {
        let data_len = self.get_transfer_data_len(current_tab);

        if self.current_index > data_len {
            self.current_index = data_len
        } else if self.current_index > 0 {
            self.current_index -= 1
        }
    }

    pub fn transfer_move_index_right(&mut self, current_tab: &TransferTab) {
        let data_len = self.get_transfer_data_len(current_tab);

        if self.current_index > data_len {
            self.current_index = data_len
        } else if data_len > self.current_index {
            self.current_index += 1
        }
    }

    pub fn add_tx_go_current_index(&mut self, current_tab: &AddTxTab) {
        self.current_index = self.get_add_tx_data_len(current_tab)
    }

    pub fn transfer_go_current_index(&mut self, current_tab: &TransferTab) {
        self.current_index = self.get_transfer_data_len(current_tab)
    }

    pub fn do_date_up(&mut self) -> Result<(), SteppingError> {
        let status = self.check_date();
        let data_len = self.get_add_tx_data_len(&AddTxTab::Date);
        if self.current_index > data_len {
            self.current_index = data_len
        }

        match status {
            VerifyingOutput::Accepted(_) => {
                // TODO date here
                let final_date = NaiveDate::parse_from_str("2025-12-31", "%Y-%m-%d").unwrap();
                let mut current_date = NaiveDate::parse_from_str(&self.date, "%Y-%m-%d").unwrap();
                if current_date != final_date {
                    current_date += Duration::days(1);
                    self.date = current_date.to_string();
                    self.add_tx_go_current_index(&AddTxTab::Date);
                }
            }
            VerifyingOutput::NotAccepted(_) => return Err(SteppingError::InvalidDate),
            VerifyingOutput::Nothing(_) => {
                self.date = String::from("2022-01-01");
                self.add_tx_go_current_index(&AddTxTab::Date);
            }
        }

        Ok(())
    }

    pub fn do_date_down(&mut self) -> Result<(), SteppingError> {
        let status = self.check_date();
        let data_len = self.get_add_tx_data_len(&AddTxTab::Date);
        if self.current_index > data_len {
            self.current_index = data_len
        }

        match status {
            VerifyingOutput::Accepted(_) => {
                let final_date = NaiveDate::parse_from_str("2022-01-01", "%Y-%m-%d").unwrap();
                let mut current_date = NaiveDate::parse_from_str(&self.date, "%Y-%m-%d").unwrap();
                if current_date != final_date {
                    current_date -= Duration::days(1);
                    self.date = current_date.to_string();
                    self.add_tx_go_current_index(&AddTxTab::Date);
                }
            }
            VerifyingOutput::NotAccepted(_) => return Err(SteppingError::InvalidDate),
            VerifyingOutput::Nothing(_) => {
                self.date = String::from("2022-01-01");
                self.add_tx_go_current_index(&AddTxTab::Date);
            }
        }

        Ok(())
    }

    pub fn do_from_method_up(&mut self, conn: &Connection) -> Result<(), SteppingError> {
        let all_methods = get_all_tx_methods(conn);

        let status = self.check_from_method(conn);
        let data_len = self.get_add_tx_data_len(&AddTxTab::TxMethod);
        if self.current_index > data_len {
            self.current_index = data_len
        }

        match status {
            VerifyingOutput::Accepted(_) => {
                let current_method_index = all_methods
                    .iter()
                    .position(|e| e == &self.from_method)
                    .unwrap();
                let next_method_index = (current_method_index + 1) % all_methods.len();
                self.from_method = String::from(&all_methods[next_method_index]);
                self.add_tx_go_current_index(&AddTxTab::TxMethod);
            }
            VerifyingOutput::NotAccepted(_) => return Err(SteppingError::InvalidTxMethod),
            VerifyingOutput::Nothing(_) => {
                self.from_method = String::from(&all_methods[0]);
                self.add_tx_go_current_index(&AddTxTab::TxMethod);
            }
        }
        Ok(())
    }

    pub fn do_from_method_down(&mut self, conn: &Connection) -> Result<(), SteppingError> {
        let all_methods = get_all_tx_methods(conn);

        let status = self.check_from_method(conn);
        let data_len = self.get_add_tx_data_len(&AddTxTab::TxMethod);
        if self.current_index > data_len {
            self.current_index = data_len
        }

        match status {
            VerifyingOutput::Accepted(_) => {
                let current_method_index = all_methods
                    .iter()
                    .position(|e| e == &self.from_method)
                    .unwrap();
                let next_method_index = if current_method_index == 0 {
                    all_methods.len() - 1
                } else {
                    (current_method_index - 1) % all_methods.len()
                };
                self.from_method = String::from(&all_methods[next_method_index]);
                self.add_tx_go_current_index(&AddTxTab::TxMethod);
            }
            VerifyingOutput::NotAccepted(_) => return Err(SteppingError::InvalidTxMethod),
            VerifyingOutput::Nothing(_) => {
                self.from_method = String::from(&all_methods[0]);
                self.add_tx_go_current_index(&AddTxTab::TxMethod);
            }
        }

        Ok(())
    }

    pub fn do_to_method_up(&mut self) -> Result<(), SteppingError> {
        Ok(())
    }

    pub fn do_to_method_down(&mut self) -> Result<(), SteppingError> {
        Ok(())
    }

    pub fn do_tx_type_up(&mut self) -> Result<(), SteppingError> {
        Ok(())
    }

    pub fn do_tx_type_down(&mut self) -> Result<(), SteppingError> {
        Ok(())
    }

    pub fn do_amount_up(&mut self, conn: &Connection) -> Result<(), SteppingError> {
        let status = self.check_amount(conn);
        let data_len = self.get_add_tx_data_len(&AddTxTab::Amount);
        if self.current_index > data_len {
            self.current_index = data_len
        }

        if self.amount == "0.00" || self.amount == "" {
            self.amount = "1.00".to_string()
        } else {
            match status {
                VerifyingOutput::Accepted(_) => {
                    let mut current_amount: f64 = self.amount.parse().unwrap();

                    if 1000000000.00 != current_amount {
                        current_amount += 1.0;
                        self.amount = format!("{current_amount:.2}");
                    }
                }
                VerifyingOutput::NotAccepted(_) => return Err(SteppingError::InvalidAmount),
                _ => {}
            }
        }
        Ok(())
    }

    pub fn do_amount_down(&mut self, conn: &Connection) -> Result<(), SteppingError> {
        let status = self.check_amount(conn);

        let data_len = self.get_add_tx_data_len(&AddTxTab::Amount);
        if self.current_index > data_len {
            self.current_index = data_len
        }

        if self.amount != "0.00" {
            match status {
                VerifyingOutput::Accepted(_) => {
                    let mut current_amount: f64 = self.amount.parse().unwrap();

                    if 0.0 != current_amount {
                        current_amount -= 1.0;
                        self.amount = format!("{current_amount:.2}")
                    }
                }
                VerifyingOutput::NotAccepted(_) => return Err(SteppingError::InvalidAmount),
                _ => {}
            }
        }
        Ok(())
    }

    pub fn do_tags_up(&mut self) -> Result<(), SteppingError> {
        Ok(())
    }

    pub fn do_tags_down(&mut self) -> Result<(), SteppingError> {
        Ok(())
    }
}
