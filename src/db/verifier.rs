use crate::db::get_all_tx_methods;
use chrono::naive::NaiveDate;
use rusqlite::Connection;
use std::error::Error;

/// A trait for verifying date, tx_method, tx_type and amount fields
/// from the ui. Turned into a trait for reusability
pub trait StatusChecker {
    /// Checks if:
    ///
    /// - the date length is 10 characters
    /// - the inputted year is between 2022 to 2025
    /// - the inputted month is between 01 to 12
    /// - the inputted date is between 01 to 31
    /// - the inputted date is empty
    /// - contains any extra spaces
    ///
    /// Finally, tries to correct the date if it was not accepted by
    /// adding 0 if the beginning if the length is smaller than necessary
    /// or restores to the smallest or the largest date if date is beyond the
    /// accepted value.

    fn verify_date(&self, user_date: &mut String) -> Result<String, Box<dyn Error>> {
        // cancel other verification if there is no text
        if user_date.is_empty() {
            return Ok("Date: Nothing to check".to_string());
        }

        // we will be splitting them into 3 parts to verify each part of the date
        let splitted = user_date.split('-');
        let split = splitted.collect::<Vec<&str>>();

        // to prevent any extra spaces passing, recreate the vec again
        let mut data = vec![];

        for i in split {
            data.push(i.trim().to_string());
        }

        // if one part of the date is missing, return unknown date
        if data.len() != 3 {
            *user_date = "2022-01-01".to_string();
            return Ok("Date: Unknown date".to_string());
        }
        // rewrite the original date in case of extra spaces
        *user_date = format!("{}-{}-{}", data[0], data[1], data[2]);

        let int_year: u32 = data[0].trim().parse()?;
        let int_month: u32 = data[1].trim().parse()?;
        let int_day: u32 = data[2].trim().parse()?;

        // checks if the year part length is 4. If not 4, turn the year to 2022 + the other character entered by the user
        // and return the new date
        if data[0].len() != 4 {
            if data[0].len() < 4 {
                *user_date = format!("2022-{}-{}", data[1], data[2]);
            } else if data[0].len() > 4 {
                *user_date = format!("{}-{}-{}", &data[0][..4], data[1], data[2]);
            }
            return Ok("Date: Year length not acceptable. Example Date: 2022-05-01".to_string());

        // checks if the month part length is 2. If not 2, turn the month to 0 + whatever month was entered + the other character entered by the user
        // and return the new date
        } else if data[1].len() != 2 {
            if int_month < 10 {
                *user_date = format!("{}-0{int_month}-{}", data[0], data[2]);
            } else if int_month > 12 {
                *user_date = format!("{}-12-{}", data[0], data[2]);
            }

            return Ok("Date: Month length not acceptable. Example Date: 2022-05-01".to_string());

        // checks if the day part length is 2. If not 2, turn the day to 0 + whatever day was entered + the other character entered by the user
        // and return the new date
        } else if data[2].len() != 2 {
            if int_day < 10 {
                *user_date = format!("{}-{}-0{int_day}", data[0], data[1]);
            } else if int_day > 31 {
                *user_date = format!("{}-{}-31", data[0], data[1]);
            }

            return Ok("Date: Day length not acceptable. Example Date: 2022-05-01".to_string());

        // checks if the year value is between 2022 and 2025
        } else if !(2022..=2025).contains(&int_year) {
            if int_year < 2022 {
                *user_date = format!("2022-{}-{}", data[1], data[2]);
            } else if int_year > 2025 {
                *user_date = format!("2025-{}-{}", data[1], data[2]);
            }

            return Ok("Date: Year must be between 2022-2025".to_string());

        // checks if the month value is between 1 and 12
        } else if !(1..=12).contains(&int_month) {
            if int_month < 1 {
                *user_date = format!("{}-01-{}", data[0], data[2]);
            } else if int_month > 12 {
                *user_date = format!("{}-12-{}", data[0], data[2]);
            }

            return Ok("Date: Month must be between 01-12".to_string());

        // checks if the day value is between 1 and 31
        } else if !(1..=31).contains(&int_day) {
            if int_day < 1 {
                *user_date = format!("{}-{}-01", data[0], data[1]);
            } else if int_day > 31 {
                *user_date = format!("{}-{}-31", data[0], data[1]);
            }

            return Ok("Date: Day must be between 01-31".to_string());
        }

        // We will check if the date actually exists otherwise
        let naive_date = NaiveDate::parse_from_str(user_date, "%Y-%m-%d");
        match naive_date {
            Ok(_) => {}
            Err(e) => {
                return Ok(format!(
                    "Date: Date not acceptable and possibly non-existing. Error: {e}"
                ))
            }
        }

        Ok("Date: Date Accepted".to_string())
    }

    /// Checks if:
    ///
    /// - Amount is empty
    /// - Amount is zero or below
    /// - Amount text contains a calculation symbol
    /// - contains any extra spaces
    ///
    /// if the value is not float, tries to make it float ending with double zero

    fn verify_amount(&self, amount: &mut String) -> Result<String, Box<dyn Error>> {
        // cancel all verification if the amount is empty
        if amount.is_empty() {
            return Ok("Amount: Nothing to check".to_string());
        }

        let calc_symbols = vec!["*", "+", "-", "/"];

        for i in calc_symbols {
            if amount.contains(i) {
                let data = amount.split(i).collect::<Vec<&str>>();
                if !data[0].trim().is_empty() && !data[1].trim().is_empty() {
                    let first_amount: f64 = data[0].trim().parse()?;
                    let second_amount: f64 = data[1].trim().parse()?;

                    match i {
                        "*" => *amount = format!("{:.2}", (first_amount * second_amount)),
                        "/" => *amount = format!("{:.2}", (first_amount / second_amount)),
                        "+" => *amount = format!("{:.2}", (first_amount + second_amount)),
                        "-" => *amount = format!("{:.2}", (first_amount - second_amount)),
                        _ => {}
                    }
                    break;
                } else {
                    *amount = amount.replace(i, "").trim().to_string();
                }
            }
        }

        if amount.contains('.') {
            let state = amount.split('.');
            let splitted = state.collect::<Vec<&str>>();
            if splitted[1].is_empty() {
                *amount += "00"
            }
        }

        // If the amount contains non-number character, make it fail
        let int_amount: f64 = amount.trim().parse()?;

        if int_amount <= 0.0 {
            *amount = format!("{:.2}", (int_amount - (int_amount * 2.0)));
            return Ok("Amount: Value must be bigger than zero".to_string());
        }

        // checks if there double zero after the dot else add double zero
        if amount.contains('.') {
            let splitted = amount.split('.');
            let splitted_data = splitted.collect::<Vec<&str>>();

            if splitted_data[1].len() < 2 {
                *amount = format!("{amount}0");
            } else if splitted_data[1].len() > 2 {
                *amount = format!("{}.{}", splitted_data[0], &splitted_data[1][..2]);
            }
        } else {
            *amount = format!("{amount}.00");
        }

        // we can safely split now as previously we just added a dot + 2 numbers with the amount
        // and create the final value for the amount
        let splitted = amount.split('.');
        let splitted_data = splitted.collect::<Vec<&str>>();

        // limit max character to 10
        if splitted_data[0].len() > 10 {
            *amount = format!(
                "{}.{}",
                &splitted_data[0].trim()[..10],
                splitted_data[1].trim()
            );
        }

        Ok("Amount: Amount Accepted".to_string())
    }

    /// Checks if:
    ///
    /// - The Transaction method exists on the database.
    /// - The Transaction method is empty
    /// - contains any extra spaces
    /// if the Transaction is not found, matches each character with the available
    /// Transaction Methods and corrects to the best matching one.

    fn verify_tx_method(
        &self,
        cu_method: &mut String,
        conn: &Connection,
    ) -> Result<String, Box<dyn Error>> {
        // get all currently added tx methods
        let all_tx_methods = get_all_tx_methods(conn);

        *cu_method = cu_method.trim().to_string();

        // cancel all verification if the text is empty
        if cu_method.is_empty() {
            return Ok("TX Method: Nothing to check".to_string());
        }

        // loops through all tx methods and matches each character
        // of the tx method with the current inputted text. Based on matches
        // selects the best matching one if text is not any exact match.
        if all_tx_methods.contains(cu_method) {
            return Ok("TX Method: Transaction Method Accepted".to_string());
        } else {
            let mut current_match = all_tx_methods[0].clone();
            let mut current_chance = 0;

            for i in all_tx_methods {
                let mut total_match = 0;
                let method = i.trim().to_string();
                for i in method.chars() {
                    if cu_method
                        .to_lowercase()
                        .contains(&i.to_string().to_lowercase().to_string())
                    {
                        total_match += 1;
                    }
                }
                let chance = (100 * total_match) / method.len();

                if chance > current_chance {
                    current_match = method;
                    current_chance = chance;
                }
            }
            *cu_method = current_match;
        }

        Ok("TX Method: Transaction Method not found".to_string())
    }

    /// Checks if:
    ///
    /// - The transaction method starts with E or I
    ///
    /// Auto expands E to Expense and I to Income.
    fn verify_tx_type(&self, tx_type: &mut String) -> Result<String, Box<dyn Error>> {
        *tx_type = tx_type.trim().to_string();

        if tx_type.is_empty() {
            return Ok("TX Type: Nothing to check".to_string());
        }
        if tx_type.to_lowercase().trim().starts_with('e') {
            *tx_type = "Expense".to_string();
            Ok("TX Type: Transaction Type Accepted".to_string())
        } else if tx_type.to_lowercase().trim().starts_with('i') {
            *tx_type = "Income".to_string();
            Ok("TX Type: Transaction Type Accepted".to_string())
        } else {
            Ok("TX Type: Transaction Type not acceptable. Values: Expense/Income/E/I".to_string())
        }
    }
}
