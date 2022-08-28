use crate::db::get_all_txs;
use rusqlite::Connection;
use chrono::naive::NaiveDate;
pub struct ChartData {
    pub all_txs: Vec<Vec<String>>,
    pub all_balance: Vec<Vec<String>>,
}

impl ChartData {
    pub fn set(year: usize) -> Self {
        let year = 0;
        let mut all_txs = vec![];
        let mut all_balance = vec![];
        let conn = Connection::open("data.sqlite").expect("Could not connect to database");
        for month in 1..13 {
            let (txs, balances, _id_num) = get_all_txs(&conn, month, year);
            all_txs.extend(txs);
            all_balance.extend(balances);
        }
        ChartData {all_txs, all_balance}
    }

    pub fn get_all_dates(&self) -> Vec<NaiveDate> {
        let mut to_return = vec![];

        for i in &self.all_txs {
            to_return.push(NaiveDate::parse_from_str(&i[0], "%d-%m-%Y").unwrap());
        }
        to_return
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chart_data() {
        let data = ChartData::set(0);
        assert_eq!(vec![vec!["1".to_string()]], data.all_txs);
    }
}