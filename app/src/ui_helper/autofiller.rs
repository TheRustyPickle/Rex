use rex_db::ConnCache;
use rex_db::models::{RecurrenceFrequency, TxType};
use strum::IntoEnumIterator;

use crate::conn::MutDbConn;
use crate::ui_helper::get_best_match;
use crate::utils::{MONTH_NAMES, WEEKDAY_NAMES};

fn suggest_from_options(user_input: &str, options: &[String]) -> String {
    let trimmed_input = user_input.trim();

    if trimmed_input.is_empty() || options.is_empty() {
        return String::new();
    }

    let lowercase = trimmed_input.to_lowercase();

    let mut prefix_matches = options
        .iter()
        .filter(|o| o.to_lowercase().starts_with(&lowercase));

    let suggestion = match (prefix_matches.next(), prefix_matches.next()) {
        (Some(only_match), None) => only_match.clone(),
        _ => get_best_match(user_input, options),
    };

    if suggestion == trimmed_input {
        String::new()
    } else {
        suggestion
    }
}

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

        suggest_from_options(user_input, &methods)
    }

    #[must_use]
    pub fn tx_type(&self, user_input: &str) -> String {
        let trimmed_input = user_input.trim();

        if trimmed_input.is_empty() {
            return String::new();
        }

        let lowercase = trimmed_input.to_lowercase();

        let shortcut = if lowercase.len() <= 2 {
            if lowercase.starts_with('t') {
                Some(TxType::Transfer)
            } else if lowercase.starts_with('e') {
                Some(TxType::Expense)
            } else if lowercase.starts_with('i') {
                Some(TxType::Income)
            } else if lowercase.starts_with("br") {
                Some(TxType::BorrowRepay)
            } else if lowercase.starts_with("lr") {
                Some(TxType::LendRepay)
            } else if lowercase.starts_with('b') {
                Some(TxType::Borrow)
            } else if lowercase.starts_with('l') {
                Some(TxType::Lend)
            } else {
                None
            }
        } else {
            None
        };

        if let Some(tx_type) = shortcut {
            let tx_type = tx_type.to_string();

            return if tx_type == trimmed_input {
                String::new()
            } else {
                tx_type
            };
        }

        let tx_types = TxType::iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        suggest_from_options(user_input, &tx_types)
    }

    /// Tags are comma separated, so only the tag currently being typed (the last one) is completed
    #[must_use]
    pub fn tags(&self, user_input: &str) -> String {
        let tags = self
            .conn
            .cache()
            .tags
            .values()
            .map(|m| m.name.clone())
            .collect::<Vec<String>>();

        let last_value = user_input.rsplit(',').next().unwrap_or_default().trim();

        suggest_from_options(last_value, &tags)
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

        suggest_from_options(user_input, &details)
    }

    #[must_use]
    pub fn frequency(&self, user_input: &str) -> String {
        let frequencies = RecurrenceFrequency::iter()
            .map(|f| f.to_string())
            .collect::<Vec<String>>();

        suggest_from_options(user_input, &frequencies)
    }

    /// Only Weekly has a name to complete (the day of the week). Monthly/Yearly take a plain
    /// day-of-month number and Daily takes nothing, so those never get a suggestion.
    #[must_use]
    pub fn recur_value(&self, user_input: &str, frequency: RecurrenceFrequency) -> String {
        if frequency != RecurrenceFrequency::Weekly {
            return String::new();
        }

        let weekdays = WEEKDAY_NAMES
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<String>>();

        suggest_from_options(user_input, &weekdays)
    }

    #[must_use]
    pub fn recur_month(&self, user_input: &str) -> String {
        let months = MONTH_NAMES
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<String>>();

        suggest_from_options(user_input, &months)
    }
}
