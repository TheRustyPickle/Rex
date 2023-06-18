use crate::outputs::{NAType, SteppingError, VerifyingOutput};
use chrono::{Duration, NaiveDate};

pub trait FieldStepper {
    fn step_date_up(
        &self,
        verify_status: VerifyingOutput,
        user_date: &mut String,
    ) -> Result<(), SteppingError> {
        match verify_status {
            VerifyingOutput::Accepted(_) => {
                let final_date = NaiveDate::parse_from_str("2037-12-31", "%Y-%m-%d").unwrap();
                let mut current_date = NaiveDate::parse_from_str(user_date, "%Y-%m-%d").unwrap();
                if current_date != final_date {
                    current_date += Duration::days(1);
                    *user_date = current_date.to_string();
                }
            }
            VerifyingOutput::NotAccepted(_) => {
                return Err(SteppingError::InvalidDate);
            }
            // * Nothing -> Empty box.
            // If nothing and pressed Up, make it the first possible date
            VerifyingOutput::Nothing(_) => {
                *user_date = String::from("2022-01-01");
            }
        }

        Ok(())
    }

    fn step_date_down(
        &self,
        verify_status: VerifyingOutput,
        user_date: &mut String,
    ) -> Result<(), SteppingError> {
        match verify_status {
            VerifyingOutput::Accepted(_) => {
                let final_date = NaiveDate::parse_from_str("2022-01-01", "%Y-%m-%d").unwrap();
                let mut current_date = NaiveDate::parse_from_str(user_date, "%Y-%m-%d").unwrap();
                if current_date != final_date {
                    current_date -= Duration::days(1);
                    *user_date = current_date.to_string();
                }
            }
            VerifyingOutput::NotAccepted(_) => {
                return Err(SteppingError::InvalidDate);
            }
            // * Nothing -> Empty box.
            // If nothing and pressed Up, make it the first possible date
            VerifyingOutput::Nothing(_) => {
                *user_date = String::from("2022-01-01");
            }
        }
        Ok(())
    }

    fn step_tx_method_up(
        &self,
        verify_status: VerifyingOutput,
        user_method: &mut String,
        all_methods: Vec<String>,
    ) -> Result<(), SteppingError> {
        match verify_status {
            VerifyingOutput::Accepted(_) => {
                let current_method_index =
                    all_methods.iter().position(|e| e == user_method).unwrap();

                // if reached final index, start from beginning
                let next_method_index = (current_method_index + 1) % all_methods.len();
                *user_method = String::from(&all_methods[next_method_index]);
            }
            VerifyingOutput::NotAccepted(_) => {
                return Err(SteppingError::InvalidTxMethod);
            }
            // * Nothing -> Empty box.
            // If nothing and pressed Up, make it the first possible method
            VerifyingOutput::Nothing(_) => {
                *user_method = String::from(&all_methods[0]);
            }
        }

        Ok(())
    }

    fn step_tx_method_down(
        &self,
        verify_status: VerifyingOutput,
        user_method: &mut String,
        all_methods: Vec<String>,
    ) -> Result<(), SteppingError> {
        match verify_status {
            VerifyingOutput::Accepted(_) => {
                let current_method_index =
                    all_methods.iter().position(|e| e == user_method).unwrap();

                // if reached final index, start from beginning
                let next_method_index = if current_method_index == 0 {
                    all_methods.len() - 1
                } else {
                    (current_method_index - 1) % all_methods.len()
                };
                *user_method = String::from(&all_methods[next_method_index]);
            }
            VerifyingOutput::NotAccepted(_) => {
                return Err(SteppingError::InvalidTxMethod);
            }
            // * Nothing -> Empty box.
            // If nothing and pressed Up, make it the first possible method
            VerifyingOutput::Nothing(_) => {
                *user_method = String::from(&all_methods[0]);
            }
        }
        Ok(())
    }

    fn step_amount_up(
        &self,
        verify_status: VerifyingOutput,
        user_amount: &mut String,
    ) -> Result<(), SteppingError> {
        match verify_status {
            VerifyingOutput::Accepted(_) => {
                let mut current_amount: f64 = user_amount.parse().unwrap();

                if 9999999999.99 > current_amount + 1.0 {
                    current_amount += 1.0;
                    *user_amount = format!("{current_amount:.2}");
                }
            }
            VerifyingOutput::NotAccepted(err_type) => match err_type {
                // if value went below 0, make it 1
                NAType::AmountBelowZero => {
                    *user_amount = String::from("1.00");
                }
                _ => {
                    return Err(SteppingError::InvalidAmount);
                }
            },
            VerifyingOutput::Nothing(_) => *user_amount = "1.00".to_string(),
        }
        Ok(())
    }

    fn step_amount_down(
        &self,
        verify_status: VerifyingOutput,
        user_amount: &mut String,
    ) -> Result<(), SteppingError> {
        match verify_status {
            VerifyingOutput::Accepted(_) => {
                let mut current_amount: f64 = user_amount.parse().unwrap();

                if (current_amount - 1.0) >= 0.00 {
                    current_amount -= 1.0;
                    *user_amount = format!("{current_amount:.2}");
                }
            }
            VerifyingOutput::NotAccepted(err_type) => match err_type {
                NAType::AmountBelowZero => {}
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
        verify_status: VerifyingOutput,
        user_type: &mut String,
    ) -> Result<(), SteppingError> {
        // * there's only 2 possible values of tx type
        if user_type.is_empty() {
            *user_type = "Income".to_string()
        } else if user_type == "Income" {
            *user_type = "Expense".to_string()
        } else if user_type == "Expense" {
            *user_type = "Income".to_string()
        }

        if let VerifyingOutput::NotAccepted(_) = verify_status {
            return Err(SteppingError::InvalidTxType);
        }

        Ok(())
    }

    fn step_tags_up(
        &self,
        user_tag: &mut String,
        all_tags: Vec<String>,
        autofill: &str,
    ) -> Result<(), SteppingError> {
        // if current tag is empty but up is pressed,
        // select the first possible tag if available
        if user_tag.is_empty() {
            if !all_tags.is_empty() {
                *user_tag = String::from(&all_tags[0]);
            } else {
                return Err(SteppingError::InvalidTags);
            }
        } else {
            // tags are separated by comma. Collect all the tags
            let mut current_tags = user_tag
                .split(',')
                .map(|s| s.trim().to_string())
                .collect::<Vec<String>>();

            // tag1, tag2, tag3
            // in this case, only work with tag3, keep the rest as it is
            let last_tag = current_tags.pop().unwrap();

            // check if the working tag exists inside all tag list
            if !all_tags
                .iter()
                .any(|tag| tag.to_lowercase() == last_tag.to_lowercase())
            {
                // tag3, tag2,
                // if kept like this with extra comma, the last_tag would be empty. In this case
                // select the first tag available in the list or just join the first two tag with , + space
                if last_tag.is_empty() {
                    if !all_tags.is_empty() {
                        current_tags.push(all_tags[0].to_owned());
                        *user_tag = current_tags.join(", ");
                    } else {
                        *user_tag = current_tags.join(", ");
                    }
                } else {
                    // as the tag didn't match with any existing tags accept the autofill suggestion
                    current_tags.push(autofill.to_owned());

                    *user_tag = current_tags.join(", ");
                    return Err(SteppingError::InvalidTags);
                }
            } else if let Some(index) = all_tags
                .iter()
                .position(|tag| tag.to_lowercase() == last_tag.to_lowercase())
            {
                // if the tag matches with something, get the index, select the next one.
                // start from beginning if reached at the end -> Join
                let next_index = (index + 1) % all_tags.len();
                current_tags.push(all_tags[next_index].to_owned());
                *user_tag = current_tags.join(", ");
            }
        }
        Ok(())
    }

    fn step_tags_down(
        &self,
        user_tag: &mut String,
        all_tags: Vec<String>,
        autofill: &str,
    ) -> Result<(), SteppingError> {
        // if current tag is empty but up is pressed,
        // select the first possible tag if available
        if user_tag.is_empty() {
            if !all_tags.is_empty() {
                *user_tag = String::from(&all_tags[0]);
            } else {
                return Err(SteppingError::InvalidTags);
            }
        } else {
            // tags are separated by comma. Collect all the tags
            let mut current_tags = user_tag
                .split(',')
                .map(|s| s.trim().to_string())
                .collect::<Vec<String>>();

            // tag1, tag2, tag3
            // in this case, only work with tag3, keep the rest as it is
            let last_tag = current_tags.pop().unwrap();

            // check if the working tag exists inside all tag list
            if !all_tags
                .iter()
                .any(|tag| tag.to_lowercase() == last_tag.to_lowercase())
            {
                // tag3, tag2,
                // if kept like this with extra comma, the last_tag would be empty. In this case
                // select the first tag available in the list or just join the first two tag with , + space
                if last_tag.is_empty() {
                    if !all_tags.is_empty() {
                        current_tags.push(all_tags[0].to_owned());
                        *user_tag = current_tags.join(", ");
                    } else {
                        *user_tag = current_tags.join(", ");
                    }
                } else {
                    // as the tag didn't match with any existing tags accept the autofill suggestion
                    current_tags.push(autofill.to_owned());

                    *user_tag = current_tags.join(", ");
                    return Err(SteppingError::InvalidTags);
                }
            } else if let Some(index) = all_tags
                .iter()
                .position(|tag| tag.to_lowercase() == last_tag.to_lowercase())
            {
                // if the tag matches with something, get the index, select the next one.
                // start from beginning if reached at the end -> Join
                let next_index = if index == 0 {
                    all_tags.len() - 1
                } else {
                    (index - 1) % all_tags.len()
                };
                current_tags.push(all_tags[next_index].to_owned());
                *user_tag = current_tags.join(", ");
            }
        }
        Ok(())
    }
}
