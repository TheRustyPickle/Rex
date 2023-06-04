use crate::tx_handler::delete_tx;
use crate::utility::{get_all_changes, get_all_tx_methods, get_all_txs, get_last_balances};
use rusqlite::{Connection, Result as sqlResult};

/// This struct stores the transaction data, balance, changes and the id num
/// Data storing format is:
///
/// all_tx : `[[date, details, tx_method, amount, tx_type, tags],]`
///
/// all_balance: `[["123.00", "123.00"],]`
///
/// all_changes: `[["↓123.00", "↑123.00"],]`
///
/// all_id_num : `["1", "2", "3",]`
pub struct TransactionData {
    pub all_tx: Vec<Vec<String>>,
    all_balance: Vec<Vec<String>>,
    all_changes: Vec<Vec<String>>,
    all_id_num: Vec<String>,
}

impl TransactionData {
    /// Calls the db to fetch transaction data, transaction changes, balances and id numbers
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

    /// returns all the Transaction data for the given index. Index is of the
    /// Home Table's selected index
    pub fn get_txs(&self) -> Vec<Vec<String>> {
        let mut table_data = Vec::new();
        for i in self.all_tx.iter() {
            table_data.push(i.clone());
        }
        table_data
    }

    /// returns all the balance data for the given index. Index is of the
    /// Home Table's selected index
    pub fn get_balance(&self, index: usize) -> Vec<String> {
        let mut balance_data = vec!["Balance".to_string()];
        for i in self.all_balance[index].iter() {
            balance_data.push(format!("{:.2}", i.parse::<f64>().unwrap()));
        }

        let mut total_balance: f64 = 0.0;
        for i in balance_data.iter().skip(1) {
            let int_bal = i.parse::<f64>().unwrap();
            total_balance += int_bal;
        }
        let formatted_total_balance = format!("{:.2}", total_balance);
        balance_data.push(formatted_total_balance);
        balance_data
    }

    /// returns the absolute final balance that is found after all transactions were counted for.
    /// * The value is saved in the DB at the final row
    pub fn get_last_balance(&self, conn: &Connection) -> Vec<String> {
        let mut balance_data = vec!["Balance".to_string()];
        let db_data = get_last_balances(conn);
        for i in db_data.iter() {
            balance_data.push(format!("{:.2}", i.parse::<f64>().unwrap()));
        }

        let mut total_balance: f64 = 0.0;
        for i in balance_data.iter().skip(1) {
            let int_bal = i.parse::<f64>().unwrap();
            total_balance += int_bal;
        }
        let formatted_total_balance = format!("{:.2}", total_balance);
        balance_data.push(formatted_total_balance);
        balance_data
    }

    /// returns all the changes data for the given index. Index is of the
    /// Home Table's selected index
    pub fn get_changes(&self, index: usize) -> Vec<String> {
        let mut changes_data = vec!["Changes".to_string()];
        for i in self.all_changes[index].iter() {
            let mut new_value = i.to_string();
            let splitted = i.split('.').collect::<Vec<&str>>();

            // the splitting and checking is necessary to make sure all strings are
            // properly ending with 2 values after dot.
            // * it's a string with ↓ or ↑
            // * so format!("{:.2}", parse to f64 won't work for the symbol.

            if splitted[1].len() == 1 {
                new_value = format!("{}0", i)
            } else if splitted[1].is_empty() {
                new_value = format!("{}.00", i)
            }
            changes_data.push(new_value);
        }
        changes_data
    }

    /// Returns the id_num of the tx of the given index
    pub fn get_id_num(&self, index: usize) -> i32 {
        self.all_id_num[index].parse::<i32>().unwrap().to_owned()
    }

    /// gets the ID Number of the selected table row and calls the function to delete a transaction from the database
    pub fn del_tx(&self, index: usize, conn: &mut Connection) -> sqlResult<()> {
        let target_id = self.get_id_num(index);
        delete_tx(target_id as usize, conn)
    }

    /// returns total incomes for the selected month by going through all the tx saved in the struct
    // Computes the total income and returns it as a vector of strings.
    pub fn get_total_income(&self, current_index: Option<usize>, conn: &Connection) -> Vec<String> {
        // Initialize the output vector with the title "Income".
        let mut final_income = vec!["Income".to_string()];

        // Get all transaction methods from the database and add a placeholder for each of them.
        let all_tx_methods = get_all_tx_methods(conn);
        for _ in all_tx_methods.iter() {
            final_income.push("-".to_string())
        }

        // Compute the stopping index based on the current index, if present.
        let mut stopping_index = -1;
        if let Some(index) = current_index {
            stopping_index = index as i32;
        }

        // Iterate over all transactions and accumulate the total income.
        let mut total_income = 0.0_f64;
        for tx in self.all_tx.iter() {
            let amount = &tx[3];
            let tx_type = &tx[4];

            if tx_type == "Income" {
                total_income += amount.parse::<f64>().unwrap();
            }

            if stopping_index == 0 {
                // We have reached the stopping index, so exit the loop early.
                break;
            } else {
                // Decrement the stopping index and continue with the loop.
                stopping_index -= 1
            }
        }

        // Add the computed total income to the output vector.
        final_income.push(format!("{:.2}", total_income));
        final_income
    }

    /// returns total expenses for the selected month by going through all the tx saved in the struct
    // Computes the total expense and returns it as a vector of strings.
    pub fn get_total_expense(
        &self,
        current_index: Option<usize>,
        conn: &Connection,
    ) -> Vec<String> {
        // Initialize the output vector with the title "Expense".
        let mut final_expense = vec!["Expense".to_string()];

        // Get all transaction methods from the database and add a placeholder for each of them.
        let all_tx_methods = get_all_tx_methods(conn);
        for _ in all_tx_methods.iter() {
            final_expense.push("-".to_string())
        }

        // Compute the stopping index based on the current index, if present.
        let mut stopping_index = -1;
        if let Some(index) = current_index {
            stopping_index = index as i32;
        }

        // Iterate over all transactions and accumulate the total expense.
        let mut total_expense = 0.0_f64;
        for tx in self.all_tx.iter() {
            let amount = &tx[3];
            let tx_type = &tx[4];

            if tx_type == "Expense" {
                total_expense += amount.parse::<f64>().unwrap();
            }

            if stopping_index == 0 {
                // We have reached the stopping index, so exit the loop early.
                break;
            } else {
                // Decrement the stopping index and continue with the loop.
                stopping_index -= 1
            }
        }

        // Add the computed total expense to the output vector.
        final_expense.push(format!("{:.2}", total_expense));
        final_expense
    }
}
