use crate::db::{MONTHS, YEARS};
use crate::page_handler::IndexedData;
use crate::utility::get_all_txs;
use chrono::naive::NaiveDate;
use rusqlite::Connection;

/// Stores relevant data to create a chart from the transaction and balance changes
/// all_txs contains all the transaction
/// all_balance contains all the balance changes after each transaction happened
pub struct ChartData {
    pub all_txs: Vec<Vec<String>>,
    pub all_balance: Vec<Vec<String>>,
}

impl ChartData {
    /// Gets all the transaction of the given year and saves them in the struct
    pub fn set(mode: &IndexedData, month: usize, year: usize, conn: &Connection) -> Self {
        let (all_txs, all_balance) = match mode.index {
            // 0 = Monthly data
            // 1 = Yearly data
            // 2 = Every single transaction data
            0 => {
                let (txs, balance, _) = get_all_txs(conn, month, year);
                (txs, balance)
            }
            1 => {
                let mut txs = vec![];
                let mut balance = vec![];
                for i in 0..MONTHS.len() {
                    let (t, b, _) = get_all_txs(conn, i, year);
                    txs.extend(t);
                    balance.extend(b);
                }
                (txs, balance)
            }
            2 => {
                let mut txs = vec![];
                let mut balance = vec![];
                for x in 0..YEARS.len() {
                    for i in 0..MONTHS.len() {
                        let (t, b, _) = get_all_txs(conn, i, x);
                        txs.extend(t);
                        balance.extend(b);
                    }
                }

                (txs, balance)
            }
            _ => (Vec::new(), Vec::new()),
        };

        ChartData {
            all_txs,
            all_balance,
        }
    }

    /// Returns all dates of the transactions that were collected in the struct
    pub fn get_all_dates(&self) -> Vec<NaiveDate> {
        let mut to_return = vec![];

        for i in &self.all_txs {
            to_return.push(NaiveDate::parse_from_str(&i[0], "%d-%m-%Y").unwrap());
        }
        to_return
    }
}
