use std::collections::HashMap;

use diesel::prelude::*;
use diesel::result::Error;

use crate::ConnCache;
use crate::models::{ActivityTxTag, EMPTY, Tag, TxMethod, TxType};
use crate::schema::activity_txs;

#[derive(Clone, Queryable, Selectable, Insertable)]
pub struct ActivityTx {
    id: i32,
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
    details: Option<String>,
    from_method: Option<TxMethod>,
    to_method: Option<TxMethod>,
    amount: Option<i64>,
    amount_type: Option<String>,
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

    pub fn insert(self, db_conn: &mut impl ConnCache) -> Result<usize, Error> {
        use crate::schema::activity_txs::dsl::activity_txs;

        diesel::insert_into(activity_txs)
            .values(self)
            .execute(db_conn.conn())
    }
}

impl ActivityTx {
    #[allow(clippy::too_many_arguments)]
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
            date,
            details,
            from_method,
            to_method,
            amount,
            amount_type,
            tx_type,
            display_order,
            activity_num,
            id,
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

            let full_tx = FullActivityTx {
                id: tx.id,
                date: tx.date.clone(),
                details: tx.details.clone(),
                from_method,
                to_method,
                amount: tx.amount,
                amount_type: tx.amount_type.clone(),
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
    pub fn to_array(&self) -> Vec<String> {
        let amount = if let Some(amount) = self.amount {
            format!("{:.2}", amount as f64 / 100.0)
        } else {
            String::new()
        };

        vec![
            self.date.clone().unwrap_or_default(),
            self.details.clone().unwrap_or_default(),
            self.from_method
                .as_ref()
                .map(|m| m.name.clone())
                .unwrap_or_default(),
            self.to_method
                .as_ref()
                .map(|m| m.name.clone())
                .unwrap_or_default(),
            amount,
            self.tx_type
                .as_ref()
                .map(|t| t.to_string())
                .unwrap_or_default(),
            self.tags
                .iter()
                .map(|t| t.name.clone())
                .collect::<Vec<String>>()
                .join(", "),
        ]
    }
}
