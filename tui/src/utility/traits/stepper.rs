use app::ui_helper::DateType;
use chrono::{Duration, NaiveDate};
use rusqlite::Connection;

use crate::outputs::{NAType, StepType, SteppingError, VerifyingOutput};
use crate::utility::traits::DataVerifier;
use crate::utility::{get_all_tags, get_all_tx_methods};

pub trait FieldStepper: DataVerifier {
    fn step_date(
        &self,
        user_date: &mut String,
        step_type: StepType,
        date_type: &DateType,
    ) -> Result<(), SteppingError> {
        let verify_status = self.verify_date(user_date, date_type);

        match verify_status {
            VerifyingOutput::Accepted(_) => match date_type {
                DateType::Exact => {
                    let mut current_date =
                        NaiveDate::parse_from_str(user_date, "%Y-%m-%d").unwrap();
                    match step_type {
                        StepType::StepUp => {
                            let final_date =
                                NaiveDate::parse_from_str("2037-12-31", "%Y-%m-%d").unwrap();
                            if current_date != final_date {
                                current_date += Duration::days(1);
                            }
                        }
                        StepType::StepDown => {
                            let final_date =
                                NaiveDate::parse_from_str("2022-01-01", "%Y-%m-%d").unwrap();
                            if current_date != final_date {
                                current_date -= Duration::days(1);
                            }
                        }
                    }
                    *user_date = current_date.to_string();
                }
                DateType::Monthly => {
                    let split_date = user_date
                        .split('-')
                        .map(|s| s.parse().unwrap())
                        .collect::<Vec<u16>>();

                    let mut month = split_date[1];
                    let mut year = split_date[0];

                    match step_type {
                        StepType::StepUp => {
                            if month < 12 {
                                month += 1;
                            } else if month == 12 && year != 2037 {
                                month = 1;
                                year += 1;
                            }
                        }
                        StepType::StepDown => {
                            if month > 1 {
                                month -= 1;
                            } else if month == 1 && year != 2022 {
                                month = 12;
                                year -= 1;
                            }
                        }
                    }
                    *user_date = format!("{year}-{month:02}");
                }
                DateType::Yearly => {
                    let mut int_year: u16 = user_date.parse().unwrap();
                    match step_type {
                        StepType::StepUp => {
                            if int_year != 2037 {
                                int_year += 1;
                            }
                        }
                        StepType::StepDown => {
                            if int_year != 2022 {
                                int_year -= 1;
                            }
                        }
                    }

                    *user_date = int_year.to_string();
                }
            },
            VerifyingOutput::NotAccepted(_) => {
                return Err(SteppingError::InvalidDate);
            }
            // Nothing -> Empty box.
            // If nothing and pressed Up, make it the first possible date
            VerifyingOutput::Nothing(_) => match date_type {
                DateType::Exact => *user_date = String::from("2022-01-01"),
                DateType::Monthly => *user_date = String::from("2022-01"),
                DateType::Yearly => *user_date = String::from("2022"),
            },
        }

        Ok(())
    }

    fn step_tx_method(
        &self,
        user_method: &mut String,
        step_type: StepType,
        conn: &Connection,
    ) -> Result<(), SteppingError> {
        let all_methods = get_all_tx_methods(conn);
        let verify_status: VerifyingOutput = self.verify_tx_method(user_method, conn);

        match verify_status {
            VerifyingOutput::Accepted(_) => {
                let current_method_index =
                    all_methods.iter().position(|e| e == user_method).unwrap();

                let next_method_index = match step_type {
                    StepType::StepUp => (current_method_index + 1) % all_methods.len(),
                    StepType::StepDown => {
                        if current_method_index == 0 {
                            all_methods.len() - 1
                        } else {
                            (current_method_index - 1) % all_methods.len()
                        }
                    }
                };
                *user_method = String::from(&all_methods[next_method_index]);
            }
            VerifyingOutput::NotAccepted(_) => {
                return Err(SteppingError::InvalidTxMethod);
            }
            // Nothing -> Empty box.
            // If nothing and pressed Up, make it the first possible method
            VerifyingOutput::Nothing(_) => {
                *user_method = String::from(&all_methods[0]);
            }
        }

        Ok(())
    }

    fn step_amount(
        &self,
        user_amount: &mut String,
        step_type: StepType,
    ) -> Result<(), SteppingError> {
        let verify_status: VerifyingOutput = self.verify_amount(user_amount);

        match verify_status {
            VerifyingOutput::Accepted(_) => {
                let mut current_amount: f64 = user_amount.parse().unwrap();

                match step_type {
                    StepType::StepUp => {
                        if 9_999_999_999.99 >= current_amount + 1.0 {
                            current_amount += 1.0;
                        }
                    }
                    StepType::StepDown => {
                        if (current_amount - 1.0) >= 0.00 {
                            current_amount -= 1.0;
                        }
                    }
                }

                *user_amount = format!("{current_amount:.2}");
            }
            VerifyingOutput::NotAccepted(err_type) => match err_type {
                // if value went below 0, make it 1
                NAType::AmountBelowZero => {
                    if let StepType::StepUp = step_type {
                        *user_amount = String::from("1.00");
                    }
                }
                _ => {
                    return Err(SteppingError::InvalidAmount);
                }
            },
            VerifyingOutput::Nothing(_) => *user_amount = "1.00".to_string(),
        }
        Ok(())
    }

    fn step_tx_type(
        &self,
        user_type: &mut String,
        step_type: StepType,
    ) -> Result<(), SteppingError> {
        let verify_status: VerifyingOutput = self.verify_tx_type(user_type);
        let tx_types = ["Income", "Expense", "Transfer"];

        if !user_type.is_empty() {
            let mut current_index: usize =
                match user_type.chars().next().unwrap().to_ascii_lowercase() {
                    'e' => 1,
                    't' => 2,
                    // 'I' is 0
                    _ => 0,
                };

            match step_type {
                StepType::StepUp => current_index = (current_index + 1) % tx_types.len(),
                StepType::StepDown => {
                    current_index = (current_index + tx_types.len() - 1) % tx_types.len();
                }
            }
            *user_type = tx_types[current_index].to_string();
        }

        match verify_status {
            VerifyingOutput::NotAccepted(_) => Err(SteppingError::InvalidTxType),

            VerifyingOutput::Nothing(_) => {
                *user_type = "Income".to_string();
                Ok(())
            }
            VerifyingOutput::Accepted(_) => Ok(()),
        }
    }

    fn step_tags(
        &self,
        user_tag: &mut String,
        autofill: &str,
        step_type: StepType,
        conn: &Connection,
    ) -> Result<(), SteppingError> {
        let all_tags = get_all_tags(conn);

        // If current tag is empty
        // select the first possible tag if available
        if user_tag.is_empty() {
            if all_tags.is_empty() {
                return Err(SteppingError::InvalidTags);
            }
            *user_tag = String::from(&all_tags[0]);
            return Ok(());
        }

        // Tags are separated by comma. Collect all the tags
        let mut current_tags = user_tag
            .split(',')
            .map(|s| s.trim().to_string())
            .collect::<Vec<String>>();

        // Tag1, tag2, tag3
        // in this case, only work with tag3, keep the rest as it is
        let last_tag = current_tags.pop().unwrap();

        // Check if the working tag exists inside all tag list
        if !all_tags
            .iter()
            .any(|tag| tag.to_lowercase() == last_tag.to_lowercase())
        {
            // Tag3, tag2,
            // if kept like this with extra comma, the last_tag would be empty. In this case
            // select the first tag available in the list or just join the first two tag with , + space
            if last_tag.is_empty() {
                if all_tags.is_empty() {
                    *user_tag = current_tags.join(", ");
                } else {
                    current_tags.push(all_tags[0].clone());
                    *user_tag = current_tags.join(", ");
                }
            } else {
                // As the tag didn't match with any existing tags accept the auto fill suggestion
                current_tags.push(autofill.to_owned());

                *user_tag = current_tags.join(", ");
                return Err(SteppingError::InvalidTags);
            }
        } else if let Some(index) = all_tags
            .iter()
            .position(|tag| tag.to_lowercase() == last_tag.to_lowercase())
        {
            let next_index = match step_type {
                StepType::StepUp => (index + 1) % all_tags.len(),

                StepType::StepDown => {
                    if index == 0 {
                        all_tags.len() - 1
                    } else {
                        (index - 1) % all_tags.len()
                    }
                }
            };
            // If the tag matches with something, get the index, select the next one.
            // Start from beginning if reached at the end -> Join
            current_tags.push(all_tags[next_index].clone());
            *user_tag = current_tags.join(", ");
        }

        Ok(())
    }
}
