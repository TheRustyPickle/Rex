use diesel::prelude::*;
use diesel::result::Error;

use crate::ConnCache;
use crate::schema::tags;

#[derive(Clone, Debug, Queryable, Insertable, Selectable)]
pub struct Tag {
    pub id: i32,
    pub name: String,
}

#[derive(Insertable)]
#[diesel(table_name = tags)]
pub struct NewTag<'a> {
    pub name: &'a str,
}

impl<'a> NewTag<'a> {
    pub fn new(name: &'a str) -> Self {
        NewTag { name }
    }

    pub fn insert(self, db_conn: &mut impl ConnCache) -> Result<Tag, Error> {
        use crate::schema::tags::dsl::{name, tags};

        diesel::insert_into(tags)
            .values(self)
            .on_conflict(name)
            .do_update()
            .set(name.eq(name))
            .returning(Tag::as_returning())
            .get_result(db_conn.conn())
    }
}

impl Tag {
    pub fn get_all(db_conn: &mut impl ConnCache) -> Result<Vec<Tag>, Error> {
        use crate::schema::tags::dsl::tags;

        tags.select(Tag::as_select()).load(db_conn.conn())
    }
}
