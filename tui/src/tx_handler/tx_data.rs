use app::fetcher::FullTx;
use chrono::prelude::Local;
use rusqlite::Connection;
use std::cmp::Ordering;
use std::collections::HashMap;

use crate::outputs::{
    CheckingError, ComparisonType, NAType, StepType, SteppingError, TxType, TxUpdateError,
    VerifyingOutput,
};
use crate::page_handler::{ActivityType, DateType, TxTab};
use crate::tx_handler::{add_tx, delete_tx};
use crate::utility::traits::{AutoFiller, DataVerifier, FieldStepper};
use crate::utility::{
    add_char_to, add_new_activity, add_new_activity_tx, check_comparison, get_all_tx_methods,
    get_empty_changes, get_last_balances, get_last_tx, get_search_data, get_tx_id_num,
};

/// Contains all data for a Transaction to work
pub struct TxData {
    pub date: String,
    pub details: String,
    pub from_method: String,
    pub to_method: String,
    pub amount: String,
    pub tx_type: String,
    pub tags: String,
    pub tx_status: Vec<String>,
    pub editing_tx: bool,
    pub id_num: i32,
    pub current_index: usize,
    pub autofill: String,
}

impl DataVerifier for TxData {}

impl AutoFiller for TxData {}

impl FieldStepper for TxData {}

impl Default for TxData {
    #[cfg(not(tarpaulin_include))]
    fn default() -> Self {
        Self::new()
    }
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
            tx_status: Vec::new(),
            editing_tx: false,
            id_num: 0,
            current_index: 0,
            autofill: String::new(),
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
            tx_status: Vec::new(),
            editing_tx: false,
            id_num: 0,
            current_index: 0,
            autofill: String::new(),
        }
    }

    pub fn from_full_tx(tx: &FullTx, edit: bool) -> Self {
        Self {
            date: tx.date.format("%d-%m-%Y").to_string(),
            details: tx.details.clone().unwrap_or_default(),
            from_method: tx.from_method.name.clone(),
            to_method: tx.to_method.clone().map(|t| t.name).unwrap_or_default(),
            amount: tx.amount.to_string(),
            tx_type: tx.tx_type.to_string(),
            tags: tx
                .tags
                .clone()
                .iter()
                .map(|t| t.name.as_str())
                .collect::<Vec<&str>>()
                .join(", "),
            tx_status: Vec::new(),
            editing_tx: edit,
            id_num: tx.id,
            current_index: 0,
            autofill: String::new(),
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
            tx_status: Vec::new(),
            editing_tx: true,
            id_num,
            current_index: 0,
            autofill: String::new(),
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
    pub fn get_tx_method(&self) -> String {
        if self.tx_type == "Transfer" {
            format!("{} to {}", self.from_method, self.to_method)
        } else {
            self.from_method.to_string()
        }
    }

    #[must_use]
    pub fn get_tx_status(&self) -> &Vec<String> {
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

    /// Takes all data and adds it as a transaction
    pub fn add_tx(&mut self, conn: &mut Connection) -> Result<(), String> {
        if let Some(output) = self.check_all_fields() {
            return Err(output.to_string());
        }

        let tx_method = self.get_tx_method();

        if self.editing_tx {
            self.editing_tx = false;
            // How saving an edited tx works
            // delete the tx that was being edited from the db using the id_num ->
            // add another tx using the new data but take the earlier id to add to the db
            let deleted_tx = get_tx_id_num(self.id_num, conn);
            let status = delete_tx(self.id_num, conn);
            match status {
                Ok(()) => {}
                Err(e) => return Err(TxUpdateError::FailedEditTx(e).to_string()),
            }

            let id_num = self.id_num.to_string();
            let status_add = add_tx(
                &self.date,
                &self.details,
                &tx_method,
                &self.amount,
                &self.tx_type,
                &self.tags,
                Some(&id_num),
                conn,
            );

            match status_add {
                Ok(()) => {
                    let activity_num =
                        add_new_activity(ActivityType::EditTX(Some(self.id_num)), conn);
                    let new_tx = vec![
                        &self.date,
                        &self.details,
                        &tx_method,
                        &self.amount,
                        &self.tx_type,
                        &self.tags,
                        &id_num,
                    ];
                    add_new_activity_tx(&new_tx, activity_num, conn);
                    add_new_activity_tx(&deleted_tx, activity_num, conn);
                    Ok(())
                }
                Err(e) => Err(TxUpdateError::FailedEditTx(e).to_string()),
            }
        } else {
            let status = add_tx(
                &self.date,
                &self.details,
                &tx_method,
                &self.amount,
                &self.tx_type,
                &self.tags,
                None,
                conn,
            );
            match status {
                Ok(()) => {
                    let activity_num = add_new_activity(ActivityType::NewTX, conn);
                    let last_tx = get_last_tx(conn);
                    add_new_activity_tx(&last_tx, activity_num, conn);
                    Ok(())
                }
                Err(e) => Err(TxUpdateError::FailedAddTx(e).to_string()),
            }
        }
    }

    pub fn get_search_tx(
        &self,
        date_type: &DateType,
        conn: &Connection,
    ) -> (Vec<Vec<String>>, Vec<String>) {
        get_search_data(
            &self.date,
            &self.details,
            &self.from_method,
            &self.to_method,
            &self.amount,
            &self.tx_type,
            &self.tags,
            date_type,
            conn,
        )
    }

    /// Adds a value to tx status
    pub fn add_tx_status(&mut self, data: String) {
        if self.tx_status.len() == 30 {
            self.tx_status.remove(0);
        }
        self.tx_status.push(data);
    }

    pub fn check_autofill(&mut self, current_tab: &TxTab, conn: &Connection) {
        self.autofill.clear();

        self.autofill = match current_tab {
            TxTab::Details => self.autofill_details(&self.details, conn),
            TxTab::FromMethod => self.autofill_tx_method(&self.from_method, conn),
            TxTab::ToMethod => self.autofill_tx_method(&self.to_method, conn),
            TxTab::Tags => self.autofill_tags(&self.tags, conn),
            _ => String::new(),
        }
    }

    pub fn accept_autofill(&mut self, current_tab: &TxTab) {
        match current_tab {
            TxTab::Details => self.details = self.autofill.to_string(),
            TxTab::FromMethod => self.from_method = self.autofill.to_string(),
            TxTab::ToMethod => self.to_method = self.autofill.to_string(),
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
    pub fn check_date(&mut self, date_type: &DateType) -> VerifyingOutput {
        let mut user_date = self.date.clone();
        let status = self.verify_date(&mut user_date, date_type);

        self.date = user_date;
        self.go_current_index(&TxTab::Date);
        status
    }

    /// Checks the inputted From Method by the user upon pressing Enter/Esc for various error.
    pub fn check_from_method(&mut self, conn: &Connection) -> VerifyingOutput {
        let mut current_method = self.from_method.clone();

        let status = self.verify_tx_method(&mut current_method, conn);

        self.from_method = current_method;
        self.go_current_index(&TxTab::FromMethod);
        status
    }

    /// Checks the inputted To Method by the user upon pressing Enter/Esc for various error.
    pub fn check_to_method(&mut self, conn: &Connection) -> VerifyingOutput {
        let mut current_method = self.to_method.clone();

        let status = self.verify_tx_method(&mut current_method, conn);

        self.to_method = current_method;
        self.go_current_index(&TxTab::ToMethod);
        status
    }

    /// Checks the inputted Amount by the user upon pressing Enter/Esc for various error.
    pub fn check_amount(&mut self, is_search: bool, conn: &Connection) -> VerifyingOutput {
        if let Err(e) = self.check_b_field(conn) {
            return e;
        }

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

        let status = self.verify_amount(&mut user_amount);

        if let Some(symbol) = comparison_symbol {
            user_amount = format!("{symbol}{user_amount}");
        }

        self.amount = user_amount;

        self.go_current_index(&TxTab::Amount);
        status
    }

    /// Checks the inputted Transaction Type by the user upon pressing Enter/Esc for various error.
    pub fn check_tx_type(&mut self) -> VerifyingOutput {
        let mut tx_type = self.tx_type.clone();

        let status = self.verify_tx_type(&mut tx_type);

        self.tx_type = tx_type;
        self.go_current_index(&TxTab::TxType);
        status
    }

    /// Checks the inputted tags to make sure it's properly separated by a comma
    pub fn check_tags(&mut self) {
        let mut tags = self.tags.clone();

        self.verify_tags(&mut tags);

        self.tags = tags;
        self.go_current_index(&TxTab::Tags);
    }

    /// Checks the inputted tags to make sure it's properly separated by a comma
    pub fn check_tags_forced(&mut self, conn: &Connection) -> VerifyingOutput {
        let mut tags = self.tags.clone();

        let status = self.verify_tags_forced(&mut tags, conn);

        self.tags = tags;
        self.go_current_index(&TxTab::Tags);
        status
    }

    /// Checks all field and verifies anything important is not empty
    pub fn check_all_fields(&mut self) -> Option<CheckingError> {
        if self.date.is_empty() {
            return Some(CheckingError::EmptyDate);
        } else if self.from_method.is_empty() && self.tx_type != "Transfer" {
            return Some(CheckingError::EmptyMethod);
        } else if self.amount.is_empty() {
            return Some(CheckingError::EmptyAmount);
        } else if self.tx_type.is_empty() {
            return Some(CheckingError::EmptyTxType);
        } else if self.tx_type == "Transfer" && self.from_method == self.to_method {
            return Some(CheckingError::SameTxMethod);
        } else if self.tx_type == "Transfer"
            && (self.from_method.is_empty() || self.to_method.is_empty())
        {
            return Some(CheckingError::EmptyMethod);
        }
        // Empty tags in a tx becomes as unknown
        if self.tags.is_empty() {
            self.tags = "Unknown".to_string();
        }
        None
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
    fn check_b_field(&mut self, conn: &Connection) -> Result<(), VerifyingOutput> {
        self.check_suffixes();
        let user_amount = self.amount.to_lowercase();

        // 'b' represents the current balance of the original tx method
        if user_amount.contains('b') && !self.from_method.is_empty() {
            let all_methods = get_all_tx_methods(conn);

            // Get all the method's final balance, loop through the balances and match the tx method name
            let last_balances = get_last_balances(conn);

            for x in 0..all_methods.len() {
                if all_methods[x] == self.from_method {
                    self.amount = user_amount.replace('b', &last_balances[x]);
                    break;
                }
            }
        } else if user_amount.contains('b') && self.from_method.is_empty() {
            return Err(VerifyingOutput::NotAccepted(NAType::InvalidBValue));
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

    /// Steps up Date value by one
    pub fn do_date_up(&mut self, date_type: &DateType) -> Result<(), SteppingError> {
        let mut user_date = self.date.clone();

        let step_status = self.step_date(&mut user_date, StepType::StepUp, date_type);
        self.date = user_date;

        // Reload index to the final point as some data just got added/changed
        self.go_current_index(&TxTab::Date);
        step_status
    }

    /// Steps down Date value by one
    pub fn do_date_down(&mut self, date_type: &DateType) -> Result<(), SteppingError> {
        let mut user_date = self.date.clone();

        let step_status = self.step_date(&mut user_date, StepType::StepDown, date_type);
        self.date = user_date;

        // Reload index to the final point as some data just got added/changed
        self.go_current_index(&TxTab::Date);
        step_status
    }

    /// Steps up From Method value by one
    pub fn do_from_method_up(&mut self, conn: &Connection) -> Result<(), SteppingError> {
        let mut user_method = self.from_method.clone();

        let step_status = self.step_tx_method(&mut user_method, StepType::StepUp, conn);
        self.from_method = user_method;

        // Reload index to the final point as some data just got added/changed
        self.go_current_index(&TxTab::FromMethod);
        step_status
    }

    /// Steps down From Method value by one
    pub fn do_from_method_down(&mut self, conn: &Connection) -> Result<(), SteppingError> {
        let mut user_method = self.from_method.clone();

        let step_status = self.step_tx_method(&mut user_method, StepType::StepDown, conn);
        self.from_method = user_method;

        // Reload index to the final point as some data just got added/changed
        self.go_current_index(&TxTab::FromMethod);
        step_status
    }

    /// Steps up To Value value by one
    pub fn do_to_method_up(&mut self, conn: &Connection) -> Result<(), SteppingError> {
        let mut user_method = self.to_method.clone();

        let step_status = self.step_tx_method(&mut user_method, StepType::StepUp, conn);
        self.to_method = user_method;

        // Reload index to the final point as some data just got added/changed
        self.go_current_index(&TxTab::ToMethod);
        step_status
    }

    /// Steps down To Method value by one
    pub fn do_to_method_down(&mut self, conn: &Connection) -> Result<(), SteppingError> {
        let mut user_method = self.to_method.clone();

        let step_status = self.step_tx_method(&mut user_method, StepType::StepDown, conn);
        self.to_method = user_method;

        // Reload index to the final point as some data just got added/changed
        self.go_current_index(&TxTab::ToMethod);
        step_status
    }

    /// Steps up Tx Type value by one
    pub fn do_tx_type_up(&mut self) -> Result<(), SteppingError> {
        let mut user_type = self.tx_type.clone();

        let step_status = self.step_tx_type(&mut user_type, StepType::StepUp);
        self.tx_type = user_type;

        // Reload index to the final point as some data just got added/changed
        self.go_current_index(&TxTab::TxType);
        step_status
    }

    /// Steps down Tx Type value by one
    pub fn do_tx_type_down(&mut self) -> Result<(), SteppingError> {
        let mut user_type = self.tx_type.clone();

        let step_status = self.step_tx_type(&mut user_type, StepType::StepDown);
        self.tx_type = user_type;

        // Reload index to the final point as some data just got added/changed
        self.go_current_index(&TxTab::TxType);
        step_status
    }

    /// Steps up Amount value by one
    pub fn do_amount_up(
        &mut self,
        is_search: bool,
        conn: &Connection,
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

        let step_status = self.step_amount(&mut user_amount, StepType::StepUp);

        if let Some(symbol) = comparison_symbol {
            user_amount = format!("{symbol}{user_amount}");
        }

        self.amount = user_amount;

        // Reload index to the final point as some data just got added/changed
        self.go_current_index(&TxTab::Amount);
        step_status
    }

    /// Steps down Amount value by one
    pub fn do_amount_down(
        &mut self,
        is_search: bool,
        conn: &Connection,
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

        let step_status = self.step_amount(&mut user_amount, StepType::StepDown);

        if let Some(symbol) = comparison_symbol {
            user_amount = format!("{symbol}{user_amount}");
        }

        self.amount = user_amount;

        // Reload index to the final point as some data just got added/changed
        self.go_current_index(&TxTab::Amount);
        step_status
    }

    /// Steps up Tags value by one
    pub fn do_tags_up(&mut self, conn: &Connection) -> Result<(), SteppingError> {
        let mut user_tag = self.tags.clone();

        let status = self.step_tags(&mut user_tag, &self.autofill, StepType::StepUp, conn);
        self.tags = user_tag;

        // Reload index to the final point as some data just got added/changed
        self.go_current_index(&TxTab::Tags);
        status
    }

    /// Steps down Tags value by one
    pub fn do_tags_down(&mut self, conn: &Connection) -> Result<(), SteppingError> {
        let mut user_tag = self.tags.clone();

        let status = self.step_tags(&mut user_tag, &self.autofill, StepType::StepDown, conn);
        self.tags = user_tag;

        // Reload index to the final point as some data just got added/changed
        self.go_current_index(&TxTab::Tags);
        status
    }

    /// Add a previously deleted tx again but with a new `id_num`
    pub fn switch_tx_id(&self, new_id: i32, activity_num: i32, conn: &mut Connection) {
        add_tx(
            &self.date,
            &self.details,
            &self.get_tx_method(),
            &self.amount,
            &self.tx_type,
            &self.tags,
            Some(&new_id.to_string()),
            conn,
        )
        .unwrap();

        add_new_activity_tx(
            &[
                &self.date,
                &self.details,
                &self.get_tx_method(),
                &self.amount,
                &self.tx_type,
                &self.tags,
                &new_id.to_string(),
            ],
            activity_num,
            conn,
        );
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
        conn: &Connection,
        mut current_balance: Vec<String>,
        mut current_changes: Vec<String>,
    ) -> Vec<String> {
        let mut balance_data = vec![String::from("Balance")];

        // If editing tx, it means we have data of the balance of the current selected row on the
        // homepage. In this case, instead of showing the absolute final balance, we will show the
        // balance before the selected tx happened. So if any fields are modified, it will show
        // what the balance will look like if the new balance is saved
        let last_balance_data = if self.editing_tx {
            let mut new_balance = Vec::new();
            // First index is the text "Balance" and "Changes". Remove them
            current_balance.remove(0);
            current_changes.remove(0);
            for (balance, change) in current_balance.iter().zip(current_changes.iter()) {
                let income = if change.contains("↑") {
                    true
                } else if change.contains("↓") {
                    false
                } else {
                    new_balance.push(balance.to_string());
                    continue;
                };
                let change: f64 = change.replace("↑", "").replace("↓", "").parse().unwrap();
                let balance: f64 = balance.parse().unwrap();
                if income {
                    new_balance.push(format!("{}", balance - change));
                } else {
                    new_balance.push(format!("{}", balance + change));
                }
            }
            new_balance
        } else {
            get_last_balances(conn)
        };

        if !self.generation_fields_exists() {
            let mut total = 0.0;

            for balance in last_balance_data {
                total += balance.parse::<f64>().unwrap();
                balance_data.push(balance);
            }
            balance_data.push(total.to_string());
            return balance_data;
        }

        let all_tx_methods = get_all_tx_methods(conn);

        let mut last_balances: HashMap<String, f64> = all_tx_methods
            .iter()
            .zip(last_balance_data.iter())
            .map(|(method, balance)| (method.clone(), balance.parse().unwrap()))
            .collect();

        let from_method = &self.from_method;
        let to_method = &self.to_method;
        let amount: f64 = self.amount.parse().unwrap();

        if self.tx_type == "Transfer" {
            let target_from_balance = last_balances.get_mut(from_method).unwrap();
            *target_from_balance -= amount;

            let target_to_balance = last_balances.get_mut(to_method).unwrap();
            *target_to_balance += amount;
        } else {
            match self.tx_type.as_ref() {
                "Income" => {
                    let target_balance = last_balances.get_mut(from_method).unwrap();
                    *target_balance += amount;
                }
                "Expense" => {
                    let target_balance = last_balances.get_mut(from_method).unwrap();
                    *target_balance -= amount;
                }
                _ => unreachable!(),
            }
        }

        let mut total_bal = 0.0;
        for method in get_all_tx_methods(conn) {
            total_bal += last_balances[&method];
            balance_data.push(last_balances[&method].to_string());
        }
        balance_data.push(total_bal.to_string());

        balance_data
    }

    /// Generate Add Transaction page Balance section's changes data based on what fields are filled up
    pub fn generate_changes_section(&self, conn: &Connection) -> Vec<String> {
        let mut changes_data = vec![String::from("Changes")];

        if !self.generation_fields_exists() {
            return get_empty_changes(conn);
        }

        let from_method = &self.from_method;
        let to_method = &self.to_method;
        let amount: f64 = self.amount.parse().unwrap();

        let all_tx_methods = get_all_tx_methods(conn);

        if self.tx_type == "Transfer" {
            for method in all_tx_methods {
                if &method == from_method {
                    changes_data.push(format!("↓{amount}"));
                } else if &method == to_method {
                    changes_data.push(format!("↑{amount}"));
                } else {
                    changes_data.push("0.00".to_string());
                }
            }
        } else {
            for method in all_tx_methods {
                if &method == from_method {
                    match self.tx_type.as_ref() {
                        "Income" => changes_data.push(format!("↑{amount}")),
                        "Expense" => changes_data.push(format!("↓{amount}")),
                        _ => unreachable!(),
                    }
                } else {
                    changes_data.push("0.00".to_string());
                }
            }
        }

        changes_data
    }
}
