use diesel::prelude::*;
use diesel::result::Error;

use crate::DbConn;
use crate::schema::tags;

#[derive(Clone, Queryable, Insertable, Selectable)]
pub struct Tag {
    pub id: i32,
    pub name: String,
}

impl Tag {
    pub fn get_all(db_conn: &mut DbConn) -> Result<Vec<Tag>, Error> {
        use crate::schema::tags::dsl::tags;

        tags.select(Tag::as_select()).load(&mut db_conn.conn)
    }
}
