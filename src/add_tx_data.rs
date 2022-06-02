use chrono::prelude::{Local};
use rusqlite::Connection;
use crate::sub_func::add_new_tx;
pub struct AddTxData {
    date: String,
    details: String,
    tx_method: String,
    amount: String,
    tx_type: String,
}

impl AddTxData {
    pub fn new() -> Self {
        let cu_date = Local::today().to_string();
        let formatted_cu_date = &cu_date[0..10];
        AddTxData {
            date: formatted_cu_date.to_string(),
            details: "".to_string(),
            tx_method: "".to_string(),
            amount: "".to_string(),
            tx_type: "".to_string(),
        }
    }

    pub fn get_all_texts(&self) -> Vec<&str> {
        vec![&self.date, &self.details, &self.tx_method, &self.amount, &self.tx_type]
    }

    //TODO emit some kind of status to place on placement field ex check date format, amount

    pub fn edit_details(&mut self, text: char, pop_last: bool) {
        match pop_last {
            true => {
                if self.details.len() > 0 {
                    self.details.pop().unwrap();
                }
            },
            false => self.details = format!("{}{text}", self.details),
        }
    }

    pub fn edit_tx_method(&mut self, text: char, pop_last: bool) {
        match pop_last {
            true => {
                if self.tx_method.len() > 0 {
                    self.tx_method.pop().unwrap();
                }
            },
            false => self.tx_method = format!("{}{text}", self.tx_method),
        }
    }

    pub fn edit_amount(&mut self, text: char, pop_last: bool) {
        match pop_last {
            true => {
                if self.amount.len() > 0 {
                    self.amount.pop().unwrap();
                }
            },
            false => self.amount = format!("{}{text}", self.amount),
        }
    }

    pub fn edit_tx_type(&mut self, text: char, pop_last: bool) {
        match pop_last {
            true => {
                if self.tx_type.len() > 0 {
                    self.tx_type.pop().unwrap();
                }
            },
            false => self.tx_type = format!("{}{text}", self.tx_type),
        }
    }

    pub fn add_tx(&mut self, conn: &Connection) -> String {
        let status = add_new_tx(conn, &self.date, &self.details, &self.tx_method, &self.amount, &self.tx_type);
        match status {
            Ok(_) => println!("Transaction Added Successfully"),
            Err(e) => println!("Error Adding Transaction. Error: {}", e),
        }
        "done".to_string()
    }
}