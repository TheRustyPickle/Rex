use chrono::prelude::{Local};
use rusqlite::Connection;
use crate::sub_func::{add_new_tx};
use std::error::Error;

pub struct AddTxData {
    date: String,
    details: String,
    tx_method: String,
    amount: String,
    tx_type: String,
    pub tx_status: Vec<String>,
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
            tx_status: Vec::new(),
        }
    }

    pub fn get_all_texts(&self) -> Vec<&str> {
        vec![&self.date, &self.details, &self.tx_method, &self.amount, &self.tx_type]
    }

    //TODO emit some kind of status to place on placement field ex check date format, amount

    pub fn edit_date(&mut self, text: char, pop_last: bool){
        match pop_last {
            true => {
                if self.date.len() > 0 {
                    self.date.pop().unwrap();
                }
            },
            false => self.date = format!("{}{text}", self.date),
        }
    }

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
            Ok(_) => println!("Success"),
            Err(e) => println!("Error happened {}", e),
        }
        "done".to_string()
    }

    pub fn add_tx_status(&mut self, data: &str) {
        if self.tx_status.len() == 20 {
            self.tx_status.remove(0);
        }
        self.tx_status.push(data.to_string());
    }

    pub fn check_date(&self) -> Result<String, Box<dyn Error>> {
        let user_date = &self.date;

        if user_date.len() == 0 {
            return Ok("Date: Nothing to check".to_string());
        }
        
        let splitted = user_date.split("-");
        let data = splitted.collect::<Vec<&str>>();
        let int_year: u32 = data[0].parse()?;
        let int_month: u32 = data[1].parse()?;
        let int_day: u32 = data[2].parse()?;

        if user_date.len() == 0 {
            return Ok("Date: Nothing to check".to_string());
        }

        if data[0].len() != 4 {
            return Ok("Date: Year Length Not Acceptable. Example Date: 2022-05-01".to_string())
        }

        if data[1].len() != 2 {
            return Ok("Date: Month Length Not Acceptable. Example Date: 2022-05-01".to_string())
        }

        if data[2].len() != 2 {
            return Ok("Date: Day Length Not Acceptable. Example Date: 2022-05-01".to_string())
        }

        if int_year < 2022 || int_year > 2025 {
            return Ok("Date: Year must be between 2022-2025".to_string())
        }

        if int_month < 1 || int_month > 12 {
            return Ok("Date: Month must be between 01-12".to_string())
        }

        if int_day < 1 || int_day > 31 {
            return Ok("Date: Day must be between 01-31".to_string())
        }

        Ok("Date: Date Accepted".to_string())
    }
}