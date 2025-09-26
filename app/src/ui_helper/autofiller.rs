use db::ConnCache;
use db::models::TxType;

use crate::conn::MutDbConn;
use crate::ui_helper::get_best_match;

pub struct Autofiller<'a> {
    conn: MutDbConn<'a>,
}

const TX_TYPES: [&str; 3] = ["Income", "Expense", "Transfer"];

impl<'a> Autofiller<'a> {
    pub(crate) fn new(conn: MutDbConn<'a>) -> Self {
        Self { conn }
    }

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

    pub fn tx_type(&self, user_input: &str) -> String {
        let trimmed_input = user_input.trim().to_lowercase();

        if trimmed_input.is_empty() {
            return String::new();
        }

        let tx_type = if trimmed_input.starts_with("t") {
            TxType::Transfer
        } else if trimmed_input.starts_with("e") {
            TxType::Expense
        } else if trimmed_input.starts_with("i") {
            TxType::Income
        } else {
            let tx_types = TX_TYPES
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>();

            let best_match = get_best_match(user_input, &tx_types);

            let to_return = if best_match == trimmed_input {
                String::new()
            } else {
                best_match
            };

            return to_return;
        };

        tx_type.to_string()
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

    pub fn details(&self, user_input: &str) -> String {
        let details = self
            .conn
            .cache()
            .details
            .iter()
            .map(|d| d.to_string())
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
