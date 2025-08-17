use chrono::naive::NaiveDate;
use rusqlite::Connection;
use std::cmp::Ordering;
use std::collections::HashSet;

use crate::outputs::{AType, NAType, VerifyingOutput};
use crate::page_handler::DateType;
use crate::utility::{get_all_tags, get_all_tx_methods, get_best_match};

pub trait DataVerifier {
    /// Checks if:
    ///
    /// - The inputted year is between 2022 to 2037.
    /// - The inputted month is between 01 to 12.
    /// - The inputted date is between 01 to 31.
    /// - The inputted date is empty.
    /// - Contains any extra spaces.
    /// - The date actually exists.
    /// - Removes any extra spaces and non-numeric characters.
    /// - Ensures proper char length for each part of the date.
    ///
    /// Finally, tries to correct the date if it was not accepted by
    /// adding 0 if the beginning if the length is smaller than necessary
    /// or restores to the smallest or the largest date if date is beyond the
    /// accepted value.
    fn verify_date(&self, user_date: &mut String, date_type: &DateType) -> VerifyingOutput {
        // Cancel other verification if there is no text
        if user_date.is_empty() {
            return VerifyingOutput::Nothing(AType::Date);
        }
        *user_date = user_date
            .chars()
            .filter(|c| c.is_numeric() || *c == '-')
            .collect();

        // We will be splitting them into 3 parts to verify each part of the date
        // 0 = year
        // 1 = month
        // 2 = day
        let split_date = user_date
            .split('-')
            .map(ToString::to_string)
            .collect::<Vec<String>>();

        // If one part of the date is missing/extra, return unknown date
        match date_type {
            DateType::Exact => {
                if split_date.len() != 3 {
                    *user_date = "2022-01-01".to_string();
                    return VerifyingOutput::NotAccepted(NAType::InvalidDate);
                }
            }
            DateType::Monthly => {
                if split_date.len() != 2 {
                    *user_date = "2022-01".to_string();
                    return VerifyingOutput::NotAccepted(NAType::InvalidDate);
                }
            }
            DateType::Yearly => {
                if split_date.len() != 1 {
                    *user_date = "2022".to_string();
                    return VerifyingOutput::NotAccepted(NAType::InvalidDate);
                }
            }
        }

        // Year is required for each date type so no need for option
        let (int_year, int_month, int_day): (u16, Option<u16>, Option<u16>) = match date_type {
            DateType::Exact => {
                let Ok(year) = split_date[0].parse() else {
                    return VerifyingOutput::NotAccepted(NAType::ParsingError(AType::Date));
                };
                let month = match split_date[1].parse() {
                    Ok(v) => Some(v),
                    Err(_) => {
                        return VerifyingOutput::NotAccepted(NAType::ParsingError(AType::Date));
                    }
                };
                let day = match split_date[2].parse() {
                    Ok(v) => Some(v),
                    Err(_) => {
                        return VerifyingOutput::NotAccepted(NAType::ParsingError(AType::Date));
                    }
                };
                (year, month, day)
            }
            DateType::Monthly => {
                let Ok(year) = split_date[0].parse() else {
                    return VerifyingOutput::NotAccepted(NAType::ParsingError(AType::Date));
                };
                let month = match split_date[1].parse() {
                    Ok(v) => Some(v),
                    Err(_) => {
                        return VerifyingOutput::NotAccepted(NAType::ParsingError(AType::Date));
                    }
                };
                (year, month, None)
            }
            DateType::Yearly => {
                let Ok(year) = split_date[0].parse() else {
                    return VerifyingOutput::NotAccepted(NAType::ParsingError(AType::Date));
                };
                (year, None, None)
            }
        };

        // Checks if the year part length is 4. If not 4, turn the year to 2022 + the other character entered by the user
        // and return the new date
        if split_date[0].len() != 4 {
            match split_date[0].len().cmp(&4) {
                Ordering::Less => match date_type {
                    DateType::Exact => {
                        *user_date = format!("2022-{}-{}", split_date[1], split_date[2]);
                    }
                    DateType::Monthly => *user_date = format!("2022-{}", split_date[1]),
                    DateType::Yearly => *user_date = "2022".to_string(),
                },
                Ordering::Greater => match date_type {
                    DateType::Exact => {
                        *user_date = format!(
                            "{}-{}-{}",
                            &split_date[0][..4],
                            split_date[1],
                            split_date[2]
                        );
                    }
                    DateType::Monthly => {
                        *user_date = format!("{}-{}", &split_date[0][..4], split_date[1]);
                    }
                    DateType::Yearly => *user_date = split_date[0][..4].to_string(),
                },
                Ordering::Equal => {}
            }
            return VerifyingOutput::NotAccepted(NAType::InvalidYear);
        }
        // Checks if the month part length is 2. If not 2, turn the month to 0 + whatever month was entered + the other character entered by the user
        // and return the new date
        match date_type {
            DateType::Exact => {
                if split_date[1].len() != 2 {
                    let unwrapped_month = int_month.unwrap();
                    if unwrapped_month < 10 {
                        *user_date =
                            format!("{}-0{unwrapped_month}-{}", split_date[0], split_date[2]);
                    } else if unwrapped_month > 12 {
                        *user_date = format!("{}-12-{}", split_date[0], split_date[2]);
                    }

                    return VerifyingOutput::NotAccepted(NAType::InvalidMonth);
                }
            }
            DateType::Monthly => {
                let unwrapped_month = int_month.unwrap();
                if split_date[1].len() != 2 {
                    if unwrapped_month < 10 {
                        *user_date = format!("{}-0{unwrapped_month}", split_date[0]);
                    } else if unwrapped_month > 12 {
                        *user_date = format!("{}-12", split_date[0]);
                    }

                    return VerifyingOutput::NotAccepted(NAType::InvalidMonth);
                }
            }
            DateType::Yearly => {}
        }

        // Checks if the day part length is 2. If not 2, turn the day to 0 + whatever day was entered + the other character entered by the user
        // and return the new date
        if let DateType::Exact = date_type {
            let unwrapped_day = int_day.unwrap();
            if split_date[2].len() != 2 {
                if unwrapped_day < 10 {
                    *user_date = format!("{}-{}-0{unwrapped_day}", split_date[0], split_date[1]);
                } else if unwrapped_day > 31 {
                    *user_date = format!("{}-{}-31", split_date[0], split_date[1]);
                }

                return VerifyingOutput::NotAccepted(NAType::InvalidDay);
            }
        }

        // Checks if the year value is between 2022 and 2037
        if !(2022..=2037).contains(&int_year) {
            if int_year < 2022 {
                match date_type {
                    DateType::Exact => {
                        *user_date = format!("2022-{}-{}", split_date[1], split_date[2]);
                    }
                    DateType::Monthly => *user_date = format!("2022-{}", split_date[1]),
                    DateType::Yearly => *user_date = "2022".to_string(),
                }
            } else if int_year > 2037 {
                match date_type {
                    DateType::Exact => {
                        *user_date = format!("2037-{}-{}", split_date[1], split_date[2]);
                    }
                    DateType::Monthly => *user_date = format!("2037-{}", split_date[1]),
                    DateType::Yearly => *user_date = "2037".to_string(),
                }
            }

            return VerifyingOutput::NotAccepted(NAType::YearTooBig);
        }

        // Checks if the month value is between 1 and 12
        match date_type {
            DateType::Exact => {
                let unwrapped_month = int_month.unwrap();
                if !(1..=12).contains(&unwrapped_month) {
                    if unwrapped_month < 1 {
                        *user_date = format!("{}-01-{}", split_date[0], split_date[2]);
                    } else if unwrapped_month > 12 {
                        *user_date = format!("{}-12-{}", split_date[0], split_date[2]);
                    }

                    return VerifyingOutput::NotAccepted(NAType::MonthTooBig);
                }
            }
            DateType::Monthly => {
                let unwrapped_month = int_month.unwrap();
                if !(1..=12).contains(&unwrapped_month) {
                    if unwrapped_month < 1 {
                        *user_date = format!("{}-01", split_date[0]);
                    } else if unwrapped_month > 12 {
                        *user_date = format!("{}-12", split_date[0]);
                    }

                    return VerifyingOutput::NotAccepted(NAType::MonthTooBig);
                }
            }
            DateType::Yearly => {}
        }

        // Checks if the day value is between 1 and 31
        if let DateType::Exact = date_type {
            let unwrapped_day = int_day.unwrap();
            if !(1..=31).contains(&unwrapped_day) {
                if unwrapped_day < 1 {
                    *user_date = format!("{}-{}-01", split_date[0], split_date[1]);
                } else if unwrapped_day > 31 {
                    *user_date = format!("{}-{}-31", split_date[0], split_date[1]);
                }

                return VerifyingOutput::NotAccepted(NAType::DayTooBig);
            }
        }

        // We will check if the date actually exists otherwise return error
        // Some months have more or less days than 31 so the date needs to be validated
        if let DateType::Exact = date_type {
            let naive_date = NaiveDate::parse_from_str(user_date, "%Y-%m-%d");
            match naive_date {
                Ok(_) => {}
                Err(_) => return VerifyingOutput::NotAccepted(NAType::NonExistingDate),
            }
        }

        VerifyingOutput::Accepted(AType::Date)
    }

    /// Checks if:
    ///
    /// - Amount is empty
    /// - Amount is zero or below
    /// - Amount text contains a calculation symbol
    /// - contains any extra spaces
    /// - removes any extra spaces and non-numeric characters
    ///
    /// If the value is not float, tries to make it float ending with double zero
    fn verify_amount(&self, user_amount: &mut String) -> VerifyingOutput {
        // Cancel all verification if the amount is empty
        if user_amount.is_empty() {
            return VerifyingOutput::Nothing(AType::Amount);
        }

        let calc_symbols = vec!['*', '/', '+', '-'];

        *user_amount = user_amount
            .chars()
            .filter(|c| c.is_numeric() || *c == '.' || calc_symbols.contains(c))
            .collect();

        // Already checked if the initial amount is empty.
        // If it becomes empty after the filtering was done, there no number inside so return error
        if user_amount.is_empty() {
            return VerifyingOutput::NotAccepted(NAType::ParsingError(AType::Amount));
        }

        // Check if any of the symbols are present
        if calc_symbols.iter().any(|s| user_amount.contains(*s)) {
            // How it works:
            // The calc_symbol intentionally starts with * and / so these calculations are done first.
            // Start a main loop which will only run for the amount of times anyone of them from calc_symbols is present.
            // Loop over the symbols and check if the symbol is present in the string
            // find the index of where the symbol is then take the number values from both side of the symbol.
            // Example: 1+5*10. We start with *, we initially, we will work with 5*10.
            // Isolate the numbers => do the calculation => replace the part of the string we are working with, with the result which is 50
            // result: 1+50 => break the symbol checking loop and continue the main loop again so we start working with 1+50.

            // Get the amount of time the symbols were found in the amount string
            let count = user_amount
                .chars()
                .filter(|c| calc_symbols.contains(c))
                .count();

            // Remove all spaces for easier indexing
            let mut working_value = user_amount.to_owned();

            for _i in 0..count {
                for symbol in &calc_symbols {
                    if let Some(location) = working_value.find(*symbol) {
                        // If a symbol is found, we want to store the values to its side to these variables.
                        // Example: 1+5 first_value = 1 last_value = 5
                        let mut first_value = String::new();
                        let mut last_value = String::new();

                        // Skip to symbol location + 1 index value and start taking chars from here until the end
                        // of the string or until another cal symbol is encountered
                        for char in working_value.chars().skip(location + 1) {
                            if calc_symbols.contains(&char) {
                                break;
                            }
                            last_value.push(char);
                        }

                        // Do the same thing as before but this time, reverse the string
                        for char in working_value
                            .chars()
                            .rev()
                            .skip(working_value.len() - location)
                        {
                            if calc_symbols.contains(&char) {
                                break;
                            }
                            first_value.push(char);
                        }
                        // Un-reverse the string
                        first_value = first_value.chars().rev().collect();

                        // If either of them is empty, the one that is not empty is the value we want to use for using in replacement
                        let final_value = if first_value.is_empty() || last_value.is_empty() {
                            if first_value.is_empty() {
                                last_value.to_string()
                            } else {
                                first_value.to_string()
                            }
                        } else {
                            // If both value is intact, do the calculation and the result is for replacement
                            let first_num: f64 = match first_value.parse() {
                                Ok(v) => v,
                                Err(_) => {
                                    return VerifyingOutput::NotAccepted(NAType::ParsingError(
                                        AType::Amount,
                                    ));
                                }
                            };

                            let last_num: f64 = match last_value.parse() {
                                Ok(v) => v,
                                Err(_) => {
                                    return VerifyingOutput::NotAccepted(NAType::ParsingError(
                                        AType::Amount,
                                    ));
                                }
                            };

                            match *symbol {
                                '*' => format!("{:.2}", (first_num * last_num)),
                                '/' => format!("{:.2}", (first_num / last_num)),
                                '+' => format!("{:.2}", (first_num + last_num)),
                                '-' => format!("{:.2}", (first_num - last_num)),
                                _ => String::new(),
                            }
                        };

                        // Example: 1+5*10
                        // if everything goes alright, first_value is 5, last_value is 10 and the symbol is *
                        // replace 5*10 with the earlier result we got which is 50. Continue with 1+50 in the next loop
                        working_value = working_value
                            .replace(&format!("{first_value}{symbol}{last_value}"), &final_value);

                        break;
                    }
                }
            }
            *user_amount = working_value;
        }

        // If dot is present but nothing after that, add 2 zero
        // if no dot, add dot + 2 zero
        if user_amount.contains('.') {
            let state = user_amount.split('.').collect::<Vec<&str>>();
            if state[1].is_empty() {
                *user_amount += "00";
            }
        } else {
            *user_amount = format!("{user_amount}.00");
        }

        let float_amount: f64 = match user_amount.parse() {
            Ok(v) => v,
            Err(_) => return VerifyingOutput::NotAccepted(NAType::ParsingError(AType::Amount)),
        };

        if float_amount <= 0.0 {
            *user_amount = format!("{:.2}", (float_amount - (float_amount * 2.0)));
            return VerifyingOutput::NotAccepted(NAType::AmountBelowZero);
        }

        // Checks if there is 2 number after the dot else add zero/s
        if user_amount.contains('.') {
            let split_amount = user_amount.split('.').collect::<Vec<&str>>();

            match split_amount[1].len().cmp(&2) {
                Ordering::Less => *user_amount = format!("{user_amount}0"),
                Ordering::Greater => {
                    *user_amount = format!("{}.{}", split_amount[0], &split_amount[1][..2]);
                }
                Ordering::Equal => (),
            }
        }

        // We can safely split now as previously we just added a dot + 2 numbers with the amount
        // and create the final value for the amount
        let split_amount = user_amount.split('.').collect::<Vec<&str>>();

        // limit max character to 10
        if split_amount[0].len() > 10 {
            *user_amount = format!("{}.{}", &split_amount[0][..10], split_amount[1]);
        }

        VerifyingOutput::Accepted(AType::Amount)
    }

    /// Checks if:
    ///
    /// - The Transaction method exists on the database.
    /// - The Transaction method is empty
    /// - contains any extra spaces
    ///
    /// If the Transaction is not found, matches each character with the available
    /// Transaction Methods and corrects to the best matching one.
    fn verify_tx_method(&self, user_method: &mut String, conn: &Connection) -> VerifyingOutput {
        // Get all currently added tx methods
        let all_tx_methods = get_all_tx_methods(conn);

        *user_method = user_method.trim().to_string();

        // Cancel all verification if the text is empty
        if user_method.is_empty() {
            return VerifyingOutput::Nothing(AType::TxMethod);
        }

        for method in &all_tx_methods {
            if method.to_lowercase() == user_method.to_lowercase() {
                *user_method = method.to_string();
                return VerifyingOutput::Accepted(AType::TxMethod);
            }
        }

        let best_match = get_best_match(user_method, &all_tx_methods);

        *user_method = best_match;
        VerifyingOutput::NotAccepted(NAType::InvalidTxMethod)
    }

    /// Checks if:
    ///
    /// - The transaction method starts with E, I, or T
    ///
    /// Auto expands E to Expense, I to Income and T to transfer.
    fn verify_tx_type(&self, user_type: &mut String) -> VerifyingOutput {
        *user_type = user_type.replace(' ', "");

        if user_type.is_empty() {
            return VerifyingOutput::Nothing(AType::TxType);
        }
        if user_type.to_lowercase().starts_with('e') {
            *user_type = "Expense".to_string();
            VerifyingOutput::Accepted(AType::TxType)
        } else if user_type.to_lowercase().starts_with('i') {
            *user_type = "Income".to_string();
            VerifyingOutput::Accepted(AType::TxType)
        } else if user_type.to_lowercase().starts_with('t') {
            *user_type = "Transfer".to_string();
            VerifyingOutput::Accepted(AType::TxType)
        } else {
            *user_type = String::new();
            VerifyingOutput::NotAccepted(NAType::InvalidTxType)
        }
    }

    /// Checks if:
    ///
    /// - All tags inserted is unique and is properly separated by commas
    fn verify_tags(&self, user_tag: &mut String) {
        let mut split_tags = user_tag.split(',').map(str::trim).collect::<Vec<&str>>();
        split_tags.retain(|s| !s.is_empty());

        let mut seen = HashSet::new();
        let mut unique = Vec::new();

        for item in split_tags {
            if seen.insert(item) {
                unique.push(item);
            }
        }

        *user_tag = unique.join(", ");
    }

    /// Checks if:
    ///
    /// - All tags inserted is unique and is properly separated by commas
    /// - There is no non-existing tags
    fn verify_tags_forced(&self, user_tag: &mut String, conn: &Connection) -> VerifyingOutput {
        if user_tag.is_empty() {
            return VerifyingOutput::Nothing(AType::Tags);
        }
        let all_tags = get_all_tags(conn);
        let mut split_tags = user_tag.split(',').map(str::trim).collect::<Vec<&str>>();
        split_tags.retain(|s| !s.is_empty());

        let mut seen = HashSet::new();
        let mut unique = Vec::new();

        for item in split_tags {
            if seen.insert(item) {
                unique.push(item);
            }
        }

        let old_tags_len = unique.len();

        unique.retain(|&tag| all_tags.contains(&tag.to_owned()));

        let new_tags_len = unique.len();

        *user_tag = unique.join(", ");

        if old_tags_len == new_tags_len {
            VerifyingOutput::Accepted(AType::Tags)
        } else {
            VerifyingOutput::NotAccepted(NAType::NonExistingTag)
        }
    }
}
