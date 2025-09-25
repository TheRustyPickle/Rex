use anyhow::{Error, Result};
use chrono::NaiveDate;
pub use db::models::FetchNature;
use db::models::{Balance, FullTx, NewSearch, NewTx, Tag, TxMethod};
use db::{Cache, ConnCache, get_connection, get_connection_no_migrations};
use diesel::{Connection, SqliteConnection};
use std::collections::HashMap;

use crate::fetcher::{
    ActivityView, ChartView, SearchView, SummaryView, TxViewGroup, get_activity_view,
    get_chart_view, get_search_txs, get_summary, get_txs,
};
use crate::modifier::{
    activity_delete_tx, activity_edit_tx, activity_new_tx, activity_search_tx,
    activity_swap_position, add_new_tx, add_new_tx_methods, delete_tx,
};
use crate::utils::month_name_to_num;

#[must_use]
pub fn get_conn(location: &str) -> DbConn {
    DbConn::new(location)
}

#[must_use]
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

impl ConnCache for MutDbConn<'_> {
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
                txs: None,
            },
        };

        to_return.reload_methods();
        to_return.reload_tags();

        to_return
    }

    #[must_use]
    pub fn new_no_migrations(db_url: &str) -> Self {
        let conn = get_connection_no_migrations(db_url);
        DbConn {
            conn,
            cache: Cache {
                tags: HashMap::new(),
                tx_methods: HashMap::new(),
                txs: None,
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

            let new_tags = add_new_tx(tx.clone(), tags, None, &mut db_conn)?;

            activity_new_tx(&tx, tags, &mut db_conn)?;
            self.cache.new_tags(new_tags);

            Ok(())
        })?;

        Ok(())
    }

    pub fn delete_tx(&mut self, tx: &FullTx) -> Result<()> {
        self.conn.transaction::<_, Error, _>(|conn| {
            let mut db_conn = MutDbConn::new(conn, &self.cache);

            delete_tx(tx, &mut db_conn)?;
            activity_delete_tx(tx, &mut db_conn)?;

            Ok(())
        })?;

        Ok(())
    }

    pub fn edit_tx(&mut self, old_tx: &FullTx, new_tx: NewTx, tags: &str) -> Result<()> {
        self.conn.transaction::<_, Error, _>(|conn| {
            let mut db_conn = MutDbConn::new(conn, &self.cache);

            let old_tx_id = old_tx.id;
            delete_tx(old_tx, &mut db_conn)?;

            let new_tags = add_new_tx(new_tx.clone(), tags, Some(old_tx_id), &mut db_conn)?;

            activity_edit_tx(old_tx, &new_tx, tags, &mut db_conn)?;

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

    pub fn fetch_txs_with_str<'a>(
        &mut self,
        month: &'a str,
        year: &'a str,
        nature: FetchNature,
    ) -> Result<TxViewGroup> {
        let result = self.conn.transaction::<TxViewGroup, Error, _>(|conn| {
            let mut db_conn = MutDbConn::new(conn, &self.cache);

            let year_num = year.parse::<i32>().unwrap();
            let month_num = month_name_to_num(month);

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

            let search_view = get_search_txs(&search, &mut db_conn)?;

            activity_search_tx(&search, &mut db_conn)?;

            Ok(search_view)
        })?;

        Ok(result)
    }

    pub fn get_summary_with_str<'a>(
        &mut self,
        month: &'a str,
        year: &'a str,
        nature: FetchNature,
    ) -> Result<SummaryView> {
        let (summary, txs) = self
            .conn
            .transaction::<(SummaryView, Option<HashMap<i32, Vec<FullTx>>>), Error, _>(|conn| {
                let mut db_conn = MutDbConn::new(conn, &self.cache);

                let year_num = year.parse::<i32>().unwrap();
                let month_num = month_name_to_num(month);

                let date = NaiveDate::from_ymd_opt(year_num, month_num, 1).unwrap();

                get_summary(date, nature, &mut db_conn)
            })?;

        if let Some(txs) = txs {
            self.cache.set_txs(txs);
        }

        Ok(summary)
    }

    pub fn get_chart_view_with_str<'a>(
        &mut self,
        month: &'a str,
        year: &'a str,
        nature: FetchNature,
    ) -> Result<ChartView> {
        let result = self.conn.transaction::<ChartView, Error, _>(|conn| {
            let mut db_conn = MutDbConn::new(conn, &self.cache);

            let year_num = year.parse::<i32>().unwrap();
            let month_num = month_name_to_num(month);

            let date = NaiveDate::from_ymd_opt(year_num, month_num, 1).unwrap();

            let tx_view = get_txs(date, nature, &mut db_conn)?;

            let chart_view = get_chart_view(tx_view);

            Ok(chart_view)
        })?;

        Ok(result)
    }

    pub fn get_activity_view_with_str<'a>(
        &mut self,
        month: &'a str,
        year: &'a str,
    ) -> Result<ActivityView> {
        let result = self.conn.transaction::<ActivityView, Error, _>(|conn| {
            let mut db_conn = MutDbConn::new(conn, &self.cache);

            let year_num = year.parse::<i32>().unwrap();
            let month_num = month_name_to_num(month);

            let date = NaiveDate::from_ymd_opt(year_num, month_num, 1).unwrap();

            let activity_view = get_activity_view(date, &mut db_conn)?;

            Ok(activity_view)
        })?;

        Ok(result)
    }

    pub fn swap_tx_position(
        &mut self,
        index_1: usize,
        index_2: usize,
        tx_view_group: &mut TxViewGroup,
    ) -> Result<bool> {
        let result = self.conn.transaction::<bool, Error, _>(|conn| {
            let mut db_conn = MutDbConn::new(conn, &self.cache);

            let result = tx_view_group.switch_tx_index(index_1, index_2, &mut db_conn)?;

            let tx_1 = tx_view_group.get_tx(index_1);
            let tx_2 = tx_view_group.get_tx(index_2);

            activity_swap_position(tx_1, tx_2, &mut db_conn)?;

            Ok(result)
        })?;

        Ok(result)
    }

    #[must_use]
    pub fn get_tx_methods(&self) -> &HashMap<i32, TxMethod> {
        &self.cache.tx_methods
    }

    #[must_use]
    pub fn get_tx_methods_sorted(&self) -> Vec<&TxMethod> {
        self.cache.get_methods()
    }

    pub fn get_tx_method_by_name(&mut self, name: &str) -> Result<&TxMethod> {
        self.cache.get_method_by_name(name)
    }

    pub fn get_final_balances(&mut self) -> Result<HashMap<i32, Balance>> {
        Ok(Balance::get_final_balance(self)?)
    }
}
