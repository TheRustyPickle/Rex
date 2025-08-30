use chrono::{Datelike, Days, Months, NaiveDate};
use diesel::prelude::*;
use diesel::result::Error;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use crate::ConnCache;
use crate::models::{Tag, TxMethod, TxTag};
use crate::schema::txs;

static EMPTY: Vec<i32> = Vec::new();

pub enum TxType {
    Income,
    Expense,
    Transfer,
}

pub enum FetchNature {
    Monthly,
    Yearly,
}

impl From<&str> for TxType {
    fn from(s: &str) -> Self {
        match s {
            "Income" => TxType::Income,
            "Expense" => TxType::Expense,
            "Transfer" => TxType::Transfer,
            other => panic!("Invalid TxType string: {other}"),
        }
    }
}

impl Display for TxType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TxType::Income => "Income",
            TxType::Expense => "Expense",
            TxType::Transfer => "Transfer",
        };
        write!(f, "{s}")
    }
}

pub struct FullTx {
    pub id: i32,
    pub date: NaiveDate,
    pub details: Option<String>,
    pub from_method: TxMethod,
    pub to_method: Option<TxMethod>,
    pub amount: i64,
    pub tx_type: TxType,
    pub activity_id: Option<i32>,
    pub tags: Vec<Tag>,
}

#[derive(Clone, Queryable, Selectable)]
pub struct Tx {
    pub id: i32,
    date: NaiveDate,
    details: Option<String>,
    pub from_method: i32,
    pub to_method: Option<i32>,
    pub amount: i64,
    pub tx_type: String,
    activity_id: Option<i32>,
}

#[derive(Clone, Insertable)]
#[diesel(table_name = txs)]
pub struct NewTx<'a> {
    date: NaiveDate,
    details: Option<&'a str>,
    from_method: i32,
    to_method: Option<i32>,
    amount: i64,
    tx_type: &'a str,
    activity_id: Option<i32>,
}

impl<'a> NewTx<'a> {
    pub fn new(
        date: NaiveDate,
        details: Option<&'a str>,
        from_method: i32,
        to_method: Option<i32>,
        amount: i64,
        tx_type: &'a str,
        activity_id: Option<i32>,
    ) -> Self {
        NewTx {
            date,
            details,
            from_method,
            to_method,
            amount,
            tx_type,
            activity_id,
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
                amount: tx.amount,
                tx_type: tx.tx_type.as_str().into(),
                activity_id: tx.activity_id,
                tags,
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
                TxType::Income => {
                    map.insert(*method_id, format!("↑{:.2}", self.amount as f64 / 100.0));
                }
                TxType::Expense => {
                    map.insert(*method_id, format!("↓{:.2}", self.amount as f64 / 100.0));
                }
                TxType::Transfer => {
                    if self.from_method.id == *method_id {
                        map.insert(*method_id, format!("↓{:.2}", self.amount as f64 / 100.0));
                    } else {
                        map.insert(*method_id, format!("↑{:.2}", self.amount as f64 / 100.0));
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

    pub fn to_array(&self) -> Vec<String> {
        let mut method = self.from_method.name.clone();

        if let Some(to_method) = &self.to_method {
            method = format!("{} → {}", self.from_method.name, to_method.name);
        }

        vec![
            self.date.format("%d-%m-%Y").to_string(),
            self.details.clone().unwrap_or_default(),
            method,
            format!("{:.2}", self.amount as f64 / 100.0),
            self.tx_type.to_string(),
            self.tags
                .iter()
                .map(|t| t.name.clone())
                .collect::<Vec<String>>()
                .join(", "),
        ]
    }
}

impl Tx {
    pub fn get_txs(
        d: NaiveDate,
        nature: FetchNature,
        db_conn: &mut impl ConnCache,
    ) -> Result<Vec<Self>, Error> {
        use crate::schema::txs::dsl::{date, id, txs};

        let (start_date, end_date) = match nature {
            FetchNature::Monthly => {
                let start_date = NaiveDate::from_ymd_opt(d.year(), d.month(), 1).unwrap();

                let end_date = start_date + Months::new(1) - Days::new(1);
                (start_date, end_date)
            }
            FetchNature::Yearly => {
                let start_date = NaiveDate::from_ymd_opt(d.year(), 1, 1).unwrap();

                let end_date = start_date + Months::new(12) - Days::new(1);
                (start_date, end_date)
            }
        };

        txs.filter(date.ge(start_date))
            .filter(date.le(end_date))
            .order((date.asc(), id.asc()))
            .select(Tx::as_select())
            .load(db_conn.conn())
    }
}
