use anyhow::{Error, Result};
use chrono::NaiveDate;
pub use db::models::FetchNature;
use db::models::{FullTx, NewSearch, NewTx, Tag, TxMethod};
use db::{Cache, ConnCache, get_connection, get_connection_no_migrations};
use diesel::{Connection, SqliteConnection};
use std::collections::HashMap;

use crate::fetcher::{SearchView, TxViewGroup, get_search_txs, get_txs};
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

    pub fn add_new_tx(&mut self, tx: NewTx, tags: &str) -> Result<()> {
        self.conn.transaction::<_, Error, _>(|conn| {
            let mut db_conn = MutDbConn::new(conn, &self.cache);

            let new_tags = add_new_tx(tx, tags, None, &mut db_conn)?;

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

    pub fn edit_tx(&mut self, old_tx: &FullTx, new_tx: NewTx, tags: &str) -> Result<()> {
        self.conn.transaction::<_, Error, _>(|conn| {
            let mut db_conn = MutDbConn::new(conn, &self.cache);

            let old_tx_id = old_tx.id;
            delete_tx(old_tx, &mut db_conn)?;

            let new_tags = add_new_tx(new_tx, tags, Some(old_tx_id), &mut db_conn)?;

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

    pub fn fetch_tx_with_id(&mut self, id: i32) -> Result<FullTx> {
        let tx = FullTx::get_tx_by_id(id, self)?;

        Ok(tx)
    }

    pub fn fetch_txs_with_index(
        &mut self,
        month: usize,
        year: usize,
        nature: FetchNature,
    ) -> Result<TxViewGroup> {
        let result = self.conn.transaction::<TxViewGroup, Error, _>(|conn| {
            let mut db_conn = MutDbConn::new(conn, &self.cache);

            let month_num = (month + 1) as u32;
            let year_num = (year + 2022) as i32;

            let date = NaiveDate::from_ymd_opt(year_num, month_num, 1).unwrap();

            get_txs(date, nature, &mut db_conn)
        })?;

        Ok(result)
    }

    pub fn fetch_txs_with_date(
        &mut self,
        date: NaiveDate,
        nature: FetchNature,
    ) -> Result<TxViewGroup> {
        let result = self.conn.transaction::<TxViewGroup, Error, _>(|conn| {
            let mut db_conn = MutDbConn::new(conn, &self.cache);

            get_txs(date, nature, &mut db_conn)
        })?;

        Ok(result)
    }

    pub fn search_txs(&mut self, search: NewSearch) -> Result<SearchView> {
        let result = self.conn.transaction::<SearchView, Error, _>(|conn| {
            let mut db_conn = MutDbConn::new(conn, &self.cache);

            get_search_txs(search, &mut db_conn)
        })?;

        Ok(result)
    }
}
