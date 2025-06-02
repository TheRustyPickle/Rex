use rusqlite::{Connection, Result as sqlResult};
use std::collections::HashMap;

use crate::tx_handler::delete_tx;
use crate::utility::{get_all_changes, get_all_tx_methods, get_all_txs, get_last_balances};

/// This struct stores the transaction data, balance, changes, and the id number
/// Data storing format is:
///
/// `all_tx` : `[[date, details, tx_method, amount, tx_type, tags],]`
///
/// `all_balance`: `[["123.00", "123.00"],]`
///
/// `all_changes`: `[["↓123.00", "↑123.00"],]`
///
/// `all_id_num` : `["1", "2", "3",]`
pub struct TransactionData {
    all_tx: Vec<Vec<String>>,
    all_balance: Vec<Vec<String>>,
    all_changes: Vec<Vec<String>>,
    all_id_num: Vec<String>,
}

impl TransactionData {
    /// Calls the db to fetch transaction data, transaction changes, balances, and id numbers
    /// from the given month and year index
    pub fn new(month: usize, year: usize, conn: &Connection) -> Self {
        let (all_tx, all_balance, all_id_num) = get_all_txs(conn, month, year);
        let all_changes = get_all_changes(month, year, conn);
        TransactionData {
            all_tx,
            all_balance,
            all_changes,
            all_id_num,
        }
    }

    pub fn new_search(all_tx: Vec<Vec<String>>, all_id_num: Vec<String>) -> Self {
        TransactionData {
            all_tx,
            all_balance: Vec::new(),
            all_changes: Vec::new(),
            all_id_num,
        }
    }

    /// Returns all the Transaction data for the given index. Index is of the
    /// Home Table's selected index
    pub fn get_txs(&self) -> Vec<Vec<String>> {
        self.all_tx.clone()
    }

    pub fn is_tx_empty(&self) -> bool {
        self.all_tx.is_empty()
    }

    /// Returns all the balance data for the given index. Index is of the
    /// Home Table's selected index
    pub fn get_balance(&self, index: usize) -> Vec<String> {
        let mut balance_data = vec!["Balance".to_string()];
        let mut total_balance = 0.0;
        for i in &self.all_balance[index] {
            let num_balance = i.parse::<f64>().unwrap();
            total_balance += num_balance;
            balance_data.push(format!("{num_balance:.2}"));
        }
        balance_data.push(format!("{total_balance:.2}"));
        balance_data
    }

    /// Returns the absolute final balance that is found after all transactions were counted for.
    /// The value is saved in the DB at the final row
    pub fn get_last_balance(&self, conn: &Connection) -> Vec<String> {
        let mut balance_data = vec!["Balance".to_string()];
        let db_data = get_last_balances(conn);
        let mut total_balance = 0.0;
        for i in &db_data {
            let num_balance = i.parse::<f64>().unwrap();
            total_balance += num_balance;
            balance_data.push(format!("{num_balance:.2}",));
        }
        balance_data.push(format!("{total_balance:.2}",));
        balance_data
    }

    /// Returns all the changes data for the given index. Index is of the
    /// Home Table's selected index
    pub fn get_changes(&self, index: usize) -> Vec<String> {
        let mut changes_data = vec!["Changes".to_string()];
        for i in &self.all_changes[index] {
            changes_data.push(i.to_string());
        }
        changes_data
    }

    /// Returns the `id_num` of the tx of the given index
    pub fn get_id_num(&self, index: usize) -> i32 {
        self.all_id_num[index].parse::<i32>().unwrap().to_owned()
    }

    /// Gets the ID Number of the selected table row and calls the function to delete a transaction from the database
    pub fn del_tx(&self, index: usize, conn: &mut Connection) -> sqlResult<()> {
        let target_id = self.get_id_num(index);
        delete_tx(target_id, conn)
    }

    /// Returns total incomes for the selected month by going through all the tx saved in the struct
    // Computes the total income and returns it as a vector of strings.
    pub fn get_total_income(&self, current_index: Option<usize>, conn: &Connection) -> Vec<String> {
        // Initialize the output vector with the title "Income".
        let mut final_income = vec!["Income".to_string()];
        let mut income_data = HashMap::new();

        // Get all transaction methods from the database and set 0 as the default value
        let all_tx_methods = get_all_tx_methods(conn);
        for method in &all_tx_methods {
            income_data.insert(method, 0.0);
        }

        // Compute the stopping index based on the current index, if present.
        let mut stopping_index = -1;
        if let Some(index) = current_index {
            stopping_index = index as i32;
        }

        // Iterate over all transactions and accumulate the total income.
        let mut total_income = 0.0_f64;
        for tx in &self.all_tx {
            let tx_type = &tx[4];

            if tx_type == "Income" {
                let method = &tx[2];
                let amount = tx[3].parse::<f64>().unwrap();
                total_income += amount;
                *income_data.get_mut(method).unwrap() += amount;
            }

            if stopping_index == 0 {
                // We have reached the stopping index, so exit the loop early.
                break;
            }
            // Decrement the stopping index and continue with the loop.
            stopping_index -= 1;
        }

        for i in &all_tx_methods {
            final_income.push(format!("{:.2}", income_data[i]));
        }

        // Add the computed total income to the output vector.
        final_income.push(format!("{total_income:.2}"));
        final_income
    }

    /// Returns total expenses for the selected month by going through all the tx saved in the struct
    // Computes the total expense and returns it as a vector of strings.
    pub fn get_total_expense(
        &self,
        current_index: Option<usize>,
        conn: &Connection,
    ) -> Vec<String> {
        // Initialize the output vector with the title "Expense".
        let mut final_expense = vec!["Expense".to_string()];
        let mut expense_data = HashMap::new();

        // Get all transaction methods from the database and set 0 as the default value
        let all_tx_methods = get_all_tx_methods(conn);
        for method in &all_tx_methods {
            expense_data.insert(method, 0.0);
        }

        // Compute the stopping index based on the current index, if present.
        let mut stopping_index = -1;
        if let Some(index) = current_index {
            stopping_index = index as i32;
        }

        // Iterate over all transactions and accumulate the total expense.
        let mut total_expense = 0.0_f64;
        for tx in &self.all_tx {
            let tx_type = &tx[4];

            if tx_type == "Expense" {
                let method = &tx[2];
                let amount = tx[3].parse::<f64>().unwrap();
                total_expense += amount;
                *expense_data.get_mut(method).unwrap() += amount;
            }

            if stopping_index == 0 {
                // We have reached the stopping index, so exit the loop early.
                break;
            }
            // Decrement the stopping index and continue with the loop.
            stopping_index -= 1;
        }

        for i in &all_tx_methods {
            final_expense.push(format!("{:.2}", expense_data[i]));
        }

        // Add the computed total expense to the output vector.
        final_expense.push(format!("{total_expense:.2}"));
        final_expense
    }

    pub fn get_tx(&self, index: usize) -> &Vec<String> {
        &self.all_tx[index]
    }

    /// Returns total expense accumulated till the current scrolling table index on the ongoing date
    pub fn get_daily_expense(
        &self,
        current_index: Option<usize>,
        conn: &Connection,
    ) -> Vec<String> {
        let all_tx_methods = get_all_tx_methods(conn);
        let mut final_expense: Vec<String> = vec!["Daily Expense".to_string()];

        let Some(mut index) = current_index else {
            // Inclusive range to include the extra Total column value
            for _ in 0..=all_tx_methods.len() {
                final_expense.push("0.00".to_string());
            }
            return final_expense;
        };

        let mut expense_data = HashMap::new();
        let mut total_expense = 0.0_f64;

        // Get all transaction methods, set 0 as the default value
        for method in &all_tx_methods {
            expense_data.insert(method, 0.0);
        }

        let ongoing_date = self.all_tx[index][0].clone();

        // Keep reducing the ongoing index till 0 is hit or the date is different
        loop {
            let target_tx = &self.all_tx[index];

            let tx_type = &target_tx[4];

            if tx_type == "Expense" {
                let date = &target_tx[0];
                let amount = &target_tx[3];
                let tx_method = &target_tx[2];

                if date != &ongoing_date {
                    break;
                }

                let parsed_amount = amount.parse::<f64>().unwrap();
                total_expense += parsed_amount;
                *expense_data.get_mut(tx_method).unwrap() += parsed_amount;
            }

            if index == 0 {
                break;
            }

            index -= 1;
        }

        for i in &all_tx_methods {
            final_expense.push(format!("{:.2}", expense_data[i]));
        }

        // Add the computed total expense to the output vector.
        final_expense.push(format!("{total_expense:.2}"));
        final_expense
    }

    /// Returns total income accumulated till the current scrolling table index on the ongoing date
    pub fn get_daily_income(&self, current_index: Option<usize>, conn: &Connection) -> Vec<String> {
        let all_tx_methods = get_all_tx_methods(conn);
        let mut final_income: Vec<String> = vec!["Daily Income".to_string()];

        let Some(mut index) = current_index else {
            for _ in 0..=all_tx_methods.len() {
                final_income.push("0.00".to_string());
            }
            return final_income;
        };

        let mut income_data = HashMap::new();
        let mut total_income = 0.0_f64;

        // Get all transaction methods and set 0 as the default value
        for method in &all_tx_methods {
            income_data.insert(method, 0.0);
        }

        let ongoing_date = self.all_tx[index][0].clone();

        // Keep reducing the ongoing index till 0 is hit or the date is different
        loop {
            let target_tx = &self.all_tx[index];

            let tx_type = &target_tx[4];

            if tx_type == "Income" {
                let date = &target_tx[0];
                let amount = &target_tx[3];
                let tx_method = &target_tx[2];
                if date != &ongoing_date {
                    break;
                }

                let parsed_amount = amount.parse::<f64>().unwrap();
                total_income += parsed_amount;
                *income_data.get_mut(tx_method).unwrap() += parsed_amount;
            }

            if index == 0 {
                break;
            }
            index -= 1;
        }

        for i in &all_tx_methods {
            final_income.push(format!("{:.2}", income_data[i]));
        }

        // Add the computed total expense to the output vector.
        final_income.push(format!("{total_income:.2}"));
        final_income
    }
}
