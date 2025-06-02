use crate::utility::{get_all_details, get_all_tags, get_all_tx_methods, get_best_match};
use rusqlite::Connection;

pub trait AutoFiller {
    fn autofill_tx_method(&self, user_input: &str, conn: &Connection) -> String {
        let all_tx_methods = get_all_tx_methods(conn);
        if !user_input.trim().is_empty() && !all_tx_methods.is_empty() {
            let best_match = get_best_match(user_input, &all_tx_methods);

            if best_match == user_input.trim() {
                String::new()
            } else {
                best_match
            }
        } else {
            String::new()
        }
    }

    fn autofill_tags(&self, user_input: &str, conn: &Connection) -> String {
        let all_tags = get_all_tags(conn);

        if !user_input.trim().is_empty() && !all_tags.is_empty() {
            let split_data = user_input.split(',').map(str::trim).collect::<Vec<&str>>();

            let last_value = split_data.last().unwrap().trim();

            if last_value.is_empty() {
                return String::new();
            }

            let best_match = get_best_match(last_value, &all_tags);

            if best_match == last_value {
                String::new()
            } else {
                best_match
            }
        } else {
            String::new()
        }
    }

    fn autofill_details(&self, user_input: &str, conn: &Connection) -> String {
        let all_details = get_all_details(conn);
        if !user_input.trim().is_empty() && !all_details.is_empty() {
            let best_match = get_best_match(user_input, &all_details);

            if best_match == user_input.trim() {
                String::new()
            } else {
                best_match
            }
        } else {
            String::new()
        }
    }
}
