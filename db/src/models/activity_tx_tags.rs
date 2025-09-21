use diesel::prelude::*;
use diesel::result::Error;

use crate::ConnCache;
use crate::schema::activity_tx_tags;

#[derive(Clone, Queryable, Insertable, Selectable)]
pub struct ActivityTxTag {
    pub tx_id: i32,
    pub tag_id: i32,
}

impl ActivityTxTag {
    pub fn new(tx_id: i32, tag_id: i32) -> Self {
        Self { tx_id, tag_id }
    }

    pub fn get_by_tx_ids(
        tx_ids: Vec<i32>,
        db_conn: &mut impl ConnCache,
    ) -> Result<Vec<Self>, Error> {
        use crate::schema::activity_tx_tags::dsl::{activity_tx_tags, tx_id};

        activity_tx_tags
            .filter(tx_id.eq_any(tx_ids))
            .load(db_conn.conn())
    }

    pub fn insert_batch(txs: Vec<Self>, db_conn: &mut impl ConnCache) -> Result<usize, Error> {
        use crate::schema::activity_tx_tags::dsl::activity_tx_tags;

        diesel::insert_into(activity_tx_tags)
            .values(txs)
            .execute(db_conn.conn())
    }

    pub fn delete_by_tx_id(tx_id_value: i32, db_conn: &mut impl ConnCache) -> Result<usize, Error> {
        use crate::schema::tx_tags::dsl::{tx_id, tx_tags};

        diesel::delete(tx_tags.filter(tx_id.eq(tx_id_value))).execute(db_conn.conn())
    }
}
