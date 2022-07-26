use crate::db::{delete_tx, get_all_changes, get_all_tx_methods, get_all_txs, get_last_balances};
use rusqlite::{Connection, Result as sqlResult};

/// This struct stores the transaction data, balance, changes and the id num
/// TX, Balance, Changes are parsed and selected to only show the relevant part.
/// Data storing format is:
///
/// all_tx : `[[date, details, tx_method, source, tx_type],]`
///
/// all_balance: `[["123.00", "123.00"],]`
///
/// all_changes: `[["↓123.00", "↑123.00"],]`
///
/// all_id_num : `["1", "2", "3",]` required only to delete the transaction
pub struct TransactionData {
    pub all_tx: Vec<Vec<String>>,
    all_balance: Vec<Vec<String>>,
    all_changes: Vec<Vec<String>>,
    all_id_num: Vec<String>,
}

impl TransactionData {
    /// Calls the db to fetch transaction data, transaction changes, balances and id numbers
    pub fn new(conn: &Connection, month: usize, year: usize) -> Self {
        let (all_tx, all_balance, all_id_num) = get_all_txs(conn, month, year);
        let all_changes = get_all_changes(conn, month, year);
        TransactionData {
            all_tx,
            all_balance,
            all_changes,
            all_id_num,
        }
    }

    /// returns all the Transaction data that is saved inside the struct for the current selected month
    pub fn get_txs(&self) -> Vec<Vec<String>> {
        let mut table_data = Vec::new();
        for i in self.all_tx.iter() {
            table_data.push(i.clone());
        }
        table_data
    }

    /// returns all the balance data that is saved inside the struct for the current selected month
    pub fn get_balance(&self, index: usize) -> Vec<String> {
        let mut balance_data = vec!["Balance".to_string()];
        for i in self.all_balance[index].iter() {
            balance_data.push(format!("{:.2}", i.parse::<f32>().unwrap()));
        }

        let mut total_balance: f32 = 0.0;
        for i in balance_data.iter().skip(1) {
            let int_bal = i.parse::<f32>().unwrap();
            total_balance += int_bal;
        }
        let formatted_total_balance = format!("{:.2}", total_balance);
        balance_data.push(formatted_total_balance);
        balance_data
    }

    /// returns the absolute final balance that is found after all transactions were counted for.
    /// The value is saved in the DB at the final row
    pub fn get_last_balance(&self, conn: &Connection) -> Vec<String> {
        let mut balance_data = vec!["Balance".to_string()];
        let db_data = get_last_balances(conn, &get_all_tx_methods(conn));
        for i in db_data.iter() {
            balance_data.push(format!("{:.2}", i.parse::<f32>().unwrap()));
        }

        let mut total_balance: f32 = 0.0;
        for i in balance_data.iter().skip(1) {
            let int_bal = i.parse::<f32>().unwrap();
            total_balance += int_bal;
        }
        let formatted_total_balance = format!("{:.2}", total_balance);
        balance_data.push(formatted_total_balance);
        balance_data
    }

    /// returns the data of balance Changes of a specific index, in this case, the selected table row index
    pub fn get_changes(&self, index: usize) -> Vec<String> {
        let mut changes_data = vec!["Changes".to_string()];
        for i in self.all_changes[index].iter() {
            let mut new_value = i.to_string();
            let split = i.split(".");
            let splitted = split.collect::<Vec<&str>>();

            // the splitting and checking is necessary to make sure all strings are
            // properly ending with 2 values after dot. it's a string with ↓ or ↑
            // so format!("{:.2}", parse to f32) won't work.

            if splitted[1].len() == 1 {
                new_value = format!("{}0", i)
            }
            changes_data.push(new_value);
        }
        changes_data
    }

    /// Returns the id_num of the given index
    pub fn get_id_num(&self, index: usize) -> i32 {
        self.all_id_num[index].parse::<i32>().unwrap().to_owned()
    }

    /// gets the ID Number of the selected table row and calls the function to delete a transaction from the database
    pub fn del_tx(&self, index: usize) -> sqlResult<()> {
        let target_id = self.all_id_num[index].parse::<i32>().unwrap().to_owned();
        delete_tx(target_id as usize, "data.sqlite")
    }

    /// returns total incomes for the selected month by going through all the tx saved in the struct
    pub fn get_total_income(&self, conn: &Connection, cu_index: Option<usize>) -> Vec<String> {
        let mut final_income = vec!["Income".to_string()];
        let all_tx_methods = get_all_tx_methods(conn);
        for _i in all_tx_methods.iter() {
            final_income.push("-".to_string())
        }

        let mut stopping_index = -1;

        if let Some(a) = cu_index {
            stopping_index = a as i32;
        }

        let mut total_income = 0.0_f32;
        for tx in self.all_tx.iter() {
            let amount = &tx[3];
            let tx_type = &tx[4];

            if tx_type == "Income" {
                total_income += amount.parse::<f32>().unwrap();
            }
            if stopping_index == 0 {
                break;
            } else {
                stopping_index -= 1
            }
        }
        final_income.push(format!("{:.2}", total_income));
        final_income
    }

    /// returns total expenses for the selected month by going through all the tx saved in the struct
    pub fn get_total_expense(&self, conn: &Connection, cu_index: Option<usize>) -> Vec<String> {
        let mut final_expense = vec!["Expense".to_string()];
        let all_tx_methods = get_all_tx_methods(conn);
        for _i in all_tx_methods.iter() {
            final_expense.push("-".to_string())
        }

        let mut stopping_index = -1;

        if let Some(a) = cu_index {
            stopping_index = a as i32;
        }

        let mut total_expense = 0.0_f32;
        for tx in self.all_tx.iter() {
            let amount = &tx[3];
            let tx_type = &tx[4];

            if tx_type == "Expense" {
                total_expense += amount.parse::<f32>().unwrap();
            }
            if stopping_index == 0 {
                break;
            } else {
                stopping_index -= 1
            }
        }
        final_expense.push(format!("{:.2}", total_expense));
        final_expense
    }
}
