use anyhow::{Error, Result};
use db::models::{FullTx, Tag, TxMethod};
use db::{Cache, ConnCache, get_connection, get_connection_no_migrations};
use diesel::{Connection, SqliteConnection};
use std::collections::HashMap;

use crate::modifier::{add_new_tx, add_new_tx_methods, delete_tx};

pub fn get_conn(location: &str) -> DbConn {
    DbConn::new(location)
}

pub fn get_conn_old(location: &str) -> DbConn {
    DbConn::new_no_migrations(location)
}

pub struct MutDbConn<'a> {
    conn: &'a mut SqliteConnection,
    cache: &'a Cache,
}

impl<'a> MutDbConn<'a> {
    pub fn new(conn: &'a mut SqliteConnection, cache: &'a Cache) -> Self {
        MutDbConn { conn, cache }
    }
}

impl<'a> ConnCache for MutDbConn<'a> {
    fn conn(&mut self) -> &mut SqliteConnection {
        self.conn
    }

    fn cache(&self) -> &Cache {
        self.cache
    }
}

pub struct DbConn {
    pub conn: SqliteConnection,
    pub cache: Cache,
}

impl ConnCache for DbConn {
    fn conn(&mut self) -> &mut SqliteConnection {
        &mut self.conn
    }

    fn cache(&self) -> &Cache {
        &self.cache
    }
}

impl DbConn {
    #[must_use]
    pub fn new(db_url: &str) -> Self {
        let conn = get_connection(db_url);

        let mut to_return = DbConn {
            conn,
            cache: Cache {
                tags: HashMap::new(),
                tx_methods: HashMap::new(),
            },
        };

        to_return.reload_methods();
        to_return.reload_tags();

        to_return
    }

    pub fn new_no_migrations(db_url: &str) -> Self {
        let conn = get_connection_no_migrations(db_url);
        DbConn {
            conn,
            cache: Cache {
                tags: HashMap::new(),
                tx_methods: HashMap::new(),
            },
        }
    }

    pub(crate) fn reload_methods(&mut self) {
        let tx_methods = TxMethod::get_all(self)
            .unwrap()
            .into_iter()
            .map(|t| (t.id, t))
            .collect();

        self.cache.tx_methods = tx_methods;
    }

    pub(crate) fn reload_tags(&mut self) {
        let tags = Tag::get_all(self)
            .unwrap()
            .into_iter()
            .map(|t| (t.id, t))
            .collect();

        self.cache.tags = tags;
    }

    pub fn add_new_tx(
        &mut self,
        date: &str,
        details: &str,
        from_method: &str,
        to_method: &str,
        amount: &str,
        tx_type: &str,
        tags: &str,
    ) -> Result<()> {
        self.conn.transaction::<_, Error, _>(|conn| {
            let mut db_conn = MutDbConn::new(conn, &self.cache);

            let new_tags = add_new_tx(
                date,
                details,
                from_method,
                to_method,
                amount,
                tx_type,
                tags,
                None,
                &mut db_conn,
            )?;

            self.cache.new_tags(new_tags);

            Ok(())
        })?;

        Ok(())
    }

    pub fn delete_tx(&mut self, tx: &FullTx) -> Result<()> {
        self.conn.transaction::<_, Error, _>(|conn| {
            let mut db_conn = MutDbConn::new(conn, &self.cache);

            delete_tx(tx, &mut db_conn)?;

            Ok(())
        })?;

        Ok(())
    }

    pub fn edit_tx(
        &mut self,
        old_tx: &FullTx,
        date: &str,
        details: &str,
        from_method: &str,
        to_method: &str,
        amount: &str,
        tx_type: &str,
        tags: &str,
    ) -> Result<()> {
        self.conn.transaction::<_, Error, _>(|conn| {
            let mut db_conn = MutDbConn::new(conn, &self.cache);

            let old_tx_id = old_tx.id;
            delete_tx(old_tx, &mut db_conn)?;

            let new_tags = add_new_tx(
                date,
                details,
                from_method,
                to_method,
                amount,
                tx_type,
                tags,
                Some(old_tx_id),
                &mut db_conn,
            )?;

            self.cache.new_tags(new_tags);

            Ok(())
        })?;

        Ok(())
    }

    pub fn add_new_methods(&mut self, method_list: &Vec<String>) -> Result<()> {
        self.conn.transaction::<_, Error, _>(|conn| {
            let mut db_conn = MutDbConn::new(conn, &self.cache);

            let new_methods = add_new_tx_methods(method_list, &mut db_conn)?;

            self.cache.new_tx_methods(new_methods);

            Ok(())
        })?;

        Ok(())
    }
}
