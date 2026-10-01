use diesel::prelude::*;
use diesel::result::Error;

use crate::ConnCache;
use crate::schema::recurring_tx_tags;

#[derive(Clone, Queryable, Insertable, Selectable)]
pub struct RecurringTxTag {
    pub recurring_tx_id: i32,
    pub tag_id: i32,
    pub is_primary: bool,
}

impl RecurringTxTag {
    #[must_use]
    pub fn new(recurring_tx_id: i32, tag_id: i32, is_primary: bool) -> Self {
        RecurringTxTag {
            recurring_tx_id,
            tag_id,
            is_primary,
        }
    }

    pub fn get_by_recurring_tx_ids(
        recurring_tx_ids: Vec<i32>,
        db_conn: &mut impl ConnCache,
    ) -> Result<Vec<RecurringTxTag>, Error> {
        use crate::schema::recurring_tx_tags::dsl::{
            is_primary, recurring_tx_id, recurring_tx_tags,
        };

        recurring_tx_tags
            .filter(recurring_tx_id.eq_any(recurring_tx_ids))
            .order(is_primary.desc())
            .load(db_conn.conn())
    }

    pub fn insert_batch(
        tags: Vec<RecurringTxTag>,
        db_conn: &mut impl ConnCache,
    ) -> Result<usize, Error> {
        use crate::schema::recurring_tx_tags::dsl::recurring_tx_tags;

        diesel::insert_into(recurring_tx_tags)
            .values(tags)
            .execute(db_conn.conn())
    }

    pub fn delete_by_recurring_tx_id(
        recurring_tx_id_value: i32,
        db_conn: &mut impl ConnCache,
    ) -> Result<usize, Error> {
        use crate::schema::recurring_tx_tags::dsl::{recurring_tx_id, recurring_tx_tags};

        diesel::delete(recurring_tx_tags.filter(recurring_tx_id.eq(recurring_tx_id_value)))
            .execute(db_conn.conn())
    }
}
