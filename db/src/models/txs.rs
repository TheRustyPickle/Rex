use chrono::{Datelike, Days, Months, NaiveDate, NaiveDateTime, NaiveTime};
use diesel::dsl::{count_star, exists, sql};
use diesel::prelude::*;
use diesel::result::Error;
use diesel::sql_types::{Integer, Text};
use rex_shared::models::{Cent, LAST_POSSIBLE_TIME};
use std::collections::HashMap;

use crate::ConnCache;
use crate::models::{AmountNature, DateNature, FetchNature, Tag, TxMethod, TxTag, TxType};
use crate::schema::{tx_tags, txs};

pub static EMPTY: Vec<i32> = Vec::new();

pub struct NewSearch<'a> {
    pub date: Option<DateNature>,
    pub details: Option<&'a str>,
    pub tx_type: Option<&'a str>,
    pub from_method: Option<i32>,
    pub to_method: Option<i32>,
    pub amount: Option<AmountNature>,
    pub tags: Option<Vec<i32>>,
}

impl<'a> NewSearch<'a> {
    #[must_use]
    pub fn new(
        date: Option<DateNature>,
        details: Option<&'a str>,
        tx_type: Option<&'a str>,
        from_method: Option<i32>,
        to_method: Option<i32>,
        amount: Option<AmountNature>,
        tags: Option<Vec<i32>>,
    ) -> Self {
        Self {
            date,
            details,
            tx_type,
            from_method,
            to_method,
            amount,
            tags,
        }
    }

    pub fn search_txs(&self, db_conn: &mut impl ConnCache) -> Result<Vec<FullTx>, Error> {
        use crate::schema::txs::dsl::{
            amount, date, details, from_method, id, to_method, tx_type, txs,
        };

        let mut query = txs.into_boxed();

        if let Some(d) = self.date.as_ref() {
            match d {
                DateNature::Exact(d) => {
                    let start_date = NaiveDate::from_ymd_opt(d.year(), d.month(), d.day()).unwrap();
                    let end_date = start_date;

                    let start_date = start_date.and_time(NaiveTime::MIN);
                    let end_date = end_date.and_time(LAST_POSSIBLE_TIME);

                    query = query.filter(date.between(start_date, end_date));
                }
                DateNature::ByMonth {
                    start_date,
                    end_date,
                }
                | DateNature::ByYear {
                    start_date,
                    end_date,
                } => {
                    query = query.filter(date.between(start_date, end_date));
                }
            }
        }

        if let Some(d) = self.details {
            query = query.filter(details.like(format!("%{d}%")));
        }

        if let Some(t) = self.tx_type {
            query = query.filter(tx_type.eq(t));
        }

        if let Some(m) = self.from_method {
            query = query.filter(from_method.eq(m));
        }

        if let Some(m) = self.to_method {
            query = query.filter(to_method.eq(m));
        }

        if let Some(a) = self.amount.as_ref() {
            match a {
                AmountNature::Exact(a) => {
                    query = query.filter(amount.eq(a.value()));
                }
                AmountNature::MoreThan(a) => {
                    query = query.filter(amount.gt(a.value()));
                }
                AmountNature::MoreThanEqual(a) => {
                    query = query.filter(amount.ge(a.value()));
                }
                AmountNature::LessThan(a) => {
                    query = query.filter(amount.lt(a.value()));
                }
                AmountNature::LessThanEqual(a) => {
                    query = query.filter(amount.le(a.value()));
                }
            }
        }

        if let Some(tag_ids) = self.tags.as_ref()
            && !tag_ids.is_empty()
        {
            let subquery = tx_tags::table
                .filter(tx_tags::tag_id.eq_any(tag_ids))
                .group_by(tx_tags::tx_id)
                .select((tx_tags::tx_id, count_star()))
                .having(count_star().eq(tag_ids.len() as i64));

            query = query.filter(id.eq_any(subquery.select(tx_tags::tx_id)));
        }

        let result = query.select(Tx::as_select()).load(db_conn.conn())?;

        FullTx::convert_to_full_tx(result, db_conn)
    }
}

#[derive(Clone, Debug)]
pub struct FullTx {
    pub id: i32,
    pub date: NaiveDateTime,
    pub details: Option<String>,
    pub from_method: TxMethod,
    pub to_method: Option<TxMethod>,
    pub amount: Cent,
    pub tx_type: TxType,
    pub tags: Vec<Tag>,
    pub display_order: i32,
}

#[derive(Clone, Queryable, Selectable, Insertable)]
pub struct Tx {
    pub id: i32,
    date: NaiveDateTime,
    details: Option<String>,
    pub from_method: i32,
    pub to_method: Option<i32>,
    pub amount: i64,
    pub tx_type: String,
    display_order: i32,
}

#[derive(Clone, Insertable)]
#[diesel(table_name = txs)]
pub struct NewTx<'a> {
    pub date: NaiveDateTime,
    pub details: Option<&'a str>,
    pub from_method: i32,
    pub to_method: Option<i32>,
    pub amount: i64,
    pub tx_type: &'a str,
}

impl<'a> NewTx<'a> {
    #[must_use]
    pub fn new(
        date: NaiveDateTime,
        details: Option<&'a str>,
        from_method: i32,
        to_method: Option<i32>,
        amount: i64,
        tx_type: &'a str,
    ) -> Self {
        NewTx {
            date,
            details,
            from_method,
            to_method,
            amount,
            tx_type,
        }
    }

    pub fn insert(self, db_conn: &mut impl ConnCache) -> Result<Tx, Error> {
        use crate::schema::txs::dsl::txs;

        diesel::insert_into(txs)
            .values(self)
            .returning(Tx::as_returning())
            .get_result(db_conn.conn())
    }
}

impl FullTx {
    pub fn get_txs(
        d: NaiveDate,
        nature: FetchNature,
        db_conn: &mut impl ConnCache,
    ) -> Result<Vec<Self>, Error> {
        let all_txs = Tx::get_txs(d, nature, db_conn)?;

        FullTx::convert_to_full_tx(all_txs, db_conn)
    }

    pub fn get_tx_by_id(id_num: i32, db_conn: &mut impl ConnCache) -> Result<Self, Error> {
        let tx = Tx::get_tx_by_id(id_num, db_conn)?;

        Ok(FullTx::convert_to_full_tx(vec![tx], db_conn)?
            .pop()
            .unwrap())
    }

    pub fn convert_to_full_tx(
        txs: Vec<Tx>,
        db_conn: &mut impl ConnCache,
    ) -> Result<Vec<FullTx>, Error> {
        let tx_ids = txs.iter().map(|t| t.id).collect::<Vec<i32>>();

        let tx_tags = TxTag::get_by_tx_ids(tx_ids, db_conn)?;

        let mut tx_tags_map = HashMap::new();

        for tag in tx_tags {
            tx_tags_map
                .entry(tag.tx_id)
                .or_insert(Vec::new())
                .push(tag.tag_id);
        }

        let mut to_return = Vec::new();

        for tx in txs {
            let tags: Vec<Tag> = {
                let tag_ids = tx_tags_map.get(&tx.id).unwrap_or(&EMPTY);
                let mut v = Vec::with_capacity(tag_ids.len());
                for tag_id in tag_ids {
                    v.push(db_conn.cache().tags.get(tag_id).unwrap().clone());
                }
                v
            };

            let full_tx = FullTx {
                id: tx.id,
                date: tx.date,
                details: tx.details,
                from_method: db_conn
                    .cache()
                    .tx_methods
                    .get(&tx.from_method)
                    .unwrap()
                    .clone(),
                to_method: tx
                    .to_method
                    .map(|method_id| db_conn.cache().tx_methods.get(&method_id).unwrap().clone()),
                amount: Cent::new(tx.amount),
                tx_type: tx.tx_type.as_str().into(),
                tags,
                display_order: tx.display_order,
            };

            to_return.push(full_tx);
        }

        Ok(to_return)
    }

    pub fn get_changes(&self, db_conn: &impl ConnCache) -> HashMap<i32, String> {
        let mut map = HashMap::new();

        for method_id in db_conn.cache().tx_methods.keys() {
            let mut no_impact = true;

            if self.from_method.id == *method_id {
                no_impact = false;
            }

            if let Some(to_method) = &self.to_method
                && to_method.id == *method_id
            {
                no_impact = false;
            }

            if no_impact {
                map.insert(*method_id, "0.00".to_string());
                continue;
            }

            match self.tx_type {
                TxType::Income | TxType::Borrow | TxType::LendRepay => {
                    map.insert(*method_id, format!("↑{:.2}", self.amount.dollar()));
                }
                TxType::Expense | TxType::Lend | TxType::BorrowRepay => {
                    map.insert(*method_id, format!("↓{:.2}", self.amount.dollar()));
                }
                TxType::Transfer => {
                    if self.from_method.id == *method_id {
                        map.insert(*method_id, format!("↓{:.2}", self.amount.dollar()));
                    } else {
                        map.insert(*method_id, format!("↑{:.2}", self.amount.dollar()));
                    }
                }
            }
        }

        map
    }

    pub fn empty_changes(db_conn: &impl ConnCache) -> HashMap<i32, String> {
        let mut map = HashMap::new();

        for method_id in db_conn.cache().tx_methods.keys() {
            map.insert(*method_id, "0.00".to_string());
        }

        map
    }

    pub fn get_changes_partial(
        from_method: i32,
        to_method: Option<i32>,
        tx_type: TxType,
        amount: Cent,
        db_conn: &impl ConnCache,
    ) -> HashMap<i32, String> {
        let mut map = HashMap::new();

        for method_id in db_conn.cache().tx_methods.keys() {
            let mut no_impact = true;

            if from_method == *method_id {
                no_impact = false;
            }

            if let Some(to_method) = &to_method
                && to_method == method_id
            {
                no_impact = false;
            }

            if no_impact {
                map.insert(*method_id, "0.00".to_string());
                continue;
            }

            match tx_type {
                TxType::Income | TxType::Borrow | TxType::LendRepay => {
                    map.insert(*method_id, format!("↑{:.2}", amount.dollar()));
                }
                TxType::Expense | TxType::Lend | TxType::BorrowRepay => {
                    map.insert(*method_id, format!("↓{:.2}", amount.dollar()));
                }
                TxType::Transfer => {
                    if from_method == *method_id {
                        map.insert(*method_id, format!("↓{:.2}", amount.dollar()));
                    } else {
                        map.insert(*method_id, format!("↑{:.2}", amount.dollar()));
                    }
                }
            }
        }

        map
    }

    #[must_use]
    pub fn to_array(&self, is_search: bool) -> Vec<String> {
        let mut method = self.from_method.name.clone();

        if let Some(to_method) = &self.to_method {
            method = format!("{} → {}", self.from_method.name, to_method.name);
        }

        let date = if is_search {
            self.date.format("%Y-%m-%d").to_string()
        } else {
            self.date.format("%a %d %I:%M %p").to_string()
        };

        vec![
            date,
            self.details.clone().unwrap_or_default(),
            method,
            format!("{:.2}", self.amount.dollar()),
            self.tx_type.to_string(),
            self.tags
                .iter()
                .map(|t| t.name.clone())
                .collect::<Vec<String>>()
                .join(", "),
        ]
    }

    pub fn set_display_order(&self, db_conn: &mut impl ConnCache) -> Result<usize, Error> {
        use crate::schema::txs::dsl::{display_order, id, txs};

        diesel::update(txs.filter(id.eq(self.id)))
            .set(display_order.eq(self.display_order))
            .execute(db_conn.conn())
    }
}

impl Tx {
    pub fn insert(self, db_conn: &mut impl ConnCache) -> Result<Self, Error> {
        use crate::schema::txs::dsl::txs;

        diesel::insert_into(txs)
            .values(self)
            .returning(Tx::as_returning())
            .get_result(db_conn.conn())
    }

    pub fn get_tx_by_id(id_num: i32, db_conn: &mut impl ConnCache) -> Result<Self, Error> {
        use crate::schema::txs::dsl::{id, txs};

        txs.filter(id.eq(id_num))
            .select(Self::as_select())
            .first(db_conn.conn())
    }

    pub fn get_txs(
        d: NaiveDate,
        nature: FetchNature,
        db_conn: &mut impl ConnCache,
    ) -> Result<Vec<Self>, Error> {
        let d = d.and_time(NaiveTime::MIN);

        use crate::schema::txs::dsl::{date, display_order, id, txs};

        let dates = match nature {
            FetchNature::Monthly => {
                let start_date = NaiveDate::from_ymd_opt(d.year(), d.month(), 1).unwrap();

                let end_date = start_date + Months::new(1) - Days::new(1);

                let start_date = start_date.and_time(NaiveTime::MIN);
                let end_date = end_date.and_time(LAST_POSSIBLE_TIME);

                Some((start_date, end_date))
            }
            FetchNature::Yearly => {
                let start_date = NaiveDate::from_ymd_opt(d.year(), 1, 1).unwrap();

                let end_date = start_date + Months::new(12) - Days::new(1);

                let start_date = start_date.and_time(NaiveTime::MIN);
                let end_date = end_date.and_time(LAST_POSSIBLE_TIME);

                Some((start_date, end_date))
            }
            FetchNature::All => None,
        };

        let mut query = txs.into_boxed();

        if let Some((start_date, end_date)) = dates {
            query = query.filter(date.ge(start_date)).filter(date.le(end_date));
        }

        query
            .order((
                sql::<Text>("DATE(date) ASC"),
                sql::<Integer>("CASE WHEN display_order = 0 THEN 1 ELSE 0 END ASC"),
                display_order.asc(),
                id.asc(),
            ))
            .select(Tx::as_select())
            .load(db_conn.conn())
    }

    pub fn delete_tx(id: i32, db_conn: &mut impl ConnCache) -> Result<usize, Error> {
        use crate::schema::txs::dsl::txs;

        diesel::delete(txs.find(id)).execute(db_conn.conn())
    }

    #[must_use]
    pub fn from_new_tx(new_tx: NewTx, id: i32) -> Self {
        Self {
            id,
            date: new_tx.date,
            details: new_tx.details.map(std::string::ToString::to_string),
            from_method: new_tx.from_method,
            to_method: new_tx.to_method,
            amount: new_tx.amount,
            tx_type: new_tx.tx_type.to_string(),
            display_order: 0,
        }
    }

    pub fn get_all_details(db_conn: &mut impl ConnCache) -> Result<Vec<String>, Error> {
        use crate::schema::txs::dsl::{details, txs};

        let result: Vec<Option<String>> = txs
            .select(details)
            .filter(details.is_not_null())
            .load(db_conn.conn())?;

        Ok(result.into_iter().flatten().collect())
    }
}
