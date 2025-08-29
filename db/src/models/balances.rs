use chrono::{Datelike, Months, NaiveDate};
use diesel::prelude::*;
use diesel::result::Error;
use diesel::upsert::excluded;
use std::collections::{HashMap, HashSet};

use crate::ConnCache;
use crate::schema::balances;

#[derive(Clone, Queryable, Insertable, Selectable)]
pub struct Balance {
    pub method_id: i32,
    pub year: i32,
    pub month: i32,
    pub balance: i64,
    pub is_final_balance: bool,
}

impl Balance {
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

    pub fn insert(self, db_conn: &mut impl ConnCache) -> Result<usize, Error> {
        use crate::schema::balances::dsl::*;

        diesel::insert_into(balances)
            .values(self)
            .on_conflict((method_id, year, month))
            .do_update()
            .set(balance.eq(excluded(balance)))
            .execute(db_conn.conn())
    }

    pub fn insert_conn(self, db_conn: &mut impl ConnCache) -> Result<usize, Error> {
        use crate::schema::balances::dsl::*;

        diesel::insert_into(balances)
            .values(self)
            .on_conflict((method_id, year, month))
            .do_update()
            .set(balance.eq(excluded(balance)))
            .execute(db_conn.conn())
    }

    pub fn insert_batch_final_balance(
        txs: Vec<Balance>,
        conn: &mut SqliteConnection,
    ) -> Result<usize, Error> {
        use crate::schema::balances::dsl::*;

        diesel::insert_into(balances).values(txs).execute(conn)
    }

    pub fn get_balance(date: NaiveDate, db_conn: &mut impl ConnCache) -> Result<Vec<Self>, Error> {
        use crate::schema::balances::dsl::{balances, is_final_balance, method_id, month, year};

        let mut pending_balance_tx_methods = HashSet::new();

        for key in db_conn.cache().tx_methods.keys().copied() {
            pending_balance_tx_methods.insert(key);
        }

        let date_year = date.year();
        let date_month = date.month() as i32;

        let tx_method_ids: Vec<i32> = db_conn.cache().tx_methods.keys().copied().collect();

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

    pub fn get_last_balance(
        date: NaiveDate,
        db_conn: &mut impl ConnCache,
    ) -> Result<HashMap<i32, i64>, Error> {
        use crate::schema::balances::dsl::{balances, method_id, month, year};

        let mut found_method_balances = HashMap::new();

        let mut pending_balance_tx_methods = HashSet::new();

        for key in db_conn.cache().tx_methods.keys().copied() {
            pending_balance_tx_methods.insert(key);
        }

        for _ in 0..3 {
            let previous_date = date - Months::new(1);

            let date_year = previous_date.year();
            let date_month = previous_date.month() as i32;

            let Some(last_balances) = balances
                .filter(year.eq(date_year))
                .filter(month.eq(date_month))
                .filter(method_id.eq_any(&pending_balance_tx_methods))
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

                found_method_balances.insert(bal.method_id, bal.balance);
                pending_balance_tx_methods.remove(&bal.method_id);
            }
        }

        if !pending_balance_tx_methods.is_empty() {
            for mid in pending_balance_tx_methods {
                if let Some(last_balance) = balances
                    .filter(method_id.eq(mid))
                    .filter(
                        year.lt(date.year())
                            .or(year.eq(date.year()).and(month.lt(date.month() as i32))),
                    )
                    .order((year.desc(), month.desc()))
                    .select(Self::as_select())
                    .first::<Self>(db_conn.conn())
                    .optional()?
                {
                    found_method_balances.insert(mid, last_balance.balance);
                }
            }
        }

        if found_method_balances.len() != db_conn.cache().tx_methods.len() {
            for method in db_conn.cache().tx_methods.keys() {
                if !found_method_balances.contains_key(method) {
                    found_method_balances.insert(*method, 0);
                }
            }
        }

        Ok(found_method_balances)
    }

    pub fn get_final_balance(db_conn: &mut impl ConnCache) -> Result<Vec<Balance>, Error> {
        use crate::schema::balances::dsl::{balances, is_final_balance};

        let balance_list = balances
            .filter(is_final_balance.eq(true))
            .select(Balance::as_select())
            .load(db_conn.conn())?;

        if balance_list.len() != db_conn.cache().tx_methods.len() {
            panic!("Final balances are not set for all transaction methods");
        }

        Ok(balance_list)
    }
}
