use anyhow::Result;
use chrono::NaiveDate;
use db::ConnCache;
pub use db::models::FullTx;
use db::models::{Balance, FetchNature, TxMethod, TxType};
use std::collections::HashMap;

use crate::conn::DbConn;
use crate::fetcher::Cent;

pub struct PartialTx<'a> {
    pub from_method: &'a str,
    pub to_method: &'a str,
    pub tx_type: &'a str,
    pub amount: &'a str,
}

#[derive(Debug)]
pub struct TxView {
    tx: FullTx,
    /// Tx Method ID -> Balance after this tx was committed
    balance: HashMap<i32, Cent>,
}

pub struct TxViewGroup(Vec<TxView>);

pub(crate) fn get_txs(
    date: NaiveDate,
    nature: FetchNature,
    db_conn: &mut impl ConnCache,
) -> Result<TxViewGroup> {
    let txs = FullTx::get_txs(date, nature, db_conn)?;

    let current_balance = Balance::get_balance(date, db_conn)?;

    let last_balance = Balance::get_last_balance(date, db_conn)?;
    let mut last_balance = last_balance
        .into_iter()
        .map(|b| (b.0, Cent::new(b.1)))
        .collect::<HashMap<i32, Cent>>();

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

        if last_balance != balance.balance {
            balance.balance = last_balance.value();
            to_insert_balance.push(balance);
        }
    }

    for to_insert in to_insert_balance {
        to_insert.insert(db_conn)?;
    }

    Ok(TxViewGroup(all_tx_views))
}

impl TxView {
    fn new(tx: FullTx, balance: HashMap<i32, Cent>) -> Self {
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

        let mut total_balance = Cent::new(0);
        let mut total_income = Cent::new(0);
        let mut total_expense = Cent::new(0);
        let mut total_daily_income = Cent::new(0);
        let mut total_daily_expense = Cent::new(0);

        for method in sorted_methods {
            let method_id = method.id;

            if let Some(index) = index {
                let target_tx = &self.0[index];

                let balance = *target_tx.balance.get(&method_id).unwrap();
                total_balance += balance;

                let method_balance = balance.dollar();
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

            to_insert_income.push(format!("{:.2}", method_income.dollar()));

            let method_expense = *expense.get(&method_id).unwrap();
            total_expense += method_expense;
            to_insert_expense.push(format!("{:.2}", method_expense.dollar()));

            let method_daily_income = *daily_income.get(&method_id).unwrap();
            total_daily_income += method_daily_income;
            to_insert_daily_income.push(format!("{:.2}", method_daily_income.dollar()));

            let method_daily_expense = *daily_expense.get(&method_id).unwrap();
            total_daily_expense += method_daily_expense;
            to_insert_daily_expense.push(format!("{:.2}", method_daily_expense.dollar()));
        }

        to_insert_balance.push(format!("{:.2}", total_balance.dollar()));

        to_insert_income.push(format!("{:.2}", total_income.dollar()));
        to_insert_expense.push(format!("{:.2}", total_expense.dollar()));

        to_insert_daily_income.push(format!("{:.2}", total_daily_income.dollar()));
        to_insert_daily_expense.push(format!("{:.2}", total_daily_expense.dollar()));

        to_return.push(to_insert_balance);
        to_return.push(to_insert_changes);

        to_return.push(to_insert_income);
        to_return.push(to_insert_expense);

        to_return.push(to_insert_daily_income);
        to_return.push(to_insert_daily_expense);

        Ok(to_return)
    }

    fn get_daily_income(&self, index: Option<usize>, db_conn: &DbConn) -> HashMap<i32, Cent> {
        let mut to_return = HashMap::new();

        for method in db_conn.cache().tx_methods.keys() {
            to_return.insert(*method, Cent::new(0));
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

    fn get_daily_expense(&self, index: Option<usize>, db_conn: &DbConn) -> HashMap<i32, Cent> {
        let mut to_return = HashMap::new();

        for method in db_conn.cache().tx_methods.keys() {
            to_return.insert(*method, Cent::new(0));
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

    fn get_income(&self, index: Option<usize>, db_conn: &DbConn) -> HashMap<i32, Cent> {
        let mut to_return = HashMap::new();

        for method in db_conn.cache().tx_methods.keys() {
            to_return.insert(*method, Cent::new(0));
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

    fn get_expense(&self, index: Option<usize>, db_conn: &DbConn) -> HashMap<i32, Cent> {
        let mut to_return = HashMap::new();

        for method in db_conn.cache().tx_methods.keys() {
            to_return.insert(*method, Cent::new(0));
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

    pub fn get_tx_by_id(&self, id: i32) -> Option<&FullTx> {
        self.0
            .iter()
            .find(|tx_view| tx_view.tx.id == id)
            .map(|tx_view| &tx_view.tx)
    }

    pub fn add_tx_balance_array(
        &self,
        index: Option<usize>,
        partial_tx: Option<PartialTx>,
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

        let changes = if let Some(partial_tx) = partial_tx {
            let tx_type = partial_tx.tx_type.into();
            let amount = (partial_tx.amount.parse::<f64>()? * 100.0).round() as i64;

            let from_method = db_conn.cache().get_method_id(partial_tx.from_method)?;
            let to_method = if partial_tx.to_method.is_empty() {
                None
            } else {
                Some(db_conn.cache().get_method_id(partial_tx.to_method)?)
            };

            FullTx::get_changes_partial(from_method, to_method, tx_type, amount, db_conn)
        } else if let Some(index) = index {
            let target_tx = &self.0[index];

            target_tx.tx.get_changes(db_conn)
        } else {
            FullTx::empty_changes(db_conn)
        };

        let mut total_balance = Cent::new(0);
        let mut to_insert_balance = vec![String::from("Balance")];
        let mut to_insert_changes = vec![String::from("Changes")];

        for method in sorted_methods {
            let method_id = method.id;

            if let Some(index) = index {
                let target_tx = &self.0[index];

                let balance = *target_tx.balance.get(&method_id).unwrap();
                total_balance += balance;

                let method_balance = balance.dollar();
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
        }

        to_insert_balance.push(format!("{:.2}", total_balance.dollar()));

        to_return.push(to_insert_balance);
        to_return.push(to_insert_changes);

        Ok(to_return)
    }

    pub fn switch_tx_index(
        &mut self,
        index_1: usize,
        index_2: usize,
        db_conn: &mut impl ConnCache,
    ) -> Result<bool> {
        let tx_1 = self.0.get(index_1).unwrap();
        let tx_2 = self.0.get(index_2).unwrap();

        // Can't switch index if not in the same date
        if tx_1.tx.date != tx_2.tx.date {
            return Ok(false);
        }

        let tx_1_order = tx_1.tx.display_order;
        let tx_2_order = tx_2.tx.display_order;

        let new_tx_1_order = if tx_2_order == 0 {
            tx_2.tx.id
        } else {
            tx_2_order
        };
        let new_tx_2_order = if tx_1_order == 0 {
            tx_1.tx.id
        } else {
            tx_1_order
        };

        let tx_1 = self.0.get_mut(index_1).unwrap();
        tx_1.tx.display_order = new_tx_1_order;

        tx_1.tx.set_display_order(db_conn)?;

        let tx_2 = self.0.get_mut(index_2).unwrap();
        tx_2.tx.display_order = new_tx_2_order;

        tx_2.tx.set_display_order(db_conn)?;

        self.0.swap(index_1, index_2);

        // TODO: Activity txs

        Ok(true)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn get_tx_balance(&self, index: usize) -> &HashMap<i32, Cent> {
        &self.0[index].balance
    }
}
