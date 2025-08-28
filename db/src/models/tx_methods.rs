use diesel::prelude::*;
use diesel::result::Error;

use crate::DbConn;
use crate::schema::tx_methods;

#[derive(Clone, Queryable, Insertable, Selectable)]
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
    pub fn new(name: &str, position: i32) -> NewTxMethod<'_> {
        NewTxMethod { name, position }
    }

    pub fn insert(self, conn: &mut SqliteConnection) -> Result<TxMethod, Error> {
        use crate::schema::tx_methods::dsl::tx_methods;

        diesel::insert_into(tx_methods)
            .values(self)
            .returning(TxMethod::as_returning())
            .get_result(conn)
    }
}

impl TxMethod {
    pub fn get_all(db_conn: &mut DbConn) -> Result<Vec<TxMethod>, Error> {
        use crate::schema::tx_methods::dsl::tx_methods;

        tx_methods
            .select(TxMethod::as_select())
            .load(&mut db_conn.conn)
    }

    pub fn get_last_position(db_conn: &mut DbConn) -> Result<i32, Error> {
        use crate::schema::tx_methods::dsl::{position, tx_methods};

        tx_methods
            .select(position)
            .order(position.desc())
            .first(&mut db_conn.conn)
            .optional()
            .map(|opt| opt.unwrap_or(0))
    }
}
