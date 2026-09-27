use std::cmp::Ordering;
use std::collections::HashSet;

use chrono::NaiveDate;
use rex_db::ConnCache;
use rex_db::models::{RecurrenceFrequency, TxType};
use strum::IntoEnumIterator;

use crate::conn::MutDbConn;
use crate::ui_helper::{DateType, Field, Output, VerifierError, get_best_match};
use crate::utils::{MONTH_NAMES, WEEKDAY_NAMES};

pub struct Verifier<'a> {
    conn: MutDbConn<'a>,
}

impl<'a> Verifier<'a> {
    pub(crate) fn new(conn: MutDbConn<'a>) -> Self {
        Self { conn }
    }

    /// Validates and normalizes the user's date input for the given `DateType`:
    ///
    /// - an empty input is left alone
    /// - every character except digits and `-` is dropped, so extra spaces and
    ///   other symbols disappear
    /// - the year must be 4 characters long, the month and the day 2
    /// - the month must be within 01-12 and the day within 01-31
    /// - for an exact date, the parts must form a date that actually exists
    ///
    /// A rejected input is repaired in place before the error is returned: a
    /// short year becomes 2022, a long one is cut down to 4 characters, single
    /// digit months and days get a leading 0 and out of range months and days
    /// are clamped to 12 and 31.
    pub fn date(
        self,
        user_date: &mut String,
        date_type: DateType,
    ) -> Result<Output, VerifierError> {
        if user_date.is_empty() {
            return Ok(Output::Nothing(Field::Date));
        }

        *user_date = user_date
            .chars()
            .filter(|c| c.is_numeric() || *c == '-')
            .collect();

        let split_date = user_date
            .split('-')
            .map(ToString::to_string)
            .collect::<Vec<String>>();

        // The part count has to match the date type, so fall back to the
        // earliest date this type accepts
        match date_type {
            DateType::Exact => {
                if split_date.len() != 3 {
                    *user_date = "2022-01-01".to_string();
                    return Err(VerifierError::InvalidDate);
                }
            }
            DateType::Monthly => {
                if split_date.len() != 2 {
                    *user_date = "2022-01".to_string();
                    return Err(VerifierError::InvalidDate);
                }
            }
            DateType::Yearly => {
                if split_date.len() != 1 {
                    *user_date = "2022".to_string();
                    return Err(VerifierError::InvalidDate);
                }
            }
        }

        let (int_month, int_day): (Option<u16>, Option<u16>) = match date_type {
            DateType::Exact => {
                let month = split_date[1]
                    .parse()
                    .map_err(|_| VerifierError::ParsingError(Field::Date))?;

                let day = split_date[2]
                    .parse()
                    .map_err(|_| VerifierError::ParsingError(Field::Date))?;

                (Some(month), Some(day))
            }
            DateType::Monthly => {
                let month = split_date[1]
                    .parse()
                    .map_err(|_| VerifierError::ParsingError(Field::Date))?;

                (Some(month), None)
            }
            DateType::Yearly => (None, None),
        };

        // The year has to be 4 characters long. A shorter one becomes 2022, a
        // longer one is cut down to its first 4 characters
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
            return Err(VerifierError::InvalidYear);
        }

        // The month has to be 2 characters long. A single digit month gets a
        // leading 0, anything above 12 is clamped to 12
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

                    return Err(VerifierError::InvalidMonth);
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

                    return Err(VerifierError::InvalidMonth);
                }
            }
            DateType::Yearly => {}
        }

        // The day has to be 2 characters long. A single digit day gets a
        // leading 0, anything above 31 is clamped to 31
        if let DateType::Exact = date_type {
            let unwrapped_day = int_day.unwrap();
            if split_date[2].len() != 2 {
                if unwrapped_day < 10 {
                    *user_date = format!("{}-{}-0{unwrapped_day}", split_date[0], split_date[1]);
                } else if unwrapped_day > 31 {
                    *user_date = format!("{}-{}-31", split_date[0], split_date[1]);
                }

                return Err(VerifierError::InvalidDay);
            }
        }

        // Clamp the month into 01-12
        match date_type {
            DateType::Exact => {
                let unwrapped_month = int_month.unwrap();
                if !(1..=12).contains(&unwrapped_month) {
                    if unwrapped_month < 1 {
                        *user_date = format!("{}-01-{}", split_date[0], split_date[2]);
                    } else if unwrapped_month > 12 {
                        *user_date = format!("{}-12-{}", split_date[0], split_date[2]);
                    }

                    return Err(VerifierError::MonthTooBig);
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

                    return Err(VerifierError::MonthTooBig);
                }
            }
            DateType::Yearly => {}
        }

        // Clamp the day into 01-31
        if let DateType::Exact = date_type {
            let unwrapped_day = int_day.unwrap();
            if !(1..=31).contains(&unwrapped_day) {
                if unwrapped_day < 1 {
                    *user_date = format!("{}-{}-01", split_date[0], split_date[1]);
                } else if unwrapped_day > 31 {
                    *user_date = format!("{}-{}-31", split_date[0], split_date[1]);
                }

                return Err(VerifierError::DayTooBig);
            }
        }

        // Months have fewer than 31 days at times, so the day still has to be
        // validated against the calendar
        if let DateType::Exact = date_type {
            NaiveDate::parse_from_str(user_date, "%Y-%m-%d")
                .map_err(|_| VerifierError::NonExistingDate)?;
        }

        Ok(Output::Accepted(Field::Date))
    }

    /// Validates and normalizes the user's amount input:
    ///
    /// - an empty input is left alone
    /// - every character except digits, `.` and the calculation symbols is dropped
    /// - a calculation such as `1+5*10` is evaluated, `*` and `/` first
    /// - the value is normalized to exactly 2 decimal places
    /// - a zero or negative value is turned positive and rejected
    pub fn amount(&self, user_amount: &mut String) -> Result<Output, VerifierError> {
        // Nothing to verify while the field is empty
        if user_amount.is_empty() {
            return Ok(Output::Nothing(Field::Amount));
        }

        let calc_symbols = vec!['*', '/', '+', '-'];

        *user_amount = user_amount
            .chars()
            .filter(|c| c.is_numeric() || *c == '.' || calc_symbols.contains(c))
            .collect();

        // The field was non empty before filtering, so it can only be empty now
        // if nothing usable was left in it
        if user_amount.is_empty() {
            return Err(VerifierError::ParsingError(Field::Amount));
        }

        // Check if any of the symbols are present
        if calc_symbols.iter().any(|s| user_amount.contains(*s)) {
            // `*` and `/` come first in `calc_symbols` so they are resolved before
            // `+` and `-`. One substitution is made per symbol found, so an
            // expression like `1+5*10` collapses `5*10` into 50 first and then
            // works on the remaining `1+50`.

            // One substitution per symbol found, so the loop below runs that many times
            let count = user_amount
                .chars()
                .filter(|c| calc_symbols.contains(c))
                .count();

            // Spaces are already gone thanks to the filter above, so indexing is safe
            let mut working_value = user_amount.to_owned();

            for _i in 0..count {
                for symbol in &calc_symbols {
                    if let Some(location) = working_value.find(*symbol) {
                        // Grab the numbers on both sides of the symbol, e.g. `1+5`
                        // gives first_value = 1 and last_value = 5
                        let mut first_value = String::new();
                        let mut last_value = String::new();

                        // Read right of the symbol, up to the end of the string or
                        // the next calculation symbol
                        for char in working_value.chars().skip(location + 1) {
                            if calc_symbols.contains(&char) {
                                break;
                            }
                            last_value.push(char);
                        }

                        // Same as above, but walking the string backwards to get the left operand
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

                        // One side is missing, as in `-5`, so there is nothing to
                        // calculate and the other side is kept as is
                        let final_value = if first_value.is_empty() || last_value.is_empty() {
                            if first_value.is_empty() {
                                last_value.clone()
                            } else {
                                first_value.clone()
                            }
                        } else {
                            // Both operands are there, so this symbol's result is ready for replacement
                            let first_num: f64 = match first_value.parse() {
                                Ok(v) => v,
                                Err(_) => {
                                    return Err(VerifierError::ParsingError(Field::Amount));
                                }
                            };

                            let last_num: f64 = match last_value.parse() {
                                Ok(v) => v,
                                Err(_) => {
                                    return Err(VerifierError::ParsingError(Field::Amount));
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

                        // e.g. `1+5*10` gives first_value = 5, last_value = 10 and
                        // symbol = `*`, so `5*10` becomes 50 and the next loop works
                        // on `1+50`
                        working_value = working_value
                            .replace(&format!("{first_value}{symbol}{last_value}"), &final_value);

                        break;
                    }
                }
            }
            *user_amount = working_value;
        }

        // Make sure a `.` exists and that something follows it, otherwise the
        // value cannot be parsed as a float
        if user_amount.contains('.') {
            let state = user_amount.split('.').collect::<Vec<&str>>();
            if state[1].is_empty() {
                *user_amount += "00";
            }
        } else {
            *user_amount = format!("{user_amount}.00");
        }

        let float_amount: f64 = user_amount
            .parse()
            .map_err(|_| VerifierError::ParsingError(Field::Amount))?;

        if float_amount <= 0.0 {
            *user_amount = format!("{:.2}", (float_amount - (float_amount * 2.0)));
            return Err(VerifierError::AmountBelowZero);
        }

        // Pad or cut the decimals down to exactly 2 digits
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

        // Safe to split now, the amount always has a `.` followed by 2 digits
        let split_amount = user_amount.split('.').collect::<Vec<&str>>();

        // The integer part is capped at 10 digits
        if split_amount[0].len() > 10 {
            *user_amount = format!("{}.{}", &split_amount[0][..10], split_amount[1]);
        }

        Ok(Output::Accepted(Field::Amount))
    }

    /// Validates the user's transaction method input:
    ///
    /// - an empty input is left alone
    /// - surrounding spaces are trimmed
    /// - the name has to match an existing method, ignoring case
    ///
    /// Anything else is fuzzy corrected to the closest existing method name and
    /// rejected.
    pub fn tx_method(&self, user_method: &mut String) -> Result<Output, VerifierError> {
        *user_method = user_method.trim().to_string();

        // Nothing to verify while the text is empty
        if user_method.is_empty() {
            return Ok(Output::Nothing(Field::TxMethod));
        }

        let all_tx_methods = self.conn.cache().get_methods();

        for method in &all_tx_methods {
            let method_name = &method.name;

            if method_name.to_lowercase() == user_method.to_lowercase() {
                *user_method = method_name.clone();
                return Ok(Output::Accepted(Field::Amount));
            }
        }

        let user_method_names = all_tx_methods
            .iter()
            .map(|m| m.name.clone())
            .collect::<Vec<String>>();

        let best_match = get_best_match(user_method, &user_method_names);

        *user_method = best_match;

        Err(VerifierError::InvalidTxMethod)
    }

    /// Validates the user's transaction type input:
    ///
    /// - a 1 or 2 character input is expanded by its first letters, so `e` becomes
    ///   Expense, `i` Income, `t` Transfer, `b` Borrow, `l` Lend and the `br` / `lr`
    ///   variants their repay types
    /// - anything longer has to be a full type name
    ///
    /// Unrecognised input is fuzzy corrected to the closest type and rejected.
    pub fn tx_type(&self, user_type: &mut String) -> Result<Output, VerifierError> {
        let trimmed_input = user_type.trim();

        if user_type.is_empty() {
            return Ok(Output::Nothing(Field::TxType));
        }

        let tx_types = TxType::iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        let return_best_match = || {
            let best_match = get_best_match(user_type, &tx_types);

            if best_match == trimmed_input {
                String::new()
            } else {
                best_match
            }
        };

        let lowercase = user_type.to_lowercase();

        if lowercase.len() <= 2 {
            if lowercase.starts_with('e') {
                *user_type = TxType::Expense.to_string();
            } else if lowercase.starts_with('i') {
                *user_type = TxType::Income.to_string();
            } else if lowercase.starts_with('t') {
                *user_type = TxType::Transfer.to_string();
            } else if lowercase.starts_with("br") {
                *user_type = TxType::BorrowRepay.to_string();
            } else if lowercase.starts_with("lr") {
                *user_type = TxType::LendRepay.to_string();
            } else if lowercase.starts_with('b') {
                *user_type = TxType::Borrow.to_string();
            } else if lowercase.starts_with('l') {
                *user_type = TxType::Lend.to_string();
            } else {
                *user_type = return_best_match();
                return Err(VerifierError::InvalidTxType);
            }
        } else {
            if tx_types.contains(user_type) {
                return Ok(Output::Accepted(Field::TxType));
            }
            *user_type = return_best_match();
            return Err(VerifierError::InvalidTxType);
        }

        Ok(Output::Accepted(Field::TxType))
    }

    /// Checks if the inputted recurrence frequency (Daily/Weekly/Monthly/Yearly) is valid,
    /// accepting the same short mnemonics (d/w/m/y)
    pub fn frequency(&self, user_freq: &mut String) -> Result<Output, VerifierError> {
        let trimmed_input = user_freq.trim();

        if user_freq.is_empty() {
            return Ok(Output::Nothing(Field::Frequency));
        }

        let frequencies = RecurrenceFrequency::iter()
            .map(|f| f.to_string())
            .collect::<Vec<String>>();

        let return_best_match = || {
            let best_match = get_best_match(user_freq, &frequencies);

            if best_match == trimmed_input {
                String::new()
            } else {
                best_match
            }
        };

        let lowercase = user_freq.to_lowercase();

        if lowercase.len() <= 2 {
            if lowercase.starts_with('d') {
                *user_freq = RecurrenceFrequency::Daily.to_string();
            } else if lowercase.starts_with('w') {
                *user_freq = RecurrenceFrequency::Weekly.to_string();
            } else if lowercase.starts_with('m') {
                *user_freq = RecurrenceFrequency::Monthly.to_string();
            } else if lowercase.starts_with('y') {
                *user_freq = RecurrenceFrequency::Yearly.to_string();
            } else {
                *user_freq = return_best_match();
                return Err(VerifierError::InvalidFrequency);
            }
        } else {
            if frequencies.contains(user_freq) {
                return Ok(Output::Accepted(Field::Frequency));
            }
            *user_freq = return_best_match();
            return Err(VerifierError::InvalidFrequency);
        }

        Ok(Output::Accepted(Field::Frequency))
    }

    /// Checks if the inputted "every N units" interval is a whole number of at least 1.
    pub fn recur_interval(&self, user_value: &mut String) -> Result<Output, VerifierError> {
        if user_value.is_empty() {
            return Ok(Output::Nothing(Field::RecurInterval));
        }

        *user_value = user_value.chars().filter(char::is_ascii_digit).collect();

        if user_value.is_empty() {
            return Err(VerifierError::ParsingError(Field::RecurInterval));
        }

        let parsed = user_value
            .parse::<i32>()
            .map_err(|_| VerifierError::ParsingError(Field::RecurInterval))?;

        if parsed < 1 {
            *user_value = "1".to_string();
            return Err(VerifierError::InvalidRecurInterval);
        }

        Ok(Output::Accepted(Field::RecurInterval))
    }

    pub fn recur_value(
        &self,
        user_value: &mut String,
        frequency: RecurrenceFrequency,
    ) -> Result<Output, VerifierError> {
        if user_value.is_empty() {
            return Ok(Output::Nothing(Field::RecurValue));
        }

        match frequency {
            RecurrenceFrequency::Daily => Ok(Output::Accepted(Field::RecurValue)),
            RecurrenceFrequency::Weekly => {
                let weekday_names = WEEKDAY_NAMES
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<String>>();

                if !weekday_names.contains(user_value) {
                    *user_value = get_best_match(user_value, &weekday_names);
                    return Err(VerifierError::InvalidRecurValueWeekly);
                }

                Ok(Output::Accepted(Field::RecurValue))
            }
            RecurrenceFrequency::Monthly | RecurrenceFrequency::Yearly => {
                *user_value = user_value.chars().filter(char::is_ascii_digit).collect();

                if user_value.is_empty() {
                    return Err(VerifierError::ParsingError(Field::RecurValue));
                }

                let parsed = user_value
                    .parse::<i32>()
                    .map_err(|_| VerifierError::ParsingError(Field::RecurValue))?;

                if !(1..=31).contains(&parsed) {
                    return Err(VerifierError::InvalidRecurValueMonthly);
                }

                Ok(Output::Accepted(Field::RecurValue))
            }
        }
    }

    /// Checks the recurrence's month name, only meaningful for Yearly.
    pub fn recur_month(&self, user_value: &mut String) -> Result<Output, VerifierError> {
        if user_value.is_empty() {
            return Ok(Output::Nothing(Field::RecurMonth));
        }

        let month_names = MONTH_NAMES
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<String>>();

        if month_names.contains(user_value) {
            return Ok(Output::Accepted(Field::RecurMonth));
        }

        *user_value = get_best_match(user_value, &month_names);
        Err(VerifierError::InvalidRecurMonth)
    }

    /// Splits the inputted tags on commas, trims each one and drops duplicates
    /// while keeping the original order
    pub fn tags(&self, user_tag: &mut String) {
        let mut split_tags = user_tag.split(',').map(str::trim).collect::<Vec<&str>>();
        split_tags.retain(|s| !s.is_empty());

        let mut seen = HashSet::new();

        // Vec for keeping the original order
        let mut unique = Vec::new();

        for item in split_tags {
            if seen.insert(item) {
                unique.push(item);
            }
        }

        *user_tag = unique.join(", ");
    }

    /// Trims and de-duplicates the inputted tags like [`Self::tags`], then drops
    /// every tag that does not exist in the database. `NonExistingTag` is returned
    /// if anything was dropped.
    pub fn tags_forced(&self, user_tag: &mut String) -> Result<Output, VerifierError> {
        if user_tag.is_empty() {
            return Ok(Output::Nothing(Field::Tags));
        }

        let all_tags = self.conn.cache().get_tags_set();

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

        unique.retain(|&tag| all_tags.contains(tag));

        let new_tags_len = unique.len();

        *user_tag = unique.join(", ");

        if old_tags_len == new_tags_len {
            Ok(Output::Accepted(Field::Tags))
        } else {
            Err(VerifierError::NonExistingTag)
        }
    }
}
