use chrono::{NaiveDate, NaiveDateTime};
use diesel::prelude::*;
use diesel::result::Error;
use rex_shared::models::Cent;
use std::collections::HashMap;

use crate::ConnCache;
use crate::models::{EMPTY, RecurrenceFrequency, RecurringTxTag, Tag, TxMethod, TxType};
use crate::schema::recurring_txs;

#[derive(Clone, Queryable, Selectable, Insertable)]
pub struct RecurringTx {
    pub id: i32,
    created_at: NaiveDateTime,
    details: Option<String>,
    pub from_method: i32,
    pub to_method: Option<i32>,
    pub amount: i64,
    pub tx_type: String,
    pub frequency: String,
    pub recur_interval: i32,
    pub recur_value: Option<i32>,
    pub recur_month: Option<i32>,
    pub last_recurred_date: Option<NaiveDate>,
    pub next_recurring_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub is_paused: bool,
}

#[derive(Clone, Insertable)]
#[diesel(table_name = recurring_txs)]
pub struct NewRecurringTx<'a> {
    pub created_at: NaiveDateTime,
    pub details: Option<&'a str>,
    pub from_method: i32,
    pub to_method: Option<i32>,
    pub amount: i64,
    pub tx_type: &'a str,
    pub frequency: &'a str,
    pub recur_interval: i32,
    pub recur_value: Option<i32>,
    pub recur_month: Option<i32>,
    pub last_recurred_date: Option<NaiveDate>,
    pub next_recurring_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub is_paused: bool,
}

impl<'a> NewRecurringTx<'a> {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        created_at: NaiveDateTime,
        details: Option<&'a str>,
        from_method: i32,
        to_method: Option<i32>,
        amount: i64,
        tx_type: &'a str,
        frequency: &'a str,
        recur_interval: i32,
        recur_value: Option<i32>,
        recur_month: Option<i32>,
        next_recurring_date: NaiveDate,
        end_date: Option<NaiveDate>,
    ) -> Self {
        NewRecurringTx {
            created_at,
            details,
            from_method,
            to_method,
            amount,
            tx_type,
            frequency,
            recur_interval,
            recur_value,
            recur_month,
            last_recurred_date: None,
            next_recurring_date,
            end_date,
            is_paused: false,
        }
    }

    pub fn insert(self, db_conn: &mut impl ConnCache) -> Result<RecurringTx, Error> {
        use crate::schema::recurring_txs::dsl::recurring_txs;

        diesel::insert_into(recurring_txs)
            .values(self)
            .returning(RecurringTx::as_returning())
            .get_result(db_conn.conn())
    }
}

pub struct FullRecurringTx {
    pub id: i32,
    pub details: Option<String>,
    pub from_method: TxMethod,
    pub to_method: Option<TxMethod>,
    pub amount: Cent,
    pub tx_type: TxType,
    pub frequency: RecurrenceFrequency,
    pub recur_interval: i32,
    pub recur_value: Option<i32>,
    pub recur_month: Option<i32>,
    pub last_recurred_date: Option<NaiveDate>,
    pub next_recurring_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub is_paused: bool,
    pub tags: Vec<Tag>,
}

impl RecurringTx {
    pub fn get_all(db_conn: &mut impl ConnCache) -> Result<Vec<Self>, Error> {
        use crate::schema::recurring_txs::dsl::recurring_txs;

        recurring_txs
            .select(RecurringTx::as_select())
            .load(db_conn.conn())
    }

    pub fn get_by_id(id_num: i32, db_conn: &mut impl ConnCache) -> Result<Self, Error> {
        use crate::schema::recurring_txs::dsl::{id, recurring_txs};

        recurring_txs
            .filter(id.eq(id_num))
            .select(Self::as_select())
            .first(db_conn.conn())
    }

    /// All non-paused rules whose next occurrence is already due
    pub fn get_due(today: NaiveDate, db_conn: &mut impl ConnCache) -> Result<Vec<Self>, Error> {
        use crate::schema::recurring_txs::dsl::{is_paused, next_recurring_date, recurring_txs};

        recurring_txs
            .filter(is_paused.eq(false))
            .filter(next_recurring_date.le(today))
            .select(Self::as_select())
            .load(db_conn.conn())
    }

    pub fn update_schedule(
        id_num: i32,
        last: Option<NaiveDate>,
        next: NaiveDate,
        db_conn: &mut impl ConnCache,
    ) -> Result<usize, Error> {
        use crate::schema::recurring_txs::dsl::{
            id, last_recurred_date, next_recurring_date, recurring_txs,
        };

        diesel::update(recurring_txs.filter(id.eq(id_num)))
            .set((last_recurred_date.eq(last), next_recurring_date.eq(next)))
            .execute(db_conn.conn())
    }

    pub fn set_paused(
        id_num: i32,
        paused: bool,
        db_conn: &mut impl ConnCache,
    ) -> Result<usize, Error> {
        use crate::schema::recurring_txs::dsl::{id, is_paused, recurring_txs};

        diesel::update(recurring_txs.filter(id.eq(id_num)))
            .set(is_paused.eq(paused))
            .execute(db_conn.conn())
    }

    pub fn update(
        id_num: i32,
        new: &NewRecurringTx,
        db_conn: &mut impl ConnCache,
    ) -> Result<usize, Error> {
        use crate::schema::recurring_txs::dsl::{
            amount, details, end_date, frequency, from_method, id, last_recurred_date,
            next_recurring_date, recur_interval, recur_month, recur_value, recurring_txs,
            to_method, tx_type,
        };

        diesel::update(recurring_txs.filter(id.eq(id_num)))
            .set((
                details.eq(new.details),
                from_method.eq(new.from_method),
                to_method.eq(new.to_method),
                amount.eq(new.amount),
                tx_type.eq(new.tx_type),
                frequency.eq(new.frequency),
                recur_interval.eq(new.recur_interval),
                recur_value.eq(new.recur_value),
                recur_month.eq(new.recur_month),
                last_recurred_date.eq(new.last_recurred_date),
                next_recurring_date.eq(new.next_recurring_date),
                end_date.eq(new.end_date),
            ))
            .execute(db_conn.conn())
    }

    pub fn delete_by_id(id_num: i32, db_conn: &mut impl ConnCache) -> Result<usize, Error> {
        use crate::schema::recurring_txs::dsl::{id, recurring_txs};

        diesel::delete(recurring_txs.filter(id.eq(id_num))).execute(db_conn.conn())
    }
}

impl FullRecurringTx {
    pub fn get_all(db_conn: &mut impl ConnCache) -> Result<Vec<Self>, Error> {
        let all = RecurringTx::get_all(db_conn)?;

        FullRecurringTx::convert_to_full(all, db_conn)
    }

    pub fn get_by_id(id_num: i32, db_conn: &mut impl ConnCache) -> Result<Self, Error> {
        let recurring_tx = RecurringTx::get_by_id(id_num, db_conn)?;

        Ok(
            FullRecurringTx::convert_to_full(vec![recurring_tx], db_conn)?
                .pop()
                .unwrap(),
        )
    }

    pub fn convert_to_full(
        recurring_txs: Vec<RecurringTx>,
        db_conn: &mut impl ConnCache,
    ) -> Result<Vec<Self>, Error> {
        let ids = recurring_txs.iter().map(|t| t.id).collect::<Vec<i32>>();

        let recurring_tx_tags = RecurringTxTag::get_by_recurring_tx_ids(ids, db_conn)?;

        let mut tags_map = HashMap::new();

        for tag in recurring_tx_tags {
            tags_map
                .entry(tag.recurring_tx_id)
                .or_insert(Vec::new())
                .push(tag.tag_id);
        }

        let mut to_return = Vec::new();

        for recurring_tx in recurring_txs {
            let tags: Vec<Tag> = {
                let tag_ids = tags_map.get(&recurring_tx.id).unwrap_or(&EMPTY);
                let mut v = Vec::with_capacity(tag_ids.len());

                for tag_id in tag_ids {
                    v.push(db_conn.cache().tags.get(tag_id).unwrap().clone());
                }

                v
            };

            let full = FullRecurringTx {
                id: recurring_tx.id,
                details: recurring_tx.details,
                from_method: db_conn
                    .cache()
                    .tx_methods
                    .get(&recurring_tx.from_method)
                    .unwrap()
                    .clone(),
                to_method: recurring_tx
                    .to_method
                    .map(|method_id| db_conn.cache().tx_methods.get(&method_id).unwrap().clone()),
                amount: Cent::new(recurring_tx.amount),
                tx_type: recurring_tx.tx_type.as_str().into(),
                frequency: recurring_tx.frequency.as_str().into(),
                recur_interval: recurring_tx.recur_interval,
                recur_value: recurring_tx.recur_value,
                recur_month: recurring_tx.recur_month,
                last_recurred_date: recurring_tx.last_recurred_date,
                next_recurring_date: recurring_tx.next_recurring_date,
                end_date: recurring_tx.end_date,
                is_paused: recurring_tx.is_paused,
                tags,
            };

            to_return.push(full);
        }

        Ok(to_return)
    }

    #[must_use]
    pub fn frequency_label(&self) -> String {
        if self.recur_interval <= 1 {
            self.frequency.to_string()
        } else {
            match self.frequency {
                RecurrenceFrequency::Daily => format!("Every {} Days", self.recur_interval),
                RecurrenceFrequency::Weekly => format!("Every {} Weeks", self.recur_interval),
                RecurrenceFrequency::Monthly => format!("Every {} Months", self.recur_interval),
                RecurrenceFrequency::Yearly => format!("Every {} Years", self.recur_interval),
            }
        }
    }

    #[must_use]
    pub fn to_array(&self) -> Vec<String> {
        let mut method = self.from_method.name.clone();

        if let Some(to_method) = &self.to_method {
            method = format!("{} → {}", self.from_method.name, to_method.name);
        }

        vec![
            self.next_recurring_date.format("%Y-%m-%d").to_string(),
            self.frequency_label(),
            self.details.clone().unwrap_or_default(),
            method,
            format!("{:.2}", self.amount.dollar()),
            self.tx_type.to_string(),
            self.tags
                .iter()
                .map(|t| t.name.clone())
                .collect::<Vec<String>>()
                .join(", "),
            if self.is_paused { "Yes" } else { "No" }.to_string(),
        ]
    }
}
