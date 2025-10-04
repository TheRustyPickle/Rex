use chrono::{Duration, Months, NaiveDate};
use db::ConnCache;

use crate::conn::MutDbConn;
use crate::ui_helper::{
    DateType, Field, Output, StepType, SteppingError, TX_TYPES, VerifierError, get_best_match,
};

pub struct Stepper<'a> {
    conn: MutDbConn<'a>,
}

impl<'a> Stepper<'a> {
    pub(crate) fn new(conn: MutDbConn<'a>) -> Self {
        Self { conn }
    }

    pub fn date(
        mut self,
        user_date: &mut String,
        step_type: StepType,
        date_type: DateType,
    ) -> Result<(), SteppingError> {
        let verify_status = self.conn.verify().date(user_date, date_type);

        match verify_status {
            Ok(data) => match data {
                Output::Nothing(_) => match date_type {
                    DateType::Exact => *user_date = String::from("2022-01-01"),
                    DateType::Monthly => *user_date = String::from("2022-01"),
                    DateType::Yearly => *user_date = String::from("2022"),
                },
                Output::Accepted(_) => match date_type {
                    DateType::Exact => {
                        let mut current_date =
                            NaiveDate::parse_from_str(user_date, "%Y-%m-%d").unwrap();
                        match step_type {
                            StepType::StepUp => {
                                current_date += Duration::days(1);
                            }
                            StepType::StepDown => {
                                current_date -= Duration::days(1);
                            }
                        }
                        *user_date = current_date.to_string();
                    }
                    DateType::Monthly => {
                        let split_date = user_date
                            .split('-')
                            .map(|s| s.parse().unwrap())
                            .collect::<Vec<u16>>();

                        let month = split_date[1];
                        let year = split_date[0];

                        let mut current_date =
                            NaiveDate::from_ymd_opt(year.into(), month.into(), 1).unwrap();

                        match step_type {
                            StepType::StepUp => {
                                current_date = current_date + Months::new(1);
                            }
                            StepType::StepDown => {
                                current_date = current_date - Months::new(1);
                            }
                        }
                        *user_date = current_date.format("%Y-%m").to_string();
                    }
                    DateType::Yearly => {
                        let mut int_year: u16 = user_date.parse().unwrap();
                        match step_type {
                            StepType::StepUp => {
                                int_year += 1;
                            }
                            StepType::StepDown => {
                                int_year -= 1;
                            }
                        }

                        *user_date = int_year.to_string();
                    }
                },
            },
            Err(_) => {
                return Err(SteppingError::InvalidDate);
            }
        }

        Ok(())
    }

    pub fn tx_method(
        mut self,
        user_method: &mut String,
        step_type: StepType,
    ) -> Result<(), SteppingError> {
        let verify_status = self.conn.verify().tx_method(user_method);

        let all_methods = self.conn.cache().get_methods();

        match verify_status {
            Ok(data) => match data {
                Output::Accepted(_) => {
                    let current_method_index = all_methods
                        .iter()
                        .position(|e| &e.name == user_method)
                        .unwrap();

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
                    *user_method = String::from(&all_methods.get(next_method_index).unwrap().name);
                }
                Output::Nothing(_) => {
                    *user_method = String::from(&all_methods.first().unwrap().name);
                }
            },
            Err(_) => {
                return Err(SteppingError::InvalidTxMethod);
            }
        }

        Ok(())
    }

    pub fn amount(
        mut self,
        user_amount: &mut String,
        step_type: StepType,
    ) -> Result<(), SteppingError> {
        let verify_status = self.conn.verify().amount(user_amount);

        match verify_status {
            Ok(data) => match data {
                Output::Accepted(_) => {
                    let mut current_amount: f64 = user_amount
                        .parse()
                        .map_err(|_| SteppingError::ParsingError(Field::Amount))?;

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
                Output::Nothing(_) => {
                    *user_amount = String::from("0.00");
                }
            },
            Err(e) => {
                if let VerifierError::AmountBelowZero = e {
                    if let StepType::StepUp = step_type {
                        *user_amount = String::from("1.00");
                    }
                } else {
                    return Err(SteppingError::InvalidAmount);
                }
            }
        }

        Ok(())
    }

    pub fn tx_type(
        mut self,
        user_type: &mut String,
        step_type: StepType,
    ) -> Result<(), SteppingError> {
        let verify_status = self.conn.verify().tx_type(user_type);

        if !user_type.is_empty() {
            let mut current_index: usize =
                match user_type.chars().next().unwrap().to_ascii_lowercase() {
                    'e' => 1,
                    't' => 2,
                    // 'I' is 0
                    _ => 0,
                };

            match step_type {
                StepType::StepUp => current_index = (current_index + 1) % TX_TYPES.len(),
                StepType::StepDown => {
                    current_index = (current_index + TX_TYPES.len() - 1) % TX_TYPES.len();
                }
            }
            *user_type = TX_TYPES[current_index].to_string();
        }

        match verify_status {
            Ok(data) => match data {
                Output::Accepted(_) => {}
                Output::Nothing(_) => {
                    *user_type = "Income".to_string();
                }
            },
            Err(_) => {
                return Err(SteppingError::InvalidTxType);
            }
        }

        Ok(())
    }

    pub fn tag(&self, user_tag: &mut String, step_type: StepType) -> Result<(), SteppingError> {
        let all_tags = self
            .conn
            .cache()
            .get_tags_sorted()
            .iter()
            .map(|tag| tag.name.clone())
            .collect::<Vec<String>>();

        if user_tag.is_empty() {
            if all_tags.is_empty() {
                return Err(SteppingError::InvalidTags);
            }
            *user_tag = String::from(all_tags.first().unwrap());
            return Ok(());
        }

        // If current tag is empty
        // select the first possible tag if available

        // Tags are separated by comma. Collect all the tags
        let mut current_tags = user_tag
            .split(',')
            .map(|s| s.trim().to_string())
            .collect::<Vec<String>>();

        // Tag1, tag2, tag3
        // in this case, only work with tag3, keep the rest as it is
        let working_tag = current_tags.pop().unwrap();

        let tag_exists = all_tags
            .iter()
            .position(|tag| tag.to_lowercase() == working_tag.to_lowercase());

        if tag_exists.is_none() {
            if working_tag.is_empty() {
                if all_tags.is_empty() {
                    *user_tag = current_tags.join(", ");
                } else {
                    current_tags.push(all_tags.first().unwrap().clone());
                    *user_tag = current_tags.join(", ");
                }
            } else {
                let tags = self
                    .conn
                    .cache()
                    .tags
                    .values()
                    .map(|m| m.name.clone())
                    .collect::<Vec<String>>();

                let best_match = get_best_match(&working_tag, &tags);

                current_tags.push(best_match);

                *user_tag = current_tags.join(", ");
                return Err(SteppingError::InvalidTags);
            }
        } else if let Some(index) = tag_exists {
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

            current_tags.push(all_tags[next_index].clone());
            *user_tag = current_tags.join(", ");
        }

        Ok(())
    }
}
