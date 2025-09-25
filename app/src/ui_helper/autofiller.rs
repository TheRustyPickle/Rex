use db::ConnCache;

use crate::{conn::MutDbConn, ui_helper::get_best_match};

pub struct Autofiller<'a> {
    conn: MutDbConn<'a>,
}

impl<'a> Autofiller<'a> {
    pub(crate) fn new(conn: MutDbConn<'a>) -> Self {
        Autofiller { conn }
    }

    pub fn tx_method(&self, user_input: &str) -> String {
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
        let details = &self.conn.cache().details;

        let trimmed_input = user_input.trim();

        if trimmed_input.is_empty() || details.is_empty() {
            return String::new();
        }

        let best_match = get_best_match(user_input, details);

        if best_match == trimmed_input {
            String::new()
        } else {
            best_match
        }
    }
}
