use chrono::Datelike;
use diesel::prelude::*;
use diesel::result::Error;
use rex_shared::models::Cent;
use std::collections::HashMap;

use crate::ConnCache;
use crate::models::{
    ActivityTxTag, AmountNature, AmountType, DateNature, EMPTY, FullTx, NewSearch, NewTx, Tag,
    TxMethod, TxType,
};
use crate::schema::activity_txs;

#[derive(Clone, Queryable, Selectable, Insertable)]
pub struct ActivityTx {
    pub id: i32,
    date: Option<String>,
    details: Option<String>,
    from_method: Option<i32>,
    to_method: Option<i32>,
    amount: Option<i64>,
    amount_type: Option<String>,
    tx_type: Option<String>,
    display_order: Option<i32>,
    activity_num: i32,
}

pub struct FullActivityTx {
    pub id: i32,
    date: Option<String>,
    pub details: Option<String>,
    from_method: Option<TxMethod>,
    to_method: Option<TxMethod>,
    amount: Option<AmountNature>,
    tx_type: Option<TxType>,
    pub display_order: Option<i32>,
    tags: Vec<Tag>,
}

#[derive(Insertable)]
#[diesel(table_name = activity_txs)]
pub struct NewActivityTx {
    date: Option<String>,
    details: Option<String>,
    from_method: Option<i32>,
    to_method: Option<i32>,
    amount: Option<i64>,
    amount_type: Option<String>,
    tx_type: Option<String>,
    display_order: Option<i32>,
    activity_num: i32,
}

impl NewActivityTx {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        date: Option<String>,
        details: Option<String>,
        from_method: Option<i32>,
        to_method: Option<i32>,
        amount: Option<i64>,
        amount_type: Option<String>,
        tx_type: Option<String>,
        display_order: Option<i32>,
        activity_num: i32,
    ) -> Self {
        Self {
            date,
            details,
            from_method,
            to_method,
            amount,
            amount_type,
            tx_type,
            display_order,
            activity_num,
        }
    }

    #[must_use]
    pub fn new_from_new_tx(tx: &NewTx, activity_num: i32) -> Self {
        let date = Some(tx.date.to_string());

        let details = Some(tx.details.unwrap_or_default().to_string());

        let from_method = Some(tx.from_method);
        let amount = Some(tx.amount);
        let amount_type = Some(AmountType::Exact.into());
        let tx_type = Some(tx.tx_type.to_string());

        Self {
            date,
            details,
            from_method,
            to_method: tx.to_method,

            amount,
            amount_type,
            tx_type,
            display_order: None,
            activity_num,
        }
    }

    #[must_use]
    pub fn new_from_full_tx(tx: &FullTx, set_display_order: bool, activity_num: i32) -> Self {
        let date = Some(tx.date.to_string());

        let details = tx.details.as_ref().map(std::string::ToString::to_string);

        let from_method = Some(tx.from_method.id);
        let to_method = tx.to_method.as_ref().map(|to_method| to_method.id);

        let amount = Some(tx.amount.value());
        let amount_type = Some(AmountType::Exact.into());
        let tx_type = Some(tx.tx_type.to_string());

        let display_order = if set_display_order {
            Some(tx.display_order)
        } else {
            None
        };

        Self {
            date,
            details,
            from_method,
            to_method,
            amount,
            amount_type,
            tx_type,
            display_order,
            activity_num,
        }
    }

    #[must_use]
    pub fn new_from_search_tx(tx: &NewSearch, activity_num: i32) -> Self {
        let date = if let Some(date) = tx.date.as_ref() {
            match date {
                DateNature::Exact(d) => Some(d.to_string()),
                DateNature::ByMonth {
                    start_date,
                    end_date: _,
                } => Some(format!("{}-{}", start_date.year(), start_date.month())),
                DateNature::ByYear {
                    start_date,
                    end_date: _,
                } => Some(format!("{}", start_date.year())),
            }
        } else {
            None
        };

        Self {
            date,
            details: tx.details.map(std::string::ToString::to_string),
            from_method: tx.from_method,
            to_method: tx.to_method,
            amount: tx.amount.map(|a| a.extract().value()),
            amount_type: tx.amount.map(|a| a.to_type().into()),
            tx_type: tx.tx_type.map(std::string::ToString::to_string),
            display_order: None,
            activity_num,
        }
    }

    pub fn insert(self, db_conn: &mut impl ConnCache) -> Result<ActivityTx, Error> {
        use crate::schema::activity_txs::dsl::activity_txs;

        diesel::insert_into(activity_txs)
            .values(self)
            .returning(ActivityTx::as_select())
            .get_result(db_conn.conn())
    }
}

impl ActivityTx {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        date: Option<String>,
        details: Option<String>,
        from_method: Option<i32>,
        to_method: Option<i32>,
        amount: Option<i64>,
        amount_type: Option<String>,
        tx_type: Option<String>,
        display_order: Option<i32>,
        activity_num: i32,
        id: i32,
    ) -> Self {
        Self {
            id,
            date,
            details,
            from_method,
            to_method,
            amount,
            amount_type,
            tx_type,
            display_order,
            activity_num,
        }
    }

    pub fn insert(self, db_conn: &mut impl ConnCache) -> Result<usize, Error> {
        use crate::schema::activity_txs::dsl::activity_txs;

        diesel::insert_into(activity_txs)
            .values(self)
            .execute(db_conn.conn())
    }

    pub fn convert_to_full_tx(
        txs: Vec<&Self>,
        db_conn: &mut impl ConnCache,
    ) -> Result<Vec<FullActivityTx>, Error> {
        let tx_ids = txs.iter().map(|t| t.id).collect::<Vec<i32>>();

        let tx_tags = ActivityTxTag::get_by_tx_ids(tx_ids, db_conn)?;

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

            let from_method = tx
                .from_method
                .as_ref()
                .map(|method_id| db_conn.cache().tx_methods.get(method_id).unwrap().clone());

            let to_method = tx
                .to_method
                .as_ref()
                .map(|method_id| db_conn.cache().tx_methods.get(method_id).unwrap().clone());

            let tx_type = tx.tx_type.as_ref().map(|tx_type| tx_type.as_str().into());

            let mut amount = None;

            if let Some(a) = tx.amount.as_ref() {
                let amount_type: AmountType = tx.amount_type.as_ref().unwrap().as_str().into();
                amount = Some(AmountNature::from_type(amount_type, Cent::new(*a)));
            }

            let full_tx = FullActivityTx {
                id: tx.id,
                date: tx.date.clone(),
                details: tx.details.clone(),
                from_method,
                to_method,
                amount,
                tx_type,
                tags,
                display_order: tx.display_order,
            };

            to_return.push(full_tx);
        }

        Ok(to_return)
    }
}

impl FullActivityTx {
    #[must_use]
    pub fn to_array(&self) -> Vec<String> {
        let amount = if let Some(amount) = self.amount.as_ref() {
            amount.to_string()
        } else {
            String::new()
        };

        let method_name = if let Some(tx_type) = self.tx_type.as_ref() {
            match tx_type {
                TxType::Transfer => {
                    let to_method = self
                        .to_method
                        .as_ref()
                        .map_or("?".to_string(), |m| m.name.clone());

                    let from_method = self
                        .from_method
                        .as_ref()
                        .map_or("?".to_string(), |m| m.name.clone());

                    format!("{to_method} → {from_method}")
                }
                TxType::Income | TxType::Expense => self
                    .from_method
                    .as_ref()
                    .map(|m| m.name.clone())
                    .unwrap_or_default(),
            }
        } else {
            self.from_method
                .as_ref()
                .map(|m| m.name.clone())
                .unwrap_or_default()
        };

        vec![
            self.date.clone().unwrap_or_default(),
            self.details.clone().unwrap_or_default(),
            method_name,
            amount,
            self.tx_type
                .as_ref()
                .map(std::string::ToString::to_string)
                .unwrap_or_default(),
            self.tags
                .iter()
                .map(|t| t.name.clone())
                .collect::<Vec<String>>()
                .join(", "),
        ]
    }
}
