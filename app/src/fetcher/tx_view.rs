use anyhow::Result;
use chrono::NaiveDate;
pub use db::models::FullTx;
use db::models::{Balance, FetchNature, TxMethod, TxType};
use db::{ConnCache, DbConn};
use std::collections::HashMap;

pub struct TxView {
    tx: FullTx,
    /// Tx Method ID -> Balance after this tx was committed
    balance: HashMap<i32, i64>,
}

pub struct TxViewGroup(Vec<TxView>);

// Month and year are in index value. 0 for month is January while 0 for year is 2022
pub fn get_txs_index(month: usize, year: usize, db_conn: &mut DbConn) -> Result<TxViewGroup> {
    let month_num = (month + 1) as u32;
    let year_num = (year + 2022) as i32;

    let date = NaiveDate::from_ymd_opt(year_num, month_num, 1).unwrap();
    get_txs(date, db_conn)
}

pub fn get_txs_date(date: NaiveDate, db_conn: &mut DbConn) -> Result<TxViewGroup> {
    get_txs(date, db_conn)
}

fn get_txs(date: NaiveDate, db_conn: &mut DbConn) -> Result<TxViewGroup> {
    let nature = FetchNature::Monthly;

    let txs = FullTx::get_txs(date, nature, db_conn)?;

    let current_balance = Balance::get_balance(date, db_conn)?;

    let mut last_balance = Balance::get_last_balance(date, db_conn)?;

    let mut all_tx_views = Vec::with_capacity(txs.len());

    for tx in txs {
        match &tx.tx_type {
            TxType::Income => {
                let method_id = tx.from_method.id;
                *last_balance.get_mut(&method_id).unwrap() += tx.amount;
            }
            TxType::Expense => {
                let method_id = tx.from_method.id;
                *last_balance.get_mut(&method_id).unwrap() -= tx.amount;
            }

            TxType::Transfer => {
                let from_method_id = tx.from_method.id;
                let to_method_id = tx.to_method.as_ref().unwrap().id;

                *last_balance.get_mut(&from_method_id).unwrap() -= tx.amount;
                *last_balance.get_mut(&to_method_id).unwrap() += tx.amount;
            }
        }

        let tx_view = TxView::new(tx, last_balance.clone());
        all_tx_views.push(tx_view);
    }

    let mut to_insert_balance = Vec::new();

    for mut balance in current_balance {
        let method_id = balance.method_id;
        let last_balance = *last_balance.get(&method_id).unwrap();

        if balance.balance != last_balance {
            balance.balance = last_balance;
            to_insert_balance.push(balance);
        }
    }

    for to_insert in to_insert_balance {
        to_insert.insert(db_conn)?;
    }

    Ok(TxViewGroup(all_tx_views))
}

impl TxView {
    fn new(tx: FullTx, balance: HashMap<i32, i64>) -> Self {
        Self { tx, balance }
    }
}

impl TxViewGroup {
    pub fn balance_array(
        &self,
        index: Option<usize>,
        db_conn: &mut DbConn,
    ) -> Result<Vec<Vec<String>>> {
        let mut final_balance: Option<HashMap<i32, Balance>> = None;

        if index.is_none() {
            final_balance = Some(Balance::get_final_balance(db_conn)?);
        }

        let mut sorted_methods: Vec<&TxMethod> = db_conn.cache().tx_methods.values().collect();
        sorted_methods.sort_by_key(|value| value.position);

        let mut to_return = vec![vec![String::new()]];

        to_return[0].extend(sorted_methods.iter().map(|m| m.name.to_string()));

        to_return[0].push(String::from("Total"));

        let changes = if let Some(index) = index {
            let target_tx = &self.0[index];

            target_tx.tx.get_changes(db_conn)
        } else {
            FullTx::empty_changes(db_conn)
        };

        let income = self.get_income(index, db_conn);
        let expense = self.get_expense(index, db_conn);

        let daily_income = self.get_daily_income(index, db_conn);
        let daily_expense = self.get_daily_expense(index, db_conn);

        let mut to_insert_balance = vec![String::from("Balance")];
        let mut to_insert_changes = vec![String::from("Changes")];

        let mut to_insert_income = vec![String::from("Income")];
        let mut to_insert_expense = vec![String::from("Expense")];

        let mut to_insert_daily_income = vec![String::from("Daily Income")];
        let mut to_insert_daily_expense = vec![String::from("Daily Expense")];

        let mut total_balance = 0;
        let mut total_income = 0;
        let mut total_expense = 0;
        let mut total_daily_income = 0;
        let mut total_daily_expense = 0;

        for method in sorted_methods {
            let method_id = method.id;

            if let Some(index) = index {
                let target_tx = &self.0[index];

                let balance = *target_tx.balance.get(&method_id).unwrap();
                total_balance += balance;

                let method_balance = balance as f64 / 100.0;
                to_insert_balance.push(format!("{method_balance:.2}"));
            } else {
                let balance = final_balance
                    .as_ref()
                    .unwrap()
                    .get(&method_id)
                    .unwrap()
                    .balance;
                total_balance += balance;

                let method_balance = balance as f64 / 100.0;
                to_insert_balance.push(format!("{method_balance:.2}"));
            }

            let changes_value = changes.get(&method_id).unwrap();
            to_insert_changes.push(changes_value.to_string());

            let method_income = *income.get(&method_id).unwrap();
            total_income += method_income;

            to_insert_income.push(format!("{:.2}", method_income as f64 / 100.0));

            let method_expense = *expense.get(&method_id).unwrap();
            total_expense += method_expense;
            to_insert_expense.push(format!("{:.2}", method_expense as f64 / 100.0));

            let method_daily_income = *daily_income.get(&method_id).unwrap();
            total_daily_income += method_daily_income;
            to_insert_daily_income.push(format!("{:.2}", method_daily_income as f64 / 100.0));

            let method_daily_expense = *daily_expense.get(&method_id).unwrap();
            total_daily_expense += method_daily_expense;
            to_insert_daily_expense.push(format!("{:.2}", method_daily_expense as f64 / 100.0));
        }

        to_insert_balance.push(format!("{:.2}", total_balance as f64 / 100.0));

        to_insert_income.push(format!("{:.2}", total_income as f64 / 100.0));
        to_insert_expense.push(format!("{:.2}", total_expense as f64 / 100.0));

        to_insert_daily_income.push(format!("{:.2}", total_daily_income as f64 / 100.0));
        to_insert_daily_expense.push(format!("{:.2}", total_daily_expense as f64 / 100.0));

        to_return.push(to_insert_balance);
        to_return.push(to_insert_changes);

        to_return.push(to_insert_income);
        to_return.push(to_insert_expense);

        to_return.push(to_insert_daily_income);
        to_return.push(to_insert_daily_expense);

        Ok(to_return)
    }

    fn get_daily_income(&self, index: Option<usize>, db_conn: &DbConn) -> HashMap<i32, i64> {
        let mut to_return = HashMap::new();

        for method in db_conn.cache().tx_methods.keys() {
            to_return.insert(*method, 0);
        }

        let Some(index) = index else {
            return to_return;
        };

        let target_tx = &self.0[index];
        let ongoing_date = target_tx.tx.date;

        for tx in self.0.iter().take(index + 1).rev() {
            if tx.tx.date != ongoing_date {
                break;
            }

            if let TxType::Income = tx.tx.tx_type {
                let method_id = tx.tx.from_method.id;
                *to_return.get_mut(&method_id).unwrap() += tx.tx.amount;
            }
        }

        to_return
    }

    fn get_daily_expense(&self, index: Option<usize>, db_conn: &DbConn) -> HashMap<i32, i64> {
        let mut to_return = HashMap::new();

        for method in db_conn.cache().tx_methods.keys() {
            to_return.insert(*method, 0);
        }

        let Some(index) = index else {
            return to_return;
        };

        let target_tx = &self.0[index];
        let ongoing_date = target_tx.tx.date;

        for tx in self.0.iter().take(index + 1).rev() {
            if tx.tx.date != ongoing_date {
                break;
            }

            if let TxType::Expense = tx.tx.tx_type {
                let method_id = tx.tx.from_method.id;
                *to_return.get_mut(&method_id).unwrap() += tx.tx.amount;
            }
        }

        to_return
    }

    fn get_income(&self, index: Option<usize>, db_conn: &DbConn) -> HashMap<i32, i64> {
        let mut to_return = HashMap::new();

        for method in db_conn.cache().tx_methods.keys() {
            to_return.insert(*method, 0);
        }

        if let Some(index) = index {
            for tx in self.0.iter().take(index + 1).rev() {
                if let TxType::Income = tx.tx.tx_type {
                    let method_id = tx.tx.from_method.id;
                    *to_return.get_mut(&method_id).unwrap() += tx.tx.amount;
                }
            }
        } else {
            for tx in &self.0 {
                if let TxType::Income = tx.tx.tx_type {
                    let method_id = tx.tx.from_method.id;
                    *to_return.get_mut(&method_id).unwrap() += tx.tx.amount;
                }
            }
        }

        to_return
    }

    fn get_expense(&self, index: Option<usize>, db_conn: &DbConn) -> HashMap<i32, i64> {
        let mut to_return = HashMap::new();

        for method in db_conn.cache().tx_methods.keys() {
            to_return.insert(*method, 0);
        }

        if let Some(index) = index {
            for tx in self.0.iter().take(index + 1).rev() {
                if let TxType::Expense = tx.tx.tx_type {
                    let method_id = tx.tx.from_method.id;
                    *to_return.get_mut(&method_id).unwrap() += tx.tx.amount;
                }
            }
        } else {
            for tx in &self.0 {
                if let TxType::Expense = tx.tx.tx_type {
                    let method_id = tx.tx.from_method.id;
                    *to_return.get_mut(&method_id).unwrap() += tx.tx.amount;
                }
            }
        }

        to_return
    }

    pub fn tx_array(&self) -> Vec<Vec<String>> {
        self.0.iter().map(|tx_view| tx_view.tx.to_array()).collect()
    }

    pub fn get_tx(&self, index: usize) -> &FullTx {
        &self.0[index].tx
    }
}
