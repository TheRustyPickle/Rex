use crate::db::{MONTHS, YEARS};
use crate::page_handler::IndexedData;
use crate::utility::{get_all_tx_methods, get_all_txs};
use rusqlite::Connection;
use std::collections::HashMap;
/// Contains the necessary information to construct the Summary Page highlighting
/// tag based expense and income information, biggest expense and income
pub struct SummaryData {
    all_txs: HashMap<i32, Vec<Vec<String>>>,
}

impl SummaryData {
    /// Goes through all transactions to collect data for the summary
    pub fn new(conn: &Connection) -> Self {
        let mut all_txs = HashMap::new();

        for x in 0..YEARS.len() {
            for i in 0..MONTHS.len() {
                let target_id = i as i32 + (x as i32 * 12);
                let (txs, ..) = get_all_txs(conn, i, x);
                all_txs.insert(target_id, txs);
            }
        }
        SummaryData { all_txs }
    }

    /// Iters through the given transactions to collect earning and expense data
    fn get_data(
        &self,
        txs: &Vec<Vec<String>>,
        method_earning: &mut HashMap<String, f64>,
        method_expense: &mut HashMap<String, f64>,
    ) -> (
        f64,
        f64,
        (f64, String, String),
        (f64, String, String),
        f64,
        f64,
    ) {
        let mut total_income: f64 = 0.0;
        let mut total_expense: f64 = 0.0;

        let mut biggest_earning = (0.0, String::new(), String::new());
        let mut biggest_expense = (0.0, String::new(), String::new());

        let mut monthly_earning = 0.0;
        let mut monthly_expense = 0.0;

        for tx in txs {
            let tx_date = &tx[0];
            let tx_method = &tx[2];
            let tx_amount: f64 = tx[3].parse().unwrap();
            let tx_type = &tx[4];

            match tx_type.as_str() {
                "Income" => {
                    if tx_amount > biggest_earning.0 {
                        biggest_earning = (tx_amount, tx_method.to_string(), tx_date.to_string())
                    }
                    total_income += tx_amount;
                    monthly_earning += tx_amount;

                    *method_earning.get_mut(tx_method).unwrap() += tx_amount;
                }
                "Expense" => {
                    if tx_amount > biggest_expense.0 {
                        biggest_expense = (tx_amount, tx_method.to_string(), tx_date.to_string())
                    }
                    total_expense += tx_amount;
                    monthly_expense += tx_amount;

                    *method_expense.get_mut(tx_method).unwrap() += tx_amount;
                }
                _ => {}
            }
        }

        (
            total_income,
            total_expense,
            biggest_earning,
            biggest_expense,
            monthly_earning,
            monthly_expense,
        )
    }

    /// Returns a vector that will be used to creating table in the Summary UI
    /// The vector contains tags and their income and expense data
    pub fn get_table_data(
        &self,
        mode: &IndexedData,
        month: usize,
        year: usize,
    ) -> Vec<Vec<String>> {
        let mut income_tags = HashMap::new();
        let mut expense_tags = HashMap::new();
        match mode.index {
            // * 0 = monthly mode. Select the data only of the given month year
            0 => {
                let target_id = month as i32 + (year as i32 * 12);

                for tx_data in self.all_txs[&target_id].iter() {
                    let tx_amount: f64 = tx_data[3].parse().unwrap();
                    let tx_type = &tx_data[4];
                    let tx_tags = tx_data[5].split(", ").collect::<Vec<&str>>();

                    // gather data by loop through each tx. If tag exists, add with the value, if not insert it
                    match tx_type.as_str() {
                        "Income" => {
                            for tag in tx_tags {
                                if income_tags.contains_key(tag) {
                                    *income_tags.get_mut(tag).unwrap() += tx_amount;
                                } else {
                                    income_tags.insert(tag, tx_amount);
                                }
                            }
                        }
                        "Expense" => {
                            for tag in tx_tags {
                                if expense_tags.contains_key(tag) {
                                    *expense_tags.get_mut(tag).unwrap() += tx_amount;
                                } else {
                                    expense_tags.insert(tag, tx_amount);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            // * 1 = yearly mode. Select the data of all months of the given year
            1 => {
                for i in 0..MONTHS.len() {
                    let target_id = i as i32 + (year as i32 * 12);

                    for tx_data in self.all_txs[&target_id].iter() {
                        let tx_amount: f64 = tx_data[3].parse().unwrap();
                        let tx_type = &tx_data[4];
                        let tx_tags = tx_data[5].split(", ").collect::<Vec<&str>>();

                        // gather data by loop through each tx. If tag exists, add with the value, if not insert it
                        match tx_type.as_str() {
                            "Income" => {
                                for tag in tx_tags {
                                    if income_tags.contains_key(tag) {
                                        *income_tags.get_mut(tag).unwrap() += tx_amount;
                                    } else {
                                        income_tags.insert(tag, tx_amount);
                                    }
                                }
                            }
                            "Expense" => {
                                for tag in tx_tags {
                                    if expense_tags.contains_key(tag) {
                                        *expense_tags.get_mut(tag).unwrap() += tx_amount;
                                    } else {
                                        expense_tags.insert(tag, tx_amount);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            //  * 2 = all time mode. Select every single data
            2 => {
                for x in 0..YEARS.len() {
                    for i in 0..MONTHS.len() {
                        let target_id = i as i32 + (x as i32 * 12);

                        for tx_data in self.all_txs[&target_id].iter() {
                            let tx_amount: f64 = tx_data[3].parse().unwrap();
                            let tx_type = &tx_data[4];
                            let tx_tags = tx_data[5].split(", ").collect::<Vec<&str>>();

                            // gather data by loop through each tx. If tag exists, add with the value, if not insert it
                            match tx_type.as_str() {
                                "Income" => {
                                    for tag in tx_tags {
                                        if income_tags.contains_key(tag) {
                                            *income_tags.get_mut(tag).unwrap() += tx_amount;
                                        } else {
                                            income_tags.insert(tag, tx_amount);
                                        }
                                    }
                                }
                                "Expense" => {
                                    for tag in tx_tags {
                                        if expense_tags.contains_key(tag) {
                                            *expense_tags.get_mut(tag).unwrap() += tx_amount;
                                        } else {
                                            expense_tags.insert(tag, tx_amount);
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        let mut table_data = self.generate_table_data(income_tags, expense_tags);
        table_data.sort();
        table_data
    }

    /// Returns a vector that will be used to highlight points such as largest transaction,
    /// biggest income etc
    pub fn get_tx_data(
        &self,
        mode: &IndexedData,
        month: usize,
        year: usize,
        conn: &Connection,
    ) -> (
        Vec<Vec<String>>,
        Vec<Vec<String>>,
        Vec<Vec<String>>,
        Vec<Vec<String>>,
        Vec<Vec<String>>,
    ) {
        let all_methods = get_all_tx_methods(conn);
        let mut total_income: f64 = 0.0;
        let mut total_expense: f64 = 0.0;

        // (Amount, Method, date)
        let mut biggest_earning = (0.0, String::from("-"), String::from("-"));
        let mut biggest_expense = (0.0, String::from("-"), String::from("-"));

        let mut largest_monthly_earning = 0.0;
        let mut largest_monthly_expense = 0.0;

        // (Amount, Method, date)
        let mut peak_earning = (0.0, String::from("-"));
        let mut peak_expense = (0.0, String::from("-"));
        let mut total_month_checked = 0.0;

        // {Method Name, Amount}
        let mut method_earning = HashMap::new();
        let mut method_expense = HashMap::new();

        for method in &all_methods {
            method_earning.insert(method.to_string(), 0.0);
            method_expense.insert(method.to_string(), 0.0);
        }

        match mode.index {
            0 => {
                let target_id = month as i32 + (year as i32 * 12);
                let tx_data = &self.all_txs[&target_id];
                if !tx_data.is_empty() {
                    total_month_checked += 1.0;
                }

                self.update_tx_data(
                    tx_data,
                    &mut total_income,
                    &mut total_expense,
                    &mut biggest_earning,
                    &mut biggest_expense,
                    &mut largest_monthly_earning,
                    &mut largest_monthly_expense,
                    &mut peak_earning,
                    &mut peak_expense,
                    &mut method_earning,
                    &mut method_expense,
                    month,
                    year,
                )
            }
            1 => {
                for i in 0..MONTHS.len() {
                    let target_id = i as i32 + (year as i32 * 12);
                    let tx_data = &self.all_txs[&target_id];
                    if !tx_data.is_empty() {
                        total_month_checked += 1.0;
                    }

                    self.update_tx_data(
                        tx_data,
                        &mut total_income,
                        &mut total_expense,
                        &mut biggest_earning,
                        &mut biggest_expense,
                        &mut largest_monthly_earning,
                        &mut largest_monthly_expense,
                        &mut peak_earning,
                        &mut peak_expense,
                        &mut method_earning,
                        &mut method_expense,
                        i,
                        year,
                    )
                }
            }
            2 => {
                for x in 0..YEARS.len() {
                    for i in 0..MONTHS.len() {
                        let target_id = i as i32 + (x as i32 * 12);
                        let tx_data = &self.all_txs[&target_id];
                        if !tx_data.is_empty() {
                            total_month_checked += 1.0;
                        }

                        self.update_tx_data(
                            tx_data,
                            &mut total_income,
                            &mut total_expense,
                            &mut biggest_earning,
                            &mut biggest_expense,
                            &mut largest_monthly_earning,
                            &mut largest_monthly_expense,
                            &mut peak_earning,
                            &mut peak_expense,
                            &mut method_earning,
                            &mut method_expense,
                            i,
                            x,
                        )
                    }
                }
            }
            _ => {}
        }

        let (income_percentage, expense_percentage) =
            self.get_percentages(total_income, total_expense);

        let average_income = if total_income != 0.0 {
            total_income / total_month_checked
        } else {
            0.0
        };

        let average_expense = if total_income != 0.0 {
            total_expense / total_month_checked
        } else {
            0.0
        };

        let mut method_data = Vec::new();

        for method in all_methods.iter() {
            let earning_percentage = if method_earning[method] != 0.0 {
                format!("{:.2}%", (method_earning[method] / total_income) * 100.0)
            } else {
                format!("{:.2}", 0.0)
            };

            let expense_percentage = if method_expense[method] != 0.0 {
                format!("{:.2}%", (method_expense[method] / total_expense) * 100.0)
            } else {
                format!("{:.2}", 0.0)
            };

            let average_earning = if method_earning[method] != 0.0 {
                format!("{:.2}", method_earning[method] / total_month_checked)
            } else {
                format!("{:.2}", 0.0)
            };

            let average_expense = if method_expense[method] != 0.0 {
                format!("{:.2}", method_expense[method] / total_month_checked)
            } else {
                format!("{:.2}", 0.0)
            };
            method_data.push(vec![
                method.to_string(),
                format!("{:.2}", method_earning[method]),
                format!("{:.2}", method_expense[method]),
                earning_percentage,
                expense_percentage,
                average_earning,
                average_expense,
            ])
        }

        let summary_data_1 = vec![
            vec![
                String::from("Total Income"),
                format!("{:.2}", total_income),
                income_percentage,
            ],
            vec![
                String::from("Total Expense"),
                format!("{:.2}", total_expense),
                expense_percentage,
            ],
            vec![
                String::from("Net"),
                format!("{:.2}", total_income - total_expense),
                String::from("-"),
            ],
        ];

        let summary_data_2 = vec![
            vec![
                String::from("Average Income"),
                format!("{:.2}", average_income),
                String::from("-"),
            ],
            vec![
                String::from("Average Expense"),
                format!("{:.2}", average_expense),
                String::from("-"),
            ],
        ];

        let summary_data_3 = vec![
            vec![
                String::from("Largest Income"),
                biggest_earning.2,
                format!("{:.2}", biggest_earning.0),
                biggest_earning.1,
            ],
            vec![
                String::from("Largest Expense"),
                biggest_expense.2,
                format!("{:.2}", biggest_expense.0),
                biggest_expense.1,
            ],
            vec![
                String::from("Months Checked"),
                total_month_checked.to_string(),
                String::from("-"),
                String::from("-"),
            ],
        ];

        let summary_data_4 = vec![
            vec![
                String::from("Peak Earning"),
                peak_earning.1,
                format!("{:.2}", peak_earning.0),
                String::from("-"),
            ],
            vec![
                String::from("Peak Expense"),
                peak_expense.1,
                format!("{:.2}", peak_expense.0),
                String::from("-"),
            ],
        ];

        (
            summary_data_1,
            summary_data_2,
            summary_data_3,
            summary_data_4,
            method_data,
        )
    }

    /// Updates values based on the gathered data
    fn update_tx_data(
        &self,
        tx_data: &Vec<Vec<String>>,
        total_income: &mut f64,
        total_expense: &mut f64,
        biggest_earning: &mut (f64, String, String),
        biggest_expense: &mut (f64, String, String),
        largest_monthly_earning: &mut f64,
        largest_monthly_expense: &mut f64,
        peak_earning: &mut (f64, String),
        peak_expense: &mut (f64, String),
        method_earning: &mut HashMap<String, f64>,
        method_expense: &mut HashMap<String, f64>,
        month: usize,
        year: usize,
    ) {
        // gather all data from the given tx data
        let (
            current_total_income,
            current_total_expense,
            current_biggest_earning,
            current_biggest_expense,
            current_monthly_earning,
            current_monthly_expense,
        ) = self.get_data(tx_data, method_earning, method_expense);

        *total_income += current_total_income;
        *total_expense += current_total_expense;

        if current_biggest_earning.0 > biggest_earning.0 {
            *biggest_earning = current_biggest_earning;
        }

        if current_biggest_expense.0 > biggest_expense.0 {
            *biggest_expense = current_biggest_expense;
        }

        if current_monthly_earning > *largest_monthly_earning {
            *largest_monthly_earning = current_monthly_earning;
            *peak_earning = (
                *largest_monthly_earning,
                format!("{}-{}", month + 1, YEARS[year]),
            );
        }

        if current_monthly_expense > *largest_monthly_expense {
            *largest_monthly_expense = current_monthly_expense;
            *peak_expense = (
                *largest_monthly_expense,
                format!("{}-{}", month + 1, YEARS[year]),
            );
        }
    }

    /// Generates a vector to be used as table data from tag list
    fn generate_table_data(
        &self,
        income_tags: HashMap<&str, f64>,
        expense_tags: HashMap<&str, f64>,
    ) -> Vec<Vec<String>> {
        let mut to_return = Vec::new();
        let mut total_income = 0.0;
        let mut total_expense = 0.0;

        for (key, value) in income_tags.iter() {
            let mut to_push = vec![key.to_string(), format!("{:.2}", value)];
            total_income += value;

            // if the same tag already exists on expense, get that value as well
            if expense_tags.contains_key(key) {
                to_push.push(format!("{:.2}", expense_tags[key]));
                total_expense += expense_tags[key];
            } else {
                to_push.push(format!("{:.2}", 0.0))
            }
            to_return.push(to_push);
        }

        for (key, value) in expense_tags.iter() {
            // gather data only from the tags that didn't exist on Income tag list
            if !income_tags.contains_key(key) {
                to_return.push(vec![
                    key.to_string(),
                    format!("{:.2}", 0.0),
                    format!("{:.2}", value),
                ]);
                total_expense += value;
            }
        }
        // we got the income and expense data earlier. Now need to loop again
        // to gather the % data
        for x in to_return.iter_mut() {
            let income_percentage = if &x[1] != "0.00" {
                let income = &x[1].parse::<f64>().unwrap();
                format!("{:.2}", ((income / total_income) * 100.0))
            } else {
                format!("{:.2}", 0.0)
            };

            let expense_percentage = if &x[2] != "0.00" {
                let expense = &x[2].parse::<f64>().unwrap();
                format!("{:.2}", ((expense / total_expense) * 100.0))
            } else {
                format!("{:.2}", 0.0)
            };

            x.push(income_percentage);
            x.push(expense_percentage);
        }

        to_return
    }

    /// Takes 2 numbers and returns how much % are each of them
    fn get_percentages(&self, value1: f64, value2: f64) -> (String, String) {
        if value1 == 0.0 && value2 == 0.0 {
            return (String::from("0.00"), String::from("0.00"));
        }
        let total = value1 + value2;
        let percentage1 = (value1 / total) * 100.0;
        let percentage2 = (value2 / total) * 100.0;
        (
            format!("{:.2}%", percentage1),
            format!("{:.2}%", percentage2),
        )
    }
}
