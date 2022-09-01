use crate::db::get_all_txs;
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
    pub fn set(year: usize) -> Self {
        let mut all_txs = vec![];
        let mut all_balance = vec![];
        let conn = Connection::open("data.sqlite").expect("Could not connect to database");
        for month in 1..13 {
            let (txs, balances, _id_num) = get_all_txs(&conn, month, year);
            all_txs.extend(txs);
            all_balance.extend(balances);
        }
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
