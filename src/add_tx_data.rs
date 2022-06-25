use crate::sub_func::{add_new_tx, get_all_tx_methods};
use chrono::prelude::Local;
use rusqlite::Connection;
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
        vec![
            &self.date,
            &self.details,
            &self.tx_method,
            &self.amount,
            &self.tx_type,
        ]
    }

    //TODO emit some kind of status to place on placement field ex check date format, amount

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

    pub fn edit_tx_method(&mut self, text: char, pop_last: bool) {
        match pop_last {
            true => {
                if self.tx_method.len() > 0 {
                    self.tx_method.pop().unwrap();
                }
            }
            false => self.tx_method = format!("{}{text}", self.tx_method),
        }
    }

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

    pub fn edit_tx_type(&mut self, text: char, pop_last: bool) {
        match pop_last {
            true => {
                if self.tx_type.len() > 0 {
                    self.tx_type.pop().unwrap();
                }
            }
            false => self.tx_type = format!("{}{text}", self.tx_type),
        }
    }

    pub fn add_tx(&mut self, conn: &Connection) -> String {
        let status = add_new_tx(
            conn,
            &self.date,
            &self.details,
            &self.tx_method,
            &self.amount,
            &self.tx_type,
        );
        match status {
            Ok(_) => {}
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
    // TODO check details to make sure it is not empty
    pub fn check_date(&mut self) -> Result<String, Box<dyn Error>> {
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
            if data[0].len() < 4 {
                let new_date = format!("2022-{}-{}", data[1], data[2]);
                self.date = new_date;
            } else if data[0].len() > 4 {
                let new_date = format!("{}-{}-{}", &data[0][..4], data[1], data[2]);
                self.date = new_date;
            }
            return Ok("Date: Year length not acceptable. Example Date: 2022-05-01".to_string());
        } else if data[1].len() != 2 {
            if int_month < 10 {
                let new_date = format!("{}-0{int_month}-{}", data[0], data[2]);
                self.date = new_date;
            } else if int_month > 12 {
                let new_date = format!("{}-12-{}", data[0], data[2]);
                self.date = new_date;
            }

            return Ok("Date: Month length not acceptable. Example Date: 2022-05-01".to_string());
        } else if data[2].len() != 2 {
            if int_day < 10 {
                let new_date = format!("{}-{}-0{int_day}", data[0], data[1]);
                self.date = new_date;
            } else if int_day > 31 {
                let new_date = format!("{}-{}-31", data[0], data[1]);
                self.date = new_date;
            }

            return Ok("Date: Day length not acceptable. Example Date: 2022-05-01".to_string());
        } else if int_year < 2022 || int_year > 2025 {
            if int_year < 2022 {
                let new_date = format!("2022-{}-{}", data[1], data[2]);
                self.date = new_date;
            } else if int_year > 2025 {
                let new_date = format!("2025-{}-{}", data[1], data[2]);
                self.date = new_date;
            }

            return Ok("Date: Year must be between 2022-2025".to_string());
        } else if int_month < 1 || int_month > 12 {
            if int_month < 1 {
                let new_date = format!("{}-01-{}", data[0], data[2]);
                self.date = new_date;
            } else if int_month > 12 {
                let new_date = format!("{}-12-{}", data[0], data[2]);
                self.date = new_date;
            }

            return Ok("Date: Month must be between 01-12".to_string());
        } else if int_day < 1 || int_day > 31 {
            if int_day < 1 {
                let new_date = format!("{}-{}-01", data[0], data[1]);
                self.date = new_date;
            } else if int_day > 31 {
                let new_date = format!("{}-{}-31", data[0], data[1]);
                self.date = new_date;
            }

            return Ok("Date: Day must be between 01-31".to_string());
        }

        Ok("Date: Date Accepted".to_string())
    }

    pub fn check_tx_method(&mut self, conn: &Connection) -> String {
        let all_tx_methods = get_all_tx_methods(conn);
        let current_text = &self.tx_method;
        if current_text.len() == 0 {
            return "TX Method: Nothing to check".to_string();
        }

        if all_tx_methods.contains(&current_text) {
            return "Tx Method: Transaction Method Accepted".to_string();
        } else {
            let mut current_match = all_tx_methods[0].clone();
            let mut current_chance = 0;

            for method in all_tx_methods {
                let mut total_match = 0;
                for i in method.chars() {
                    if current_text.contains(i) {
                        total_match += 1;
                    }
                }
                let chance = (100 * total_match) / method.len();

                if chance > current_chance {
                    current_match = method;
                    current_chance = chance;
                }
            }
            self.tx_method = current_match;

            return "TX Method: Transaction Method not found".to_string();
        }
    }

    pub fn check_amount(&mut self) -> Result<String, Box<dyn Error>> {
        if self.amount.len() == 0 {
            return Ok("Amount: Nothing to check".to_string());
        }
        let mut data = self.amount.clone();

        if data.contains(".") {
            let state = data.split(".");
            let splitted = state.collect::<Vec<&str>>();
            if splitted[1].len() == 0 {
                data += "00"
            }
        }

        // If the amount contains non-number character, make it fail
        let int_amount: f32 = self.amount.parse()?;
        
        if int_amount == 0.0 {
            return Ok("Amount: Value must be bigger than zero".to_string());
        }

        if data.contains(".") {
            let splitted = data.split(".");
            let splitted_data = splitted.collect::<Vec<&str>>();

            if splitted_data[1].len() < 2 {
                data = format!("{data}0");
            } else if splitted_data[1].len() > 2 {
                data = format!("{}.{}", splitted_data[0], &splitted_data[1][..2]);
            }
        } else {
            data = format!("{data}.00");
        }

        let splitted = data.split(".");
        let splitted_data = splitted.collect::<Vec<&str>>();

        if splitted_data[0].len() > 10 {
            data = format!("{}.{}", &splitted_data[0][..10], splitted_data[1]);
        }

        self.amount = data.to_string();

        Ok("Amount: Amount Accepted".to_string())
    }

    pub fn check_tx_type(&mut self) -> String {
        if self.tx_type.len() == 0 {
            return "TX Type: Nothing to check".to_string();
        }
        if self.tx_type.to_lowercase().starts_with("e") {
            self.tx_type = "Expense".to_string();
            return "TX Type: Transaction Type Accepted".to_string();
        } else if self.tx_type.to_lowercase().starts_with("i") {
            self.tx_type = "Income".to_string();
            return "TX Type: Transaction Type Accepted".to_string();
        } else {
            return "TX Type: Transaction Type not acceptable. Values: Expense/Income/E/I"
                .to_string();
        }
    }
}
