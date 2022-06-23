use crate::sub_func::{
    delete_tx, get_all_changes, get_all_tx_methods, get_all_txs, get_last_balances,
};
use rusqlite::{Connection, Result as sqlResult};

pub struct TransactionData {
    pub all_tx: Vec<Vec<String>>,
    all_balance: Vec<Vec<String>>,
    all_changes: Vec<Vec<String>>,
    all_id_num: Vec<String>,
}

impl TransactionData {
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

    /*pub fn get_txs(&self) -> Vec<Vec<String>> {
        let mut table_data = Vec::new();
        for (i, x) in self.all_tx.iter() {
            table_data.push(x.clone())
        }
        table_data
    }*/

    pub fn get_txs(&self) -> Vec<Vec<String>> {
        let mut table_data = Vec::new();
        for i in self.all_tx.iter() {
            table_data.push(i.clone());
        }
        table_data
    }

    pub fn get_balance(&self, index: usize) -> Vec<String> {
        let mut balance_data = vec!["Balance".to_string()];
        for i in self.all_balance[index].iter() {
            if i == "0.0" {
                balance_data.push(format!("0.00"))
            } else {
                balance_data.push(format!("{}", i));
            }
        }
        balance_data
    }

    pub fn get_last_balance(&self, conn: &Connection) -> Vec<String> {
        let mut balance_data = vec!["Balance".to_string()];
        let db_data = get_last_balances(conn, &get_all_tx_methods(conn));
        for i in db_data.iter() {
            if i == "0.0" {
                balance_data.push(format!("0.00"))
            } else {
                balance_data.push(format!("{}", i))
            }
        }
        balance_data
    }

    pub fn get_changes(&self, index: usize) -> Vec<String> {
        let mut changes_data = vec!["Changes".to_string()];
        for i in self.all_changes[index].iter() {
            if i == "0.0" {
                changes_data.push(format!("0.00"));
            } else {
                changes_data.push(format!("{}", i));
            }
        }
        changes_data
    }

    pub fn del_tx(&self, conn: &Connection, index: usize) -> sqlResult<()> {
        let target_id = self.all_id_num[index].parse::<i32>().unwrap().to_owned();
        delete_tx(conn, target_id as usize)
    }
}
