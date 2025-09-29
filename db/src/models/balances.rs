use chrono::{Datelike, Months, NaiveDate};
use diesel::prelude::*;
use diesel::result::Error;
use diesel::upsert::excluded;
use shared::models::Cent;
use std::collections::{HashMap, HashSet};

use crate::ConnCache;
use crate::models::FetchNature;
use crate::schema::balances;

#[derive(Clone, Debug, Queryable, Insertable, Selectable)]
pub struct Balance {
    pub method_id: i32,
    pub year: i32,
    pub month: i32,
    pub balance: i64,
    pub is_final_balance: bool,
}

impl Balance {
    #[must_use]
    pub fn new(
        method_id: i32,
        year: i32,
        month: i32,
        balance: i64,
        is_final_balance: bool,
    ) -> Self {
        Balance {
            method_id,
            year,
            month,
            balance,
            is_final_balance,
        }
    }

    pub fn insert(&self, db_conn: &mut impl ConnCache) -> Result<usize, Error> {
        use crate::schema::balances::dsl::{balance, balances, method_id, month, year};

        diesel::insert_into(balances)
            .values(self)
            .on_conflict((method_id, year, month))
            .do_update()
            .set(balance.eq(excluded(balance)))
            .execute(db_conn.conn())
    }

    pub fn insert_batch_final_balance(
        txs: Vec<Balance>,
        db_conn: &mut impl ConnCache,
    ) -> Result<usize, Error> {
        use crate::schema::balances::dsl::balances;

        diesel::insert_into(balances)
            .values(txs)
            .execute(db_conn.conn())
    }

    pub fn update_final_balance(&self, db_conn: &mut impl ConnCache) -> Result<usize, Error> {
        use crate::schema::balances::dsl::{balance, balances, is_final_balance, method_id};

        diesel::update(
            balances
                .filter(method_id.eq(self.method_id))
                .filter(is_final_balance.eq(true)),
        )
        .set(balance.eq(self.balance))
        .execute(db_conn.conn())
    }

    pub fn get_balance_map(
        date: NaiveDate,
        db_conn: &mut impl ConnCache,
    ) -> Result<HashMap<i32, Self>, Error> {
        use crate::schema::balances::dsl::{balances, is_final_balance, method_id, month, year};

        let date_year = date.year();
        let date_month = date.month() as i32;

        let tx_method_ids: Vec<i32> = db_conn.cache().tx_methods.keys().copied().collect();

        let balance_list = balances
            .filter(year.eq(date_year))
            .filter(month.eq(date_month))
            .filter(method_id.eq_any(tx_method_ids))
            .filter(is_final_balance.eq(false))
            .select(Self::as_select())
            .load(db_conn.conn())?;

        let mut balance_map: HashMap<i32, Self> =
            balance_list.into_iter().map(|b| (b.method_id, b)).collect();

        if balance_map.len() == db_conn.cache().tx_methods.len() {
            return Ok(balance_map);
        }

        for bal in db_conn.cache().tx_methods.values() {
            balance_map
                .entry(bal.id)
                .or_insert_with(|| Balance::new(bal.id, date_year, date_month, 0, false));
        }

        Ok(balance_map)
    }

    pub fn get_balance(
        date: NaiveDate,
        nature: FetchNature,
        db_conn: &mut impl ConnCache,
    ) -> Result<Vec<Self>, Error> {
        use crate::schema::balances::dsl::{balances, is_final_balance, method_id, month, year};

        let mut pending_balance_tx_methods = HashSet::new();

        for key in db_conn.cache().tx_methods.keys().copied() {
            pending_balance_tx_methods.insert(key);
        }

        let date_year = date.year();
        let mut date_month = date.month() as i32;

        if let FetchNature::Yearly = nature {
            date_month = 1;
        }

        let tx_method_ids: Vec<i32> = db_conn.cache().tx_methods.keys().copied().collect();

        if let FetchNature::All = nature {
            let mut balance_list = Vec::new();

            for m_id in pending_balance_tx_methods {
                let new_bal = Balance::new(m_id, date_year, date_month, 0, false);
                balance_list.push(new_bal);
            }

            return Ok(balance_list);
        }

        let mut balance_list = balances
            .filter(year.eq(date_year))
            .filter(month.eq(date_month))
            .filter(method_id.eq_any(tx_method_ids))
            .filter(is_final_balance.eq(false))
            .select(Self::as_select())
            .load(db_conn.conn())?;

        if balance_list.len() == pending_balance_tx_methods.len() {
            return Ok(balance_list);
        }

        for bal in &balance_list {
            pending_balance_tx_methods.remove(&bal.method_id);
        }

        for m_id in pending_balance_tx_methods {
            let new_bal = Balance::new(m_id, date_year, date_month, 0, false);
            balance_list.push(new_bal);
        }

        Ok(balance_list)
    }

    /// Get the last non-final balance of a method. Date = Current date
    pub fn get_last_balance(
        date: NaiveDate,
        nature: FetchNature,
        db_conn: &mut impl ConnCache,
    ) -> Result<HashMap<i32, Cent>, Error> {
        use crate::schema::balances::dsl::{balances, is_final_balance, method_id, month, year};

        let mut ongoing_date = date;

        if let FetchNature::Yearly = nature {
            ongoing_date = NaiveDate::from_ymd_opt(date.year(), 1, 1).unwrap();
        }

        let mut found_method_balances = HashMap::new();

        let mut pending_balance_tx_methods = HashSet::new();

        // All means all txs were fetched. The last balance is the balance before the first tx
        // which is 0
        if let FetchNature::All = nature {
            let mut to_return = HashMap::new();

            for key in db_conn.cache().tx_methods.keys().copied() {
                to_return.insert(key, Cent::new(0));
            }

            return Ok(to_return);
        }

        for key in db_conn.cache().tx_methods.keys().copied() {
            pending_balance_tx_methods.insert(key);
        }

        for _ in 0..3 {
            ongoing_date = ongoing_date - Months::new(1);

            let date_year = ongoing_date.year();
            let date_month = ongoing_date.month() as i32;

            let Some(last_balances) = balances
                .filter(year.eq(date_year))
                .filter(month.eq(date_month))
                .filter(method_id.eq_any(&pending_balance_tx_methods))
                .filter(is_final_balance.eq(false))
                .select(Self::as_select())
                .load(db_conn.conn())
                .optional()?
            else {
                continue;
            };

            for bal in last_balances {
                if found_method_balances.contains_key(&bal.method_id) {
                    continue;
                }

                found_method_balances.insert(bal.method_id, Cent::new(bal.balance));
                pending_balance_tx_methods.remove(&bal.method_id);
            }
        }

        // Fallback. Start from the previous month and look for the last non-final balance
        let date = date - Months::new(1);

        if !pending_balance_tx_methods.is_empty() {
            for mid in pending_balance_tx_methods {
                if let Some(last_balance) = balances
                    .filter(method_id.eq(mid))
                    .filter(is_final_balance.eq(false))
                    .filter(
                        year.lt(date.year())
                            .or(year.eq(date.year()).and(month.lt(date.month() as i32))),
                    )
                    .order((year.desc(), month.desc()))
                    .select(Self::as_select())
                    .first::<Self>(db_conn.conn())
                    .optional()?
                {
                    found_method_balances.insert(mid, Cent::new(last_balance.balance));
                }
            }
        }

        if found_method_balances.len() != db_conn.cache().tx_methods.len() {
            for method in db_conn.cache().tx_methods.keys() {
                if !found_method_balances.contains_key(method) {
                    found_method_balances.insert(*method, Cent::new(0));
                }
            }
        }

        Ok(found_method_balances)
    }

    pub fn get_final_balance(db_conn: &mut impl ConnCache) -> Result<HashMap<i32, Self>, Error> {
        use crate::schema::balances::dsl::{balances, is_final_balance};

        let balance_list = balances
            .filter(is_final_balance.eq(true))
            .select(Balance::as_select())
            .load(db_conn.conn())?;

        assert!(
            (balance_list.len() == db_conn.cache().tx_methods.len()),
            "Final balances are not set for all transaction methods"
        );

        let balance_map = balance_list.into_iter().map(|b| (b.method_id, b)).collect();

        Ok(balance_map)
    }
}
