use crate::outputs::{AType, NAType, VerifyingOutput};
use crate::tx_handler::TxData;
use crate::utility::get_all_tx_methods;
use chrono::naive::NaiveDate;
use rusqlite::Connection;

/// Checks if:
///
/// - the date length is 10 characters
/// - the inputted year is between 2022 to 2025
/// - the inputted month is between 01 to 12
/// - the inputted date is between 01 to 31
/// - the inputted date is empty
/// - contains any extra spaces
/// - the date actually exists
///
/// Finally, tries to correct the date if it was not accepted by
/// adding 0 if the beginning if the length is smaller than necessary
/// or restores to the smallest or the largest date if date is beyond the
/// accepted value.

impl TxData {
    pub fn verify_date(&self, user_date: &mut String) -> VerifyingOutput {
        // cancel other verification if there is no text
        if user_date.is_empty() {
            return VerifyingOutput::Nothing(AType::Date);
        }
        *user_date = user_date.replace(' ', "");

        // we will be splitting them into 3 parts to verify each part of the date
        let splitted_date = user_date
            .split('-')
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        // if one part of the date is missing, return unknown date
        if splitted_date.len() != 3 {
            *user_date = "2022-01-01".to_string();
            return VerifyingOutput::NotAccepted(NAType::InvalidDate);
        }

        let int_year: u32 = match splitted_date[0].parse() {
            Ok(v) => v,
            Err(_) => return VerifyingOutput::NotAccepted(NAType::ParsingError(AType::Date)),
        };

        let int_month: u32 = match splitted_date[1].parse() {
            Ok(v) => v,
            Err(_) => return VerifyingOutput::NotAccepted(NAType::ParsingError(AType::Date)),
        };

        let int_day: u32 = match splitted_date[2].parse() {
            Ok(v) => v,
            Err(_) => return VerifyingOutput::NotAccepted(NAType::ParsingError(AType::Date)),
        };

        // checks if the year part length is 4. If not 4, turn the year to 2022 + the other character entered by the user
        // and return the new date
        if splitted_date[0].len() != 4 {
            if splitted_date[0].len() < 4 {
                *user_date = format!("2022-{}-{}", splitted_date[1], splitted_date[2]);
            } else if splitted_date[0].len() > 4 {
                *user_date = format!(
                    "{}-{}-{}",
                    &splitted_date[0][..4],
                    splitted_date[1],
                    splitted_date[2]
                );
            }
            return VerifyingOutput::NotAccepted(NAType::InvalidYear);

        // checks if the month part length is 2. If not 2, turn the month to 0 + whatever month was entered + the other character entered by the user
        // and return the new date
        } else if splitted_date[1].len() != 2 {
            if int_month < 10 {
                *user_date = format!("{}-0{int_month}-{}", splitted_date[0], splitted_date[2]);
            } else if int_month > 12 {
                *user_date = format!("{}-12-{}", splitted_date[0], splitted_date[2]);
            }

            return VerifyingOutput::NotAccepted(NAType::InvalidMonth);

        // checks if the day part length is 2. If not 2, turn the day to 0 + whatever day was entered + the other character entered by the user
        // and return the new date
        } else if splitted_date[2].len() != 2 {
            if int_day < 10 {
                *user_date = format!("{}-{}-0{int_day}", splitted_date[0], splitted_date[1]);
            } else if int_day > 31 {
                *user_date = format!("{}-{}-31", splitted_date[0], splitted_date[1]);
            }

            return VerifyingOutput::NotAccepted(NAType::InvalidDay);

        // checks if the year value is between 2022 and 2025
        // TODO year check
        } else if !(2022..=2025).contains(&int_year) {
            if int_year < 2022 {
                *user_date = format!("2022-{}-{}", splitted_date[1], splitted_date[2]);
            } else if int_year > 2025 {
                *user_date = format!("2025-{}-{}", splitted_date[1], splitted_date[2]);
            }

            return VerifyingOutput::NotAccepted(NAType::YearTooBig);

        // checks if the month value is between 1 and 12
        } else if !(1..=12).contains(&int_month) {
            if int_month < 1 {
                *user_date = format!("{}-01-{}", splitted_date[0], splitted_date[2]);
            } else if int_month > 12 {
                *user_date = format!("{}-12-{}", splitted_date[0], splitted_date[2]);
            }

            return VerifyingOutput::NotAccepted(NAType::MonthTooBig);

        // checks if the day value is between 1 and 31
        } else if !(1..=31).contains(&int_day) {
            if int_day < 1 {
                *user_date = format!("{}-{}-01", splitted_date[0], splitted_date[1]);
            } else if int_day > 31 {
                *user_date = format!("{}-{}-31", splitted_date[0], splitted_date[1]);
            }

            return VerifyingOutput::NotAccepted(NAType::DayTooBig);
        }

        // We will check if the date actually exists otherwise
        let naive_date = NaiveDate::parse_from_str(user_date, "%Y-%m-%d");
        match naive_date {
            Ok(_) => {}
            Err(_) => return VerifyingOutput::NotAccepted(NAType::NonExistingDate),
        }

        VerifyingOutput::Accepted(AType::Date)
    }

    /// Checks if:
    ///
    /// - Amount is empty
    /// - Amount is zero or below
    /// - Amount text contains a calculation symbol
    /// - contains any extra spaces
    ///
    /// if the value is not float, tries to make it float ending with double zero

    pub fn verify_amount(&self, amount: &mut String) -> VerifyingOutput {
        // cancel all verification if the amount is empty
        if amount.is_empty() {
            return VerifyingOutput::Accepted(AType::Amount);
        }

        *amount = amount.replace(' ', "");

        let calc_symbols = vec!["*", "/", "+", "-"];

        // check if any of the symbols are present
        if calc_symbols.iter().any(|s| amount.contains(s)) {
            // how it works:
            // the calc_symbol intentionally starts with * and / so these calculations are done first
            // start a main loop which will only run for the amount of times any one of them from calc_symbols is present
            // loop over the symbols and check if the symbol is present in the string
            // find the index of where the symbol is then take the number values from both side of the symbol
            // example: 1+5*10. We start with *, we initially, we will work with 5*10
            // isolate the numbers => do the calculation => replace the part of the string we are working with, with the result which is 50
            // result: 1+50 => break the symbol checking loop and continue the main loop again so we start working with 1+50.

            // get the amount of time the symbols were found in the amount string
            let count = amount
                .chars()
                .filter(|c| calc_symbols.contains(&c.to_string().as_str()))
                .count();

            // remove all spaces for easier indexing
            let mut working_value = amount.to_owned();

            for _i in 0..count {
                for symbol in &calc_symbols {
                    if let Some(location) = working_value.find(symbol) {
                        // if a symbol is found, we want to store the values to its side to these variables.
                        // example: 1+5 first_value = 1 last_value = 5
                        let mut first_value = String::new();
                        let mut last_value = String::new();

                        // skip to symbol location + 1 index value and start taking chars from here until the end
                        // of the string or until another cal symbol is encountered
                        for char in working_value.chars().skip(location + 1) {
                            if !calc_symbols.contains(&String::from(char).as_str()) {
                                last_value.push(char)
                            } else {
                                break;
                            }
                        }

                        // do the same thing as before but this time, reverse the string
                        for char in working_value
                            .chars()
                            .rev()
                            .skip(working_value.len() - location)
                        {
                            if !calc_symbols.contains(&String::from(char).as_str()) {
                                first_value.push(char)
                            } else {
                                break;
                            }
                        }
                        // un-reverse the string
                        first_value = first_value.chars().rev().collect();

                        // if either of them is empty, the one that is not empty is the value we want to use for using in replacement
                        let final_value = if first_value.is_empty() || last_value.is_empty() {
                            if first_value.is_empty() {
                                last_value.to_string()
                            } else {
                                first_value.to_string()
                            }
                        } else {
                            // if both value is intact, do the calculation and the result is for replacement
                            let first_num: f64 = match first_value.parse() {
                                Ok(v) => v,
                                Err(_) => {
                                    return VerifyingOutput::NotAccepted(NAType::ParsingError(
                                        AType::Amount,
                                    ))
                                }
                            };

                            let last_num: f64 = match last_value.parse() {
                                Ok(v) => v,
                                Err(_) => {
                                    return VerifyingOutput::NotAccepted(NAType::ParsingError(
                                        AType::Amount,
                                    ))
                                }
                            };

                            match *symbol {
                                "*" => format!("{:.2}", (first_num * last_num)),
                                "/" => format!("{:.2}", (first_num / last_num)),
                                "+" => format!("{:.2}", (first_num + last_num)),
                                "-" => format!("{:.2}", (first_num - last_num)),
                                _ => String::new(),
                            }
                        };

                        // example: 1+5*10
                        // if everything goes alright, first_value is 5, last_value is 10 and the symbol is *
                        // replace 5*10 with the earlier result we got which is 50. Continue with 1+50 in the next loop
                        working_value = working_value
                            .replace(&format!("{first_value}{symbol}{last_value}"), &final_value);

                        break;
                    }
                }
            }
            *amount = working_value;
        }

        // if dot is present but nothing after that, add 2 zero
        // if no dot, add dot + 2 zero
        if amount.contains('.') {
            let state = amount.split('.').collect::<Vec<&str>>();
            if state[1].is_empty() {
                *amount += "00"
            }
        } else {
            *amount = format!("{amount}.00");
        }

        let float_amount: f64 = match amount.parse() {
            Ok(v) => v,
            Err(_) => return VerifyingOutput::NotAccepted(NAType::ParsingError(AType::Amount)),
        };

        if float_amount <= 0.0 {
            *amount = format!("{:.2}", (float_amount - (float_amount * 2.0)));
            return VerifyingOutput::NotAccepted(NAType::AmountBelowZero);
        }

        // checks if there is 2 number after the dot else add zero/s
        if amount.contains('.') {
            let splitted = amount.split('.').collect::<Vec<&str>>();

            if splitted[1].len() < 2 {
                *amount = format!("{amount}0");
            } else if splitted[1].len() > 2 {
                *amount = format!("{}.{}", splitted[0], &splitted[1][..2]);
            }
        }

        // we can safely split now as previously we just added a dot + 2 numbers with the amount
        // and create the final value for the amount
        let splitted_amount = amount.split('.').collect::<Vec<&str>>();

        // limit max character to 10
        if splitted_amount[0].len() > 10 {
            *amount = format!("{}.{}", &splitted_amount[0][..10], splitted_amount[1]);
        }

        VerifyingOutput::Accepted(AType::Amount)
    }

    /// Checks if:
    ///
    /// - The Transaction method exists on the database.
    /// - The Transaction method is empty
    /// - contains any extra spaces
    /// if the Transaction is not found, matches each character with the available
    /// Transaction Methods and corrects to the best matching one.

    pub fn verify_tx_method(&self, cu_method: &mut String, conn: &Connection) -> VerifyingOutput {
        // get all currently added tx methods
        let all_tx_methods = get_all_tx_methods(conn);

        *cu_method = cu_method.trim().to_string();

        // cancel all verification if the text is empty
        if cu_method.is_empty() {
            return VerifyingOutput::Nothing(AType::TxMethod);
        }

        for method in &all_tx_methods {
            if method.to_lowercase() == cu_method.to_lowercase() {
                *cu_method = method.to_string();
                return VerifyingOutput::Accepted(AType::TxMethod);
            }
        }

        let mut matching_tx_methods: Vec<&str> = all_tx_methods
            .iter()
            .filter(|&s| s.to_lowercase().contains(&cu_method.to_lowercase()))
            .map(|s| s.as_str())
            .collect();

        if matching_tx_methods.is_empty() {
            matching_tx_methods.push(&all_tx_methods[0]);
        }

        if matching_tx_methods.len() > 1 {
            let mut best_match = matching_tx_methods[0];
            let mut best_score = rank_match(best_match, cu_method);

            for &match_str in &matching_tx_methods[1..] {
                let score = rank_match(match_str, cu_method);
                if score > best_score {
                    best_match = match_str;
                    best_score = score;
                }
            }
            *cu_method = best_match.to_string()
        } else {
            *cu_method = matching_tx_methods[0].to_string();
        }

        VerifyingOutput::NotAccepted(NAType::InvalidTxMethod)
    }

    /// Checks if:
    ///
    /// - The transaction method starts with E or I
    ///
    /// Auto expands E to Expense and I to Income.
    pub fn verify_tx_type(&self, tx_type: &mut String) -> VerifyingOutput {
        *tx_type = tx_type.replace(' ', "");

        if tx_type.is_empty() {
            return VerifyingOutput::Nothing(AType::TxType);
        }
        if tx_type.to_lowercase().starts_with('e') {
            *tx_type = "Expense".to_string();
            VerifyingOutput::Accepted(AType::TxType)
        } else if tx_type.to_lowercase().starts_with('i') {
            *tx_type = "Income".to_string();
            VerifyingOutput::Accepted(AType::TxType)
        } else {
            *tx_type = String::new();
            VerifyingOutput::NotAccepted(NAType::InvalidTxType)
        }
    }
}

fn rank_match(match_str: &str, search_str: &str) -> f64 {
    let match_count = match_str.to_lowercase().matches(search_str).count();
    let match_len = match_str.len() as f64;
    (match_count as f64) / match_len
}
