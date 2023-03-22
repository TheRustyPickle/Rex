use crate::db::{get_all_txs, get_month_name};
use rusqlite::Connection;
use std::collections::HashMap;
/// Contains the necessary information to construct the Summary Page highlighting
/// tag based expense and income information, biggest expense and income
pub struct SummaryData {
    tags_income: HashMap<String, f64>,
    tags_expense: HashMap<String, f64>,
    biggest_income: (f64, String),
    biggest_expense: (f64, String),
    income_date: (f64, String),
    expense_date: (f64, String),
    total_income: f64,
    total_expense: f64,
}

impl SummaryData {
    /// Goes through all transactions to collect data for the summary
    pub fn new(conn: &Connection) -> Self {
        // * create a default value in case of no data available
        let mut default = SummaryData {
            tags_income: HashMap::new(),
            tags_expense: HashMap::new(),
            biggest_income: (0.0, String::from("Not Found")),
            biggest_expense: (0.0, String::from("Not Found")),
            income_date: (0.0, String::from("Not Found")),
            expense_date: (0.0, String::from("Not Found")),
            total_income: 0.0,
            total_expense: 0.0,
        };

        // * start collecting transaction based on month
        for year in 0..4 {
            for month in 0..12 {
                default.collect_data(conn, month, year);
            }
        }

        default
    }

    /// Returns a vector that will be used to creating table in the Summary UI
    /// The vector contains tags and their income and expense data
    pub fn get_table_data(&self) -> Vec<Vec<String>> {
        let mut to_return = Vec::new();

        for (key, value) in &self.tags_income {
            let mut to_push = vec![key.to_string(), format!("{:.2}", value)];
            if self.tags_expense.contains_key(key) {
                to_push.push(format!("{:.2}", self.tags_expense[key]));
            } else {
                to_push.push("0.00".to_string())
            }
            to_return.push(to_push);
        }

        for (key, value) in &self.tags_expense {
            if !self.tags_income.contains_key(key) {
                to_return.push(vec![
                    key.to_string(),
                    "0.00".to_string(),
                    format!("{:.2}", value),
                ])
            }
        }
        to_return.sort();
        to_return
    }

    /// Returns a vector that will be used to highlight points such as largest transaction,
    /// biggest income etc
    pub fn get_tx_data(&self) -> Vec<(f64, String)> {
        vec![
            (self.total_income, "Total Income:".to_string()),
            (self.total_expense, "Total Expense:".to_string()),
            self.biggest_income.to_owned(),
            self.biggest_expense.to_owned(),
            self.income_date.to_owned(),
            self.expense_date.to_owned(),
        ]
    }

    /// Collects data from the given month and year, updates SummaryData with relevant information
    fn collect_data(&mut self, conn: &Connection, month: usize, year: usize) {
        let mut total_income = 0.0;
        let mut total_expense = 0.0;

        let (cu_month, cu_year) = get_month_name(month, year);

        let (all_tx, ..) = get_all_txs(conn, month, year);
        for tx in all_tx {
            let tx_date = &tx[0];
            let tx_method = &tx[2];
            let tx_amount: f64 = tx[3].parse().unwrap();
            let tx_type = &tx[4];
            let tx_tags = tx[5].split(", ").collect::<Vec<&str>>();

            if tx_type == "Income" {
                if tx_amount > self.biggest_income.0 {
                    self.biggest_income = (tx_amount, format!("{}, Date: {}", tx_method, tx_date));
                }
                total_income += tx_amount;

                for tag in tx_tags {
                    if self.tags_income.contains_key(tag) {
                        let previous_value = self.tags_income[tag];
                        *self.tags_income.get_mut(tag).unwrap() = tx_amount + previous_value;
                    } else {
                        self.tags_income.insert(tag.to_string(), tx_amount);
                    }
                }
            } else if tx_type == "Expense" {
                if tx_amount > self.biggest_expense.0 {
                    self.biggest_expense = (tx_amount, format!("{}, Date: {}", tx_method, tx_date));
                }
                total_expense += tx_amount;

                for tag in tx_tags {
                    if self.tags_expense.contains_key(tag) {
                        let previous_value = self.tags_expense[tag];
                        *self.tags_expense.get_mut(tag).unwrap() = tx_amount + previous_value;
                    } else {
                        self.tags_expense.insert(tag.to_string(), tx_amount);
                    }
                }
            }
        }

        if total_income > self.income_date.0 {
            self.income_date = (total_income, format!("{} of {}", cu_month, cu_year));
        }
        if total_expense > self.expense_date.0 {
            self.expense_date = (total_expense, format!("{} of {}", cu_month, cu_year));
        }

        self.total_income += total_income;
        self.total_expense += total_expense;
    }
}
