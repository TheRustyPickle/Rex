use chrono::naive::NaiveDate;
use rusqlite::Connection;
use std::collections::HashMap;

use crate::db::{MONTHS, YEARS};
use crate::page_handler::IndexedData;
use crate::utility::get_all_txs;

/// Stores every transaction in the database and along with
/// all balance amount after each transaction was committed
/// Gets reloaded only after a new tx is added/removed/edited
pub struct ChartData {
    all_txs: HashMap<i32, Vec<Vec<String>>>,
    all_balance: HashMap<i32, Vec<Vec<String>>>,
}

impl ChartData {
    /// Fetches and stores all tx and balance information for all months and years
    pub fn new(conn: &Connection) -> Self {
        let mut all_txs = HashMap::new();
        let mut all_balance = HashMap::new();
        for x in 0..YEARS.len() {
            for i in 0..MONTHS.len() {
                let target_id = i as i32 + (x as i32 * 12);
                let (t, b, _) = get_all_txs(conn, i, x);
                all_txs.insert(target_id, t);
                all_balance.insert(target_id, b);
            }
        }

        ChartData {
            all_txs,
            all_balance,
        }
    }

    /// Returns all dates of the transactions from the given month and year
    pub fn get_all_dates(&self, mode: &IndexedData, month: usize, year: usize) -> Vec<NaiveDate> {
        let mut to_return = vec![];

        match mode.index {
            // 0 = monthly mode. Select the data only of the given month year
            0 => {
                let target_id = month as i32 + (year as i32 * 12);
                for i in &self.all_txs[&target_id] {
                    to_return.push(NaiveDate::parse_from_str(&i[0], "%d-%m-%Y").unwrap());
                }
            }
            // 1 = yearly mode. Select the data of all months of the given year
            1 => {
                for i in 0..MONTHS.len() {
                    let target_id = i as i32 + (year as i32 * 12);
                    for i in &self.all_txs[&target_id] {
                        to_return.push(NaiveDate::parse_from_str(&i[0], "%d-%m-%Y").unwrap());
                    }
                }
            }
            // 2 = all time mode. Select every single data
            2 => {
                for x in 0..YEARS.len() {
                    for i in 0..MONTHS.len() {
                        let target_id = i as i32 + (x as i32 * 12);
                        for i in &self.all_txs[&target_id] {
                            to_return.push(NaiveDate::parse_from_str(&i[0], "%d-%m-%Y").unwrap());
                        }
                    }
                }
            }
            _ => {}
        }
        to_return
    }

    pub fn get_data(
        &self,
        mode: &IndexedData,
        month: usize,
        year: usize,
    ) -> (Vec<&Vec<String>>, Vec<&Vec<String>>) {
        let mut to_return_tx = vec![];
        let mut to_return_balance = vec![];

        match mode.index {
            // 0 = monthly mode. Select the data only of the given month year
            0 => {
                let target_id = month as i32 + (year as i32 * 12);
                for i in &self.all_txs[&target_id] {
                    to_return_tx.push(i);
                }
                for i in &self.all_balance[&target_id] {
                    to_return_balance.push(i);
                }
            }
            // 1 = yearly mode. Select the data of all months of the given year
            1 => {
                for i in 0..MONTHS.len() {
                    let target_id = i as i32 + (year as i32 * 12);
                    for i in &self.all_txs[&target_id] {
                        to_return_tx.push(i);
                    }

                    for i in &self.all_balance[&target_id] {
                        to_return_balance.push(i);
                    }
                }
            }
            // 2 = all time mode. Select every single data
            2 => {
                for x in 0..YEARS.len() {
                    for i in 0..MONTHS.len() {
                        let target_id = i as i32 + (x as i32 * 12);
                        for i in &self.all_txs[&target_id] {
                            to_return_tx.push(i);
                        }

                        for i in &self.all_balance[&target_id] {
                            to_return_balance.push(i);
                        }
                    }
                }
            }
            _ => {}
        }
        (to_return_tx, to_return_balance)
    }
}
