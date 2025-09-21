use diesel::prelude::*;
use diesel::result::Error;

use crate::ConnCache;
use crate::schema::tx_methods;

#[derive(Clone, Debug, Queryable, Insertable, Selectable)]
pub struct TxMethod {
    pub id: i32,
    pub name: String,
    pub position: i32,
}

#[derive(Insertable)]
#[diesel(table_name = tx_methods)]
pub struct NewTxMethod<'a> {
    name: &'a str,
    position: i32,
}

impl NewTxMethod<'_> {
    #[must_use]
    pub fn new(name: &str, position: i32) -> NewTxMethod<'_> {
        NewTxMethod { name, position }
    }

    pub fn insert(self, db_conn: &mut impl ConnCache) -> Result<TxMethod, Error> {
        use crate::schema::tx_methods::dsl::tx_methods;

        diesel::insert_into(tx_methods)
            .values(self)
            .returning(TxMethod::as_returning())
            .get_result(db_conn.conn())
    }
}

impl TxMethod {
    pub fn get_all(db_conn: &mut impl ConnCache) -> Result<Vec<TxMethod>, Error> {
        use crate::schema::tx_methods::dsl::tx_methods;

        tx_methods
            .select(TxMethod::as_select())
            .load(db_conn.conn())
    }

    pub fn get_last_position(db_conn: &mut impl ConnCache) -> Result<i32, Error> {
        use crate::schema::tx_methods::dsl::{position, tx_methods};

        tx_methods
            .select(position)
            .order(position.desc())
            .first(db_conn.conn())
            .optional()
            .map(|opt| opt.unwrap_or(0))
    }

    pub fn get_by_name(db_conn: &mut impl ConnCache, name: &str) -> Result<TxMethod, Error> {
        use crate::schema::tx_methods::dsl::{name as tx_method_name, tx_methods};

        tx_methods
            .filter(tx_method_name.eq(name))
            .first(db_conn.conn())
    }
}
