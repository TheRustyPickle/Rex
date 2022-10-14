use crate::db::{get_all_txs, get_month_name};
use rusqlite::Connection;
use std::collections::HashMap;

/// Contains the necessary information to construct the Summary Page highlighting
/// tag based expense and income information, biggest expense and income

// TODO add some tests for this struct
pub struct SummaryData {
    tags_income: HashMap<String, f64>,
    tags_expense: HashMap<String, f64>,
    biggest_income: (f64, String),
    biggest_expense: (f64, String),
    income_date: (f64, String, String),
    expense_date: (f64, String, String),
}

impl SummaryData {
    pub fn new(conn: &Connection) -> Self {
        let mut default = SummaryData {
            tags_income: HashMap::new(),
            tags_expense: HashMap::new(),
            biggest_income: (0.0, String::new()),
            biggest_expense: (0.0, String::new()),
            income_date: (0.0, String::new(), String::new()),
            expense_date: (0.0, String::new(), String::new()),
        };

        for year in 0..4 {
            for month in 0..12 {
                default.collect_data(conn, month, year);
            }
        }
        

        default
    }

    pub fn get_table_data(&self) -> Vec<Vec<String>> {
        let mut to_return = Vec::new();

        for (key, value) in &self.tags_income {
            let mut to_push = vec![key.to_string(), format!("{:.2}",value)];
            if self.tags_expense.contains_key(key) {
                to_push.push(self.tags_expense[key].to_string());
            } else {
                to_push.push("0".to_string())
            }
            to_return.push(to_push);
        }

        for (key, value) in &self.tags_expense {
            if !self.tags_income.contains_key(key) {
                to_return.push(vec![key.to_string(), "0".to_string(), format!("{:.2}",value)])
            }
        }
        to_return
    }

    fn collect_data(&mut self, conn: &Connection, month: usize, year: usize) {
        let mut total_income = 0.0;
        let mut total_expense = 0.0;

        let mut biggest_income = (0.0, "".to_string());
        let mut biggest_expense = (0.0, "".to_string());

        let (cu_month, cu_year) = get_month_name(month, year);

        let (all_tx, ..) = get_all_txs(conn, month, year);
        for tx in all_tx {
            let tx_method = &tx[2];
            let tx_amount: f64 = tx[3].parse().unwrap();
            let tx_type = &tx[4];
            let tx_tags = tx[5].split(", ").collect::<Vec<&str>>();

            if tx_type == "Income" {
                if tx_amount > self.biggest_income.0 {
                    self.biggest_income = (tx_amount, tx_method.to_string());
                    total_income += tx_amount;
                }

                for tag in tx_tags {
                    if self.tags_income.contains_key(tag) {
                        let previous_value = self.tags_income[tag];
                        *self.tags_income.get_mut(tag).unwrap() = tx_amount + previous_value;
                    } else {
                        self.tags_income.insert(tag.to_string(), tx_amount);
                    }
                }

                if tx_amount > biggest_income.0 {
                    biggest_income = (tx_amount, tx_method.to_string());
                }
            } else if tx_type == "Expense" {
                if tx_amount > self.biggest_expense.0 {
                    self.biggest_expense = (tx_amount, tx_method.to_string());
                    total_expense += tx_amount;
                }

                for tag in tx_tags {
                    if self.tags_expense.contains_key(tag) {
                        let previous_value = self.tags_expense[tag];
                        *self.tags_expense.get_mut(tag).unwrap() = tx_amount + previous_value;
                    } else {
                        self.tags_expense.insert(tag.to_string(), tx_amount);
                    }
                }

                if tx_amount > biggest_expense.0 {
                    biggest_expense = (tx_amount, tx_method.to_string());
                }
            }
        }

        if total_income > self.income_date.0 {
            self.income_date = (total_income, cu_month.clone(), cu_year.clone());
        }
        if total_expense > self.expense_date.0 {
            self.expense_date = (total_expense, cu_month, cu_year);
        }

        if self.biggest_income.0 > biggest_income.0 {
            self.biggest_income = biggest_income
        }

        if self.biggest_expense.0 > biggest_expense.0 {
            self.biggest_expense = biggest_expense
        }
    }
}
