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

impl TxMethod {
    pub fn get_all(db_conn: &mut DbConn) -> Result<Vec<TxMethod>, Error> {
        use crate::schema::tx_methods::dsl::tx_methods;

        tx_methods
            .select(TxMethod::as_select())
            .load(&mut db_conn.conn)
    }
}
