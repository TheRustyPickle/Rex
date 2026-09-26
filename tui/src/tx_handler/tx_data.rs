use anyhow::Result as AResult;
use chrono::prelude::Local;
use rex_app::conn::{DbConn, FullRecurringTx, RecurrenceFrequency};
use rex_app::modifier::{parse_recurring_tx_fields, parse_search_fields, parse_tx_fields};
use rex_app::ui_helper::{DateType, Output, StepType, SteppingError, VerifierError};
use rex_app::utils::{num_to_month_name, num_to_weekday_name};
use rex_app::views::{FullTx, PartialTx, SearchView, TxViewGroup};
use rex_shared::models::Cent;
use std::cmp::Ordering;

use crate::outputs::{CheckingError, ComparisonType, TxType};
use crate::page_handler::{LogData, LogType, TxTab};
use crate::utility::{add_char_to, check_comparison};

/// Contains all data for a Transaction to work
#[derive(Default)]
pub struct TxData {
    pub date: String,
    pub details: String,
    pub from_method: String,
    pub to_method: String,
    pub amount: String,
    pub tx_type: String,
    pub tags: String,
    pub frequency: String,
    pub recur_interval: String,
    pub recur_value: String,
    pub recur_month: String,
    pub end_date: String,
    pub tx_status: Vec<LogData>,
    pub editing_tx: bool,
    pub id_num: i32,
    pub current_index: usize,
    pub autofill: String,
    pub from_search: bool,
}

impl TxData {
    /// Creates an instance of the struct however the date field is
    /// edited with the current local date of the device.
    #[must_use]
    pub fn new() -> Self {
        let current_date = Local::now().to_string();
        let formatted_current_date = &current_date[0..10];
        TxData {
            date: formatted_current_date.to_string(),
            details: String::new(),
            from_method: String::new(),
            to_method: String::new(),
            amount: String::new(),
            tx_type: String::new(),
            tags: String::new(),
            frequency: String::new(),
            recur_interval: String::new(),
            recur_value: String::new(),
            recur_month: String::new(),
            end_date: String::new(),
            tx_status: Vec::new(),
            editing_tx: false,
            id_num: 0,
            current_index: 0,
            autofill: String::new(),
            from_search: false,
        }
    }

    #[must_use]
    pub fn new_empty() -> Self {
        TxData {
            date: String::new(),
            details: String::new(),
            from_method: String::new(),
            to_method: String::new(),
            amount: String::new(),
            tx_type: String::new(),
            tags: String::new(),
            frequency: String::new(),
            recur_interval: String::new(),
            recur_value: String::new(),
            recur_month: String::new(),
            end_date: String::new(),
            tx_status: Vec::new(),
            editing_tx: false,
            id_num: 0,
            current_index: 0,
            autofill: String::new(),
            from_search: false,
        }
    }

    #[must_use]
    pub fn from_full_tx(tx: &FullTx, edit: bool, from_search: bool) -> Self {
        Self {
            date: tx.date.format("%Y-%m-%d").to_string(),
            details: tx.details.clone().unwrap_or_default(),
            from_method: tx.from_method.name.clone(),
            to_method: tx.to_method.clone().map(|t| t.name).unwrap_or_default(),
            amount: format!("{:.2}", tx.amount.dollar()),
            tx_type: tx.tx_type.to_string(),
            tags: tx
                .tags
                .clone()
                .iter()
                .map(|t| t.name.as_str())
                .collect::<Vec<&str>>()
                .join(", "),
            frequency: String::new(),
            recur_interval: String::new(),
            recur_value: String::new(),
            recur_month: String::new(),
            end_date: String::new(),
            tx_status: Vec::new(),
            editing_tx: edit,
            id_num: tx.id,
            current_index: 0,
            autofill: String::new(),
            from_search,
        }
    }

    /// Populates the Recurring page's form from an existing recurring rule, for editing.
    #[must_use]
    pub fn from_full_recurring_tx(tx: &FullRecurringTx, edit: bool) -> Self {
        let recur_value = match tx.frequency {
            RecurrenceFrequency::Daily => String::new(),
            RecurrenceFrequency::Weekly => tx
                .recur_value
                .and_then(|v| num_to_weekday_name(v).ok())
                .unwrap_or_default()
                .to_string(),
            RecurrenceFrequency::Monthly | RecurrenceFrequency::Yearly => {
                tx.recur_value.map(|v| v.to_string()).unwrap_or_default()
            }
        };

        let recur_month = tx
            .recur_month
            .and_then(|m| num_to_month_name(m as u32).ok())
            .unwrap_or_default()
            .to_string();

        Self {
            date: tx.next_recurring_date.format("%Y-%m-%d").to_string(),
            details: tx.details.clone().unwrap_or_default(),
            from_method: tx.from_method.name.clone(),
            to_method: tx.to_method.clone().map(|t| t.name).unwrap_or_default(),
            amount: format!("{:.2}", tx.amount.dollar()),
            tx_type: tx.tx_type.to_string(),
            tags: tx
                .tags
                .iter()
                .map(|t| t.name.as_str())
                .collect::<Vec<&str>>()
                .join(", "),
            frequency: tx.frequency.to_string(),
            recur_interval: tx.recur_interval.to_string(),
            recur_value,
            recur_month,
            end_date: tx
                .end_date
                .map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_default(),
            tx_status: Vec::new(),
            editing_tx: edit,
            id_num: tx.id,
            current_index: 0,
            autofill: String::new(),
            from_search: false,
        }
    }

    /// Used to adding custom pre-defined data inside the widgets of Add Transaction Page.
    /// Currently used on Editing transaction.
    #[must_use]
    pub fn custom(
        date: &str,
        details: &str,
        from_method: &str,
        to_method: &str,
        amount: &str,
        tx_type: &str,
        tags: &str,
        id_num: i32,
    ) -> Self {
        let new_date = if date.is_empty() {
            String::new()
        } else {
            let data = date.split('-').collect::<Vec<&str>>();
            let year = data[2];
            let month = data[1];
            let day = data[0];
            format!("{year}-{month}-{day}")
        };

        TxData {
            date: new_date,
            details: details.to_string(),
            from_method: from_method.to_string(),
            to_method: to_method.to_string(),
            amount: amount.to_string(),
            tx_type: tx_type.to_string(),
            tags: tags.to_string(),
            frequency: String::new(),
            recur_interval: String::new(),
            recur_value: String::new(),
            recur_month: String::new(),
            end_date: String::new(),
            tx_status: Vec::new(),
            editing_tx: true,
            id_num,
            current_index: 0,
            autofill: String::new(),
            from_search: true,
        }
    }

    /// Returns all the data saved
    #[must_use]
    pub fn get_all_texts(&self) -> Vec<&str> {
        vec![
            &self.date,
            &self.details,
            &self.from_method,
            &self.to_method,
            &self.amount,
            &self.tx_type,
            &self.tags,
            &self.autofill,
        ]
    }

    #[must_use]
    pub fn get_tx_status(&self) -> &Vec<LogData> {
        &self.tx_status
    }

    #[must_use]
    pub fn get_tx_type(&self) -> TxType {
        if let Some(first_letter) = self.tx_type.chars().next() {
            match first_letter.to_ascii_lowercase() {
                'i' | 'e' => return TxType::IncomeExpense,
                't' => return TxType::Transfer,
                _ => {}
            }
        }
        TxType::IncomeExpense
    }

    /// Returns the Recurring page's own fields: frequency, recur interval, recur value,
    /// recur month and end date, in that order.
    #[must_use]
    pub fn get_recurring_texts(&self) -> Vec<&str> {
        vec![
            &self.frequency,
            &self.recur_interval,
            &self.recur_value,
            &self.recur_month,
            &self.end_date,
        ]
    }

    /// Same safe-default idea as `get_tx_type`: never panics on an empty/partial frequency
    /// field, defaulting to Daily.
    #[must_use]
    pub fn get_frequency(&self) -> RecurrenceFrequency {
        if let Some(first_letter) = self.frequency.chars().next() {
            match first_letter.to_ascii_lowercase() {
                'd' => return RecurrenceFrequency::Daily,
                'w' => return RecurrenceFrequency::Weekly,
                'm' => return RecurrenceFrequency::Monthly,
                'y' => return RecurrenceFrequency::Yearly,
                _ => {}
            }
        }
        RecurrenceFrequency::Daily
    }

    /// Insert or remove from date field according to the index point
    pub fn edit_date(&mut self, to_add: Option<char>) {
        add_char_to(to_add, &mut self.current_index, &mut self.date);
    }

    /// Insert or remove from details field according to the index point
    pub fn edit_details(&mut self, to_add: Option<char>) {
        add_char_to(to_add, &mut self.current_index, &mut self.details);
    }

    /// Insert or remove from method field according to the index point
    pub fn edit_from_method(&mut self, to_add: Option<char>) {
        add_char_to(to_add, &mut self.current_index, &mut self.from_method);
    }

    /// Insert or remove from to method field according to the index point
    pub fn edit_to_method(&mut self, to_add: Option<char>) {
        add_char_to(to_add, &mut self.current_index, &mut self.to_method);
    }

    /// Insert or remove from amount field according to the index point
    pub fn edit_amount(&mut self, to_add: Option<char>) {
        add_char_to(to_add, &mut self.current_index, &mut self.amount);
    }

    /// Insert or remove from tx type field according to the index point
    pub fn edit_tx_type(&mut self, to_add: Option<char>) {
        add_char_to(to_add, &mut self.current_index, &mut self.tx_type);
    }

    /// Insert or remove from tags field according to the index point
    pub fn edit_tags(&mut self, to_add: Option<char>) {
        add_char_to(to_add, &mut self.current_index, &mut self.tags);
    }

    /// Insert or remove from frequency field according to the index point
    pub fn edit_frequency(&mut self, to_add: Option<char>) {
        add_char_to(to_add, &mut self.current_index, &mut self.frequency);
    }

    /// Insert or remove from recur interval field according to the index point
    pub fn edit_recur_interval(&mut self, to_add: Option<char>) {
        add_char_to(to_add, &mut self.current_index, &mut self.recur_interval);
    }

    /// Insert or remove from recur value field according to the index point
    pub fn edit_recur_value(&mut self, to_add: Option<char>) {
        add_char_to(to_add, &mut self.current_index, &mut self.recur_value);
    }

    /// Insert or remove from recur month field according to the index point
    pub fn edit_recur_month(&mut self, to_add: Option<char>) {
        add_char_to(to_add, &mut self.current_index, &mut self.recur_month);
    }

    /// Insert or remove from end date field according to the index point
    pub fn edit_end_date(&mut self, to_add: Option<char>) {
        add_char_to(to_add, &mut self.current_index, &mut self.end_date);
    }

    /// Takes all data and adds it as a transaction
    pub fn add_tx(&mut self, tx_view: &TxViewGroup, migrated_conn: &mut DbConn) -> AResult<()> {
        self.check_all_fields()?;

        let editing_tx = self.editing_tx;
        let parsed_tx = parse_tx_fields(
            &self.date,
            &self.details,
            &self.from_method,
            &self.to_method,
            &self.amount,
            &self.tx_type,
            migrated_conn,
        )?;

        if editing_tx {
            let old_tx_id = self.id_num;
            let old_tx = if let Some(tx) = tx_view.get_tx_by_id(old_tx_id) {
                tx
            } else {
                &migrated_conn.fetch_tx_with_id(old_tx_id)?
            };

            migrated_conn.edit_tx(old_tx, parsed_tx, &self.tags)
        } else {
            migrated_conn.add_new_tx(parsed_tx, &self.tags)
        }
    }

    /// Takes all Recurring page data and creates or updates a recurring transaction rule
    pub fn add_recurring_tx(&mut self, migrated_conn: &mut DbConn) -> AResult<()> {
        self.check_all_recurring_fields()?;

        let editing_tx = self.editing_tx;

        let parsed = parse_recurring_tx_fields(
            &self.date,
            &self.details,
            &self.from_method,
            &self.to_method,
            &self.amount,
            &self.tx_type,
            &self.frequency,
            &self.recur_interval,
            &self.recur_value,
            &self.recur_month,
            &self.end_date,
            migrated_conn,
        )?;

        if editing_tx {
            migrated_conn.edit_recurring_tx(self.id_num, parsed, &self.tags)
        } else {
            migrated_conn.add_recurring_tx(parsed, &self.tags)
        }
    }

    pub fn get_search_tx(&self, migrated_conn: &mut DbConn) -> AResult<SearchView> {
        let new_search = parse_search_fields(
            &self.date,
            &self.details,
            &self.from_method,
            &self.to_method,
            &self.amount,
            &self.tx_type,
            &self.tags,
            migrated_conn,
        )?;

        migrated_conn.search_txs(new_search)
    }

    /// Adds a value to tx status
    pub fn add_tx_status(&mut self, text: String, log_type: LogType) {
        if self.tx_status.len() == 30 {
            self.tx_status.remove(0);
        }

        let data = LogData::new(text, log_type);
        self.tx_status.push(data);
    }

    pub fn check_autofill(&mut self, current_tab: &TxTab, conn: &mut DbConn) {
        self.autofill.clear();

        self.autofill = match current_tab {
            TxTab::Details => conn.autofill().details(&self.details),
            TxTab::FromMethod => conn.autofill().tx_method(&self.from_method),
            TxTab::ToMethod => conn.autofill().tx_method(&self.to_method),
            TxTab::Tags => conn.autofill().tags(&self.tags),
            TxTab::TxType => conn.autofill().tx_type(&self.tx_type),
            _ => String::new(),
        }
    }

    pub fn accept_autofill(&mut self, current_tab: &TxTab) {
        match current_tab {
            TxTab::Details => self.details = self.autofill.clone(),
            TxTab::FromMethod => self.from_method = self.autofill.clone(),
            TxTab::ToMethod => self.to_method = self.autofill.clone(),
            TxTab::TxType => self.tx_type = self.autofill.clone(),
            TxTab::Tags => {
                let mut split_tags = self.tags.split(',').map(str::trim).collect::<Vec<&str>>();

                split_tags.pop().unwrap();

                split_tags.push(&self.autofill);
                self.tags = split_tags.join(", ");
            }
            _ => {}
        }
        self.autofill.clear();
        self.go_current_index(current_tab);
    }

    /// Checks the inputted Date by the user upon pressing Enter/Esc for various error.
    pub fn check_date(
        &mut self,
        date_type: DateType,
        conn: &mut DbConn,
    ) -> Result<Output, VerifierError> {
        let mut user_date = self.date.clone();
        let status = conn.verify().date(&mut user_date, date_type);

        self.date = user_date;
        self.go_current_index(&TxTab::Date);
        status
    }

    /// Checks the inputted From Method by the user upon pressing Enter/Esc for various error.
    pub fn check_from_method(&mut self, conn: &mut DbConn) -> Result<Output, VerifierError> {
        let mut current_method = self.from_method.clone();

        let status = conn.verify().tx_method(&mut current_method);

        self.from_method = current_method;
        self.go_current_index(&TxTab::FromMethod);

        status
    }

    /// Checks the inputted To Method by the user upon pressing Enter/Esc for various error.
    pub fn check_to_method(&mut self, conn: &mut DbConn) -> Result<Output, VerifierError> {
        let mut current_method = self.to_method.clone();

        let status = conn.verify().tx_method(&mut current_method);

        self.to_method = current_method;
        self.go_current_index(&TxTab::ToMethod);
        status
    }

    /// Checks the inputted Amount by the user upon pressing Enter/Esc for various error.
    pub fn check_amount(
        &mut self,
        is_search: bool,
        conn: &mut DbConn,
    ) -> Result<Output, VerifierError> {
        self.check_b_field(conn)?;

        let mut comparison_symbol = None;

        let mut user_amount = self.amount.clone().to_lowercase();

        if is_search {
            match check_comparison(&user_amount) {
                ComparisonType::Equal => comparison_symbol = None,
                ComparisonType::BiggerThan => comparison_symbol = Some(">"),
                ComparisonType::SmallerThan => comparison_symbol = Some("<"),
                ComparisonType::EqualOrBigger => comparison_symbol = Some(">="),
                ComparisonType::EqualOrSmaller => comparison_symbol = Some("<="),
            }
        }

        if let Some(symbol) = comparison_symbol {
            user_amount = user_amount.replace(symbol, "");
        }

        let status = conn.verify().amount(&mut user_amount);

        if let Some(symbol) = comparison_symbol {
            user_amount = format!("{symbol}{user_amount}");
        }

        self.amount = user_amount;

        self.go_current_index(&TxTab::Amount);
        status
    }

    /// Checks the inputted Transaction Type by the user upon pressing Enter/Esc for various error.
    pub fn check_tx_type(&mut self, conn: &mut DbConn) -> Result<Output, VerifierError> {
        let mut tx_type = self.tx_type.clone();

        let status = conn.verify().tx_type(&mut tx_type);

        self.tx_type = tx_type;
        self.go_current_index(&TxTab::TxType);
        status
    }

    /// Checks the inputted recurrence frequency by the user upon pressing Enter/Esc
    pub fn check_frequency(&mut self, conn: &mut DbConn) -> Result<Output, VerifierError> {
        let mut frequency = self.frequency.clone();

        let status = conn.verify().frequency(&mut frequency);

        self.frequency = frequency;
        self.go_current_index(&TxTab::Frequency);
        status
    }

    /// Checks the inputted recur interval by the user upon pressing Enter/Esc
    pub fn check_recur_interval(&mut self, conn: &mut DbConn) -> Result<Output, VerifierError> {
        let mut recur_interval = self.recur_interval.clone();

        let status = conn.verify().recur_interval(&mut recur_interval);

        self.recur_interval = recur_interval;
        self.go_current_index(&TxTab::RecurInterval);
        status
    }

    /// Checks the inputted recur value by the user upon pressing Enter/Esc
    pub fn check_recur_value(&mut self, conn: &mut DbConn) -> Result<Output, VerifierError> {
        let mut recur_value = self.recur_value.clone();
        let frequency = self.get_frequency();

        let status = conn.verify().recur_value(&mut recur_value, frequency);

        self.recur_value = recur_value;
        self.go_current_index(&TxTab::RecurValue);
        status
    }

    /// Checks the inputted recur month by the user upon pressing Enter/Esc
    pub fn check_recur_month(&mut self, conn: &mut DbConn) -> Result<Output, VerifierError> {
        let mut recur_month = self.recur_month.clone();

        let status = conn.verify().recur_month(&mut recur_month);

        self.recur_month = recur_month;
        self.go_current_index(&TxTab::RecurMonth);
        status
    }

    /// Checks the inputted end date by the user upon pressing Enter/Esc. Empty is valid -
    /// it means the recurrence never ends.
    pub fn check_end_date(&mut self, conn: &mut DbConn) -> Result<Output, VerifierError> {
        let mut end_date = self.end_date.clone();

        let status = conn.verify().date(&mut end_date, DateType::Exact);

        self.end_date = end_date;
        self.go_current_index(&TxTab::EndDate);
        status
    }

    /// Checks the inputted tags to make sure it's properly separated by a comma
    pub fn check_tags(&mut self, conn: &mut DbConn) {
        let mut tags = self.tags.clone();

        conn.verify().tags(&mut tags);

        self.tags = tags;
        self.go_current_index(&TxTab::Tags);
    }

    /// Checks the inputted tags to make sure it's properly separated by a comma
    pub fn check_tags_forced(&mut self, conn: &mut DbConn) -> Result<Output, VerifierError> {
        let mut tags = self.tags.clone();

        let status = conn.verify().tags_forced(&mut tags);

        self.tags = tags;
        self.go_current_index(&TxTab::Tags);
        status
    }

    /// Checks all field and verifies anything important is not empty
    pub fn check_all_fields(&mut self) -> Result<(), CheckingError> {
        if self.date.is_empty() {
            return Err(CheckingError::EmptyDate);
        } else if self.from_method.is_empty() && self.tx_type != "Transfer" {
            return Err(CheckingError::EmptyMethod);
        } else if self.amount.is_empty() {
            return Err(CheckingError::EmptyAmount);
        } else if self.tx_type.is_empty() {
            return Err(CheckingError::EmptyTxType);
        } else if self.tx_type == "Transfer" && self.from_method == self.to_method {
            return Err(CheckingError::SameTxMethod);
        } else if self.tx_type == "Transfer"
            && (self.from_method.is_empty() || self.to_method.is_empty())
        {
            return Err(CheckingError::EmptyMethod);
        }
        // Empty tags in a tx becomes as unknown
        if self.tags.is_empty() {
            self.tags = "Unknown".to_string();
        }
        Ok(())
    }

    /// Checks all Recurring page fields and verifies anything important is not empty
    pub fn check_all_recurring_fields(&mut self) -> Result<(), CheckingError> {
        if self.date.is_empty() {
            return Err(CheckingError::EmptyDate);
        } else if self.from_method.is_empty() && self.tx_type != "Transfer" {
            return Err(CheckingError::EmptyMethod);
        } else if self.amount.is_empty() {
            return Err(CheckingError::EmptyAmount);
        } else if self.tx_type.is_empty() {
            return Err(CheckingError::EmptyTxType);
        } else if self.tx_type == "Transfer" && self.from_method == self.to_method {
            return Err(CheckingError::SameTxMethod);
        } else if self.tx_type == "Transfer"
            && (self.from_method.is_empty() || self.to_method.is_empty())
        {
            return Err(CheckingError::EmptyMethod);
        } else if self.frequency.is_empty() {
            return Err(CheckingError::EmptyFrequency);
        } else if self.recur_interval.is_empty() {
            return Err(CheckingError::EmptyRecurInterval);
        }

        match self.get_frequency() {
            RecurrenceFrequency::Daily => {}
            RecurrenceFrequency::Weekly | RecurrenceFrequency::Monthly => {
                if self.recur_value.is_empty() {
                    return Err(CheckingError::EmptyRecurValue);
                }
            }
            RecurrenceFrequency::Yearly => {
                if self.recur_value.is_empty() {
                    return Err(CheckingError::EmptyRecurValue);
                } else if self.recur_month.is_empty() {
                    return Err(CheckingError::EmptyRecurMonth);
                }
            }
        }

        // Empty tags in a tx becomes as unknown
        if self.tags.is_empty() {
            self.tags = "Unknown".to_string();
        }
        Ok(())
    }

    #[must_use]
    pub fn check_all_empty(&self) -> bool {
        let all_data = [
            &self.date,
            &self.details,
            &self.from_method,
            &self.to_method,
            &self.amount,
            &self.tx_type,
            &self.tags,
        ];
        let non_empty_count = all_data.iter().filter(|&value| !value.is_empty()).count();

        if non_empty_count == 0 {
            return true;
        }

        false
    }

    /// Checks for b on amount field to replace with the balance of the tx method field
    fn check_b_field(&mut self, conn: &mut DbConn) -> Result<(), VerifierError> {
        self.check_suffixes();
        let user_amount = self.amount.to_lowercase();

        // 'b' represents the current balance of the original tx method
        if user_amount.contains('b') && !self.from_method.is_empty() {
            let final_balances = conn
                .get_final_balances()
                .map_err(|e| VerifierError::Others(e.to_string()))?;

            let target_method = conn
                .get_tx_method_by_name(self.from_method.as_str())
                .map_err(|e| VerifierError::Others(e.to_string()))?;

            // Get all the method's final balance, loop through the balances and match the tx method name

            for balance in final_balances.values() {
                if balance.method_id == target_method.id {
                    self.amount = user_amount
                        .replace('b', &(Cent::new(balance.balance).dollar()).to_string());
                }
            }
        } else if user_amount.contains('b') && self.from_method.is_empty() {
            return Err(VerifierError::InvalidBValue);
        }
        Ok(())
    }

    /// If `k` and `m` are present, multiplies the number 1 thousand and 1 million respectively
    fn check_suffixes(&mut self) {
        let mut user_amount = self.amount.to_lowercase();
        let mut failed_to_parse = false;

        // target char list
        let target_letters = ['k', 'm'];

        'starter: for letter in target_letters {
            // How many times this target char is present in the string
            let count = user_amount.chars().filter(|c| c == &letter).count();

            // We will loop the `count` of times to ensure all the target are handled
            for _ in 0..count {
                // Convert the string to a vec of char for easier indexing
                let amount_vec: Vec<char> = user_amount.chars().collect();

                // The index where the 'k' or 'm' is in the string
                let target_index = user_amount.find(letter).unwrap();
                let mut gathered_value = String::new();

                // Ending here would be smaller than the target index.
                // We are looping from target index to the beginning of the string
                let mut ending_index = 0;

                for index in (0..target_index).rev() {
                    let value = amount_vec[index];
                    if value == ' ' {
                        continue;
                    }

                    if value == '.' {
                        gathered_value = format!("{value}{gathered_value}");
                        continue;
                    }

                    // If the char is a number, convert to u16 and save it on gathered value
                    if let Ok(num_amount) = value.to_string().parse::<u16>() {
                        gathered_value = format!("{num_amount}{gathered_value}");
                    } else {
                        // 100 + 5k
                        // If this is the + char, then break the loop. The ending index is index of '+' + 1
                        // + 1 so we don't replace the original char itself after the calculation is done
                        ending_index = index + 1;
                        break;
                    }
                }

                // In the case like this: 1k1, 5m12
                // This is invalid. This ensure the next char where k or m was found is not a number
                // If number, we can't parse it here
                for value in amount_vec
                    .iter()
                    .take(user_amount.len())
                    .skip(target_index + 1)
                {
                    if *value == ' ' {
                        continue;
                    }

                    if value.to_string().parse::<u16>().is_ok() {
                        failed_to_parse = true;
                        break 'starter;
                    }
                    break;
                }

                if let Ok(parsed_value) = gathered_value.parse::<f64>() {
                    let suffixed_added_value = match letter {
                        'k' => parsed_value * 1_000.0,
                        'm' => parsed_value * 1_000_000.0,
                        _ => unreachable!(),
                    };

                    // Example string: 100 + 5k
                    // Target index would be the index of 'k' and ending index is the index of '+' + 1
                    // Replace everything in that range with the new calculated value
                    user_amount = user_amount.replacen(
                        &user_amount[ending_index..=target_index],
                        &suffixed_added_value.to_string(),
                        1,
                    );
                }
            }
        }

        if !failed_to_parse {
            self.amount = user_amount;
        }
    }

    pub fn clear_date(&mut self) {
        self.date = String::new();
    }

    /// Returns the current index
    #[must_use]
    pub fn get_current_index(&self) -> usize {
        self.current_index
    }

    /// Returns the length of the data based on which `TxTab` is selected
    fn get_data_len(&self, current_tab: &TxTab) -> usize {
        match current_tab {
            TxTab::Date => self.date.len(),
            TxTab::Details => self.details.len(),
            TxTab::FromMethod => self.from_method.len(),
            TxTab::ToMethod => self.to_method.len(),
            TxTab::Amount => self.amount.len(),
            TxTab::TxType => self.tx_type.len(),
            TxTab::Tags => self.tags.len(),
            TxTab::Frequency => self.frequency.len(),
            TxTab::RecurInterval => self.recur_interval.len(),
            TxTab::RecurValue => self.recur_value.len(),
            TxTab::RecurMonth => self.recur_month.len(),
            TxTab::EndDate => self.end_date.len(),
            TxTab::Nothing => 0,
        }
    }

    /// Moves index by one value to left
    pub fn move_index_left(&mut self, current_tab: &TxTab) {
        let data_len = self.get_data_len(current_tab);

        if self.current_index > data_len {
            self.current_index = data_len;
        } else if self.current_index > 0 {
            self.current_index -= 1;
        }
    }

    /// Moves index by one value to right
    pub fn move_index_right(&mut self, current_tab: &TxTab) {
        let data_len = self.get_data_len(current_tab);

        match data_len.cmp(&self.current_index) {
            Ordering::Less => self.current_index = data_len,
            Ordering::Greater => self.current_index += 1,
            Ordering::Equal => {}
        }
    }

    /// Set current index to max point based on `TxTab`
    pub fn go_current_index(&mut self, current_tab: &TxTab) {
        self.current_index = self.get_data_len(current_tab);
    }

    /// Steps Date value by one
    pub fn step_date(
        &mut self,
        date_type: DateType,
        step_type: StepType,
        conn: &mut DbConn,
    ) -> Result<(), SteppingError> {
        let mut user_date = self.date.clone();

        let step_status = conn.step().date(&mut user_date, step_type, date_type);
        self.date = user_date;

        // Reload index to the final point as some data just got added/changed
        self.go_current_index(&TxTab::Date);
        step_status
    }

    /// Steps From Method value by one
    pub fn step_from_method(
        &mut self,
        step_type: StepType,
        conn: &mut DbConn,
    ) -> Result<(), SteppingError> {
        let mut user_method = self.from_method.clone();

        let step_status = conn.step().tx_method(&mut user_method, step_type);
        self.from_method = user_method;

        // Reload index to the final point as some data just got added/changed
        self.go_current_index(&TxTab::FromMethod);
        step_status
    }

    /// Steps To Value value by one
    pub fn step_to_method(
        &mut self,
        step_type: StepType,
        conn: &mut DbConn,
    ) -> Result<(), SteppingError> {
        let mut user_method = self.to_method.clone();

        let step_status = conn.step().tx_method(&mut user_method, step_type);
        self.to_method = user_method;

        // Reload index to the final point as some data just got added/changed
        self.go_current_index(&TxTab::ToMethod);
        step_status
    }

    /// Steps Tx Type value by one
    pub fn step_tx_type(
        &mut self,
        step_type: StepType,
        conn: &mut DbConn,
    ) -> Result<(), SteppingError> {
        let mut user_type = self.tx_type.clone();

        let step_status = conn.step().tx_type(&mut user_type, step_type);
        self.tx_type = user_type;

        // Reload index to the final point as some data just got added/changed
        self.go_current_index(&TxTab::TxType);
        step_status
    }

    /// Steps Frequency value by one
    pub fn step_frequency(
        &mut self,
        step_type: StepType,
        conn: &mut DbConn,
    ) -> Result<(), SteppingError> {
        let mut frequency = self.frequency.clone();

        let step_status = conn.step().frequency(&mut frequency, step_type);
        self.frequency = frequency;

        self.go_current_index(&TxTab::Frequency);
        step_status
    }

    /// Steps Recur Interval value by one
    pub fn step_recur_interval(
        &mut self,
        step_type: StepType,
        conn: &mut DbConn,
    ) -> Result<(), SteppingError> {
        let mut recur_interval = self.recur_interval.clone();

        let step_status = conn.step().recur_interval(&mut recur_interval, step_type);
        self.recur_interval = recur_interval;

        self.go_current_index(&TxTab::RecurInterval);
        step_status
    }

    /// Steps Recur Value value by one
    pub fn step_recur_value(
        &mut self,
        step_type: StepType,
        conn: &mut DbConn,
    ) -> Result<(), SteppingError> {
        let mut recur_value = self.recur_value.clone();
        let frequency = self.get_frequency();

        let step_status = conn.step().recur_value(&mut recur_value, frequency, step_type);
        self.recur_value = recur_value;

        self.go_current_index(&TxTab::RecurValue);
        step_status
    }

    /// Steps Recur Month value by one
    pub fn step_recur_month(
        &mut self,
        step_type: StepType,
        conn: &mut DbConn,
    ) -> Result<(), SteppingError> {
        let mut recur_month = self.recur_month.clone();

        let step_status = conn.step().recur_month(&mut recur_month, step_type);
        self.recur_month = recur_month;

        self.go_current_index(&TxTab::RecurMonth);
        step_status
    }

    /// Steps End Date value by one
    pub fn step_end_date(
        &mut self,
        step_type: StepType,
        conn: &mut DbConn,
    ) -> Result<(), SteppingError> {
        let mut end_date = self.end_date.clone();

        let step_status = conn.step().date(&mut end_date, step_type, DateType::Exact);
        self.end_date = end_date;

        self.go_current_index(&TxTab::EndDate);
        step_status
    }

    /// Steps Amount value by one
    pub fn step_amount(
        &mut self,
        is_search: bool,
        step_type: StepType,
        conn: &mut DbConn,
    ) -> Result<(), SteppingError> {
        if self.check_b_field(conn).is_err() {
            return Err(SteppingError::UnknownBValue);
        }

        let mut comparison_symbol = None;

        let mut user_amount = self.amount.clone();

        if is_search {
            match check_comparison(&user_amount) {
                ComparisonType::Equal => comparison_symbol = None,
                ComparisonType::BiggerThan => comparison_symbol = Some(">"),
                ComparisonType::SmallerThan => comparison_symbol = Some("<"),
                ComparisonType::EqualOrBigger => comparison_symbol = Some(">="),
                ComparisonType::EqualOrSmaller => comparison_symbol = Some("<="),
            }
        }

        if let Some(symbol) = comparison_symbol {
            user_amount = user_amount.replace(symbol, "");
        }

        let step_status = conn.step().amount(&mut user_amount, step_type);

        if let Some(symbol) = comparison_symbol {
            user_amount = format!("{symbol}{user_amount}");
        }

        self.amount = user_amount;

        // Reload index to the final point as some data just got added/changed
        self.go_current_index(&TxTab::Amount);
        step_status
    }

    /// Steps up Tags value by one
    pub fn step_tags(
        &mut self,
        step_type: StepType,
        conn: &mut DbConn,
    ) -> Result<(), SteppingError> {
        let mut user_tag = self.tags.clone();

        let status = conn.step().tag(&mut user_tag, step_type);
        self.tags = user_tag;

        // Reload index to the final point as some data just got added/changed
        self.go_current_index(&TxTab::Tags);
        status
    }

    /// Whether the required fields for balance section data generate is filled up
    fn generation_fields_exists(&self) -> bool {
        if self.amount.is_empty() {
            return false;
        }

        if self.tx_type.is_empty() {
            return false;
        }

        if self.from_method.is_empty() {
            return false;
        }

        if self.tx_type == "Transfer" && self.to_method.is_empty() {
            return false;
        }
        true
    }

    /// Generate Add Transaction page Balance section's balance data based on what fields are filled up
    pub fn generate_balance_section(
        &self,
        tx_view: &TxViewGroup,
        index: Option<usize>,
        migrated_conn: &mut DbConn,
    ) -> AResult<Vec<Vec<String>>> {
        if self.generation_fields_exists() {
            let partial_tx = PartialTx {
                from_method: &self.from_method,
                to_method: &self.to_method,
                amount: &self.amount,
                tx_type: &self.tx_type,
            };

            tx_view.add_tx_balance_array(index, Some(partial_tx), migrated_conn)
        } else {
            tx_view.add_tx_balance_array(index, None, migrated_conn)
        }
    }
}
