use diesel::prelude::*;
use diesel::result::Error;

use crate::DbConn;
use crate::schema::tx_tags;

#[derive(Clone, Queryable, Insertable, Selectable)]
pub struct TxTag {
    pub tx_id: i32,
    pub tag_id: i32,
}

impl TxTag {
    pub fn new(tx_id: i32, tag_id: i32) -> Self {
        TxTag { tx_id, tag_id }
    }

    pub fn get_by_tx_ids(tx_ids: Vec<i32>, db_conn: &mut DbConn) -> Result<Vec<TxTag>, Error> {
        use crate::schema::tx_tags::dsl::{tx_id, tx_tags};

        tx_tags.filter(tx_id.eq_any(tx_ids)).load(&mut db_conn.conn)
    }

    pub fn insert_batch(txs: Vec<TxTag>, conn: &mut SqliteConnection) -> Result<usize, Error> {
        use crate::schema::tx_tags::dsl::tx_tags;

        diesel::insert_into(tx_tags).values(txs).execute(conn)
    }
}
