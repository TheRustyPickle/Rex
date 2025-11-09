use rex_db::ConnCache;
use rex_db::models::TxType;
use strum::IntoEnumIterator;

use crate::conn::MutDbConn;
use crate::ui_helper::get_best_match;

pub struct Autofiller<'a> {
    conn: MutDbConn<'a>,
}

impl<'a> Autofiller<'a> {
    pub(crate) fn new(conn: MutDbConn<'a>) -> Self {
        Self { conn }
    }

    #[must_use]
    pub fn tx_method(self, user_input: &str) -> String {
        let methods = self
            .conn
            .cache()
            .tx_methods
            .values()
            .map(|m| m.name.clone())
            .collect::<Vec<String>>();

        let trimmed_input = user_input.trim();

        if trimmed_input.is_empty() || methods.is_empty() {
            return String::new();
        }

        let best_match = get_best_match(user_input, &methods);

        if best_match == trimmed_input {
            String::new()
        } else {
            best_match
        }
    }

    #[must_use]
    pub fn tx_type(&self, user_input: &str) -> String {
        let trimmed_input = user_input.trim();

        let lowercase = trimmed_input.to_lowercase();

        if trimmed_input.is_empty() {
            return String::new();
        }

        let return_best_match = || {
            let tx_types = TxType::iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>();

            let best_match = get_best_match(user_input, &tx_types);

            if best_match == trimmed_input {
                String::new()
            } else {
                best_match
            }
        };

        let best_match = if lowercase.len() <= 2 {
            let tx_type = if lowercase.starts_with('t') {
                TxType::Transfer.to_string()
            } else if lowercase.starts_with('e') {
                TxType::Expense.to_string()
            } else if lowercase.starts_with('i') {
                TxType::Income.to_string()
            } else if lowercase.starts_with("br") {
                TxType::BorrowRepay.to_string()
            } else if lowercase.starts_with("lr") {
                TxType::LendRepay.to_string()
            } else if lowercase.starts_with('b') {
                TxType::Borrow.to_string()
            } else if lowercase.starts_with('l') {
                TxType::Lend.to_string()
            } else {
                return_best_match()
            };

            tx_type.clone()
        } else {
            return_best_match()
        };

        if best_match == trimmed_input {
            String::new()
        } else {
            best_match
        }
    }

    pub fn tags(&self, user_input: &str) -> String {
        let tags = self
            .conn
            .cache()
            .tags
            .values()
            .map(|m| m.name.clone())
            .collect::<Vec<String>>();

        let trimmed_input = user_input.trim();

        if trimmed_input.is_empty() || tags.is_empty() {
            return String::new();
        }

        let split_data = user_input.split(',').map(str::trim).collect::<Vec<&str>>();

        let last_value = split_data.last().unwrap().trim();

        if last_value.is_empty() {
            return String::new();
        }

        let best_match = get_best_match(last_value, &tags);

        if best_match == last_value {
            String::new()
        } else {
            best_match
        }
    }

    #[must_use]
    pub fn details(&self, user_input: &str) -> String {
        let details = self
            .conn
            .cache()
            .details
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<String>>();

        let trimmed_input = user_input.trim();

        if trimmed_input.is_empty() || details.is_empty() {
            return String::new();
        }

        let best_match = get_best_match(user_input, &details);

        if best_match == trimmed_input {
            String::new()
        } else {
            best_match
        }
    }
}
