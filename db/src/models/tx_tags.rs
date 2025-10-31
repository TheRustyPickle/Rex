use diesel::prelude::*;
use diesel::result::Error;

use crate::ConnCache;
use crate::schema::tx_tags;

#[derive(Clone, Queryable, Insertable, Selectable)]
pub struct TxTag {
    pub tx_id: i32,
    pub tag_id: i32,
    pub is_primary: bool,
}

impl TxTag {
    #[must_use]
    pub fn new(tx_id: i32, tag_id: i32, is_primary: bool) -> Self {
        TxTag {
            tx_id,
            tag_id,
            is_primary,
        }
    }

    pub fn get_by_tx_ids(
        tx_ids: Vec<i32>,
        db_conn: &mut impl ConnCache,
    ) -> Result<Vec<TxTag>, Error> {
        use crate::schema::tx_tags::dsl::{tx_id, tx_tags};

        tx_tags.filter(tx_id.eq_any(tx_ids)).load(db_conn.conn())
    }

    pub fn insert_batch(txs: Vec<TxTag>, db_conn: &mut impl ConnCache) -> Result<usize, Error> {
        use crate::schema::tx_tags::dsl::tx_tags;

        diesel::insert_into(tx_tags)
            .values(txs)
            .execute(db_conn.conn())
    }

    pub fn delete_by_tx_id(tx_id_value: i32, db_conn: &mut impl ConnCache) -> Result<usize, Error> {
        use crate::schema::tx_tags::dsl::{tx_id, tx_tags};

        diesel::delete(tx_tags.filter(tx_id.eq(tx_id_value))).execute(db_conn.conn())
    }
}
