#![recursion_limit = "2048"]
pub mod models;
mod schema;

use diesel::prelude::*;
use std::collections::HashMap;

use crate::models::{Tag, TxMethod};

pub struct DbConn {
    pub conn: SqliteConnection,
    pub tags: HashMap<i32, Tag>,
    pub tx_methods: HashMap<i32, TxMethod>,
}

impl DbConn {
    #[must_use]
    pub fn new(db_url: &str) -> Self {
        let mut conn = get_connection(db_url);
        diesel::sql_query("PRAGMA foreign_keys = ON;")
            .execute(&mut conn)
            .unwrap();

        let mut to_return = DbConn {
            conn,
            tags: HashMap::new(),
            tx_methods: HashMap::new(),
        };

        let tags = Tag::get_all(&mut to_return)
            .unwrap()
            .into_iter()
            .map(|t| (t.id, t))
            .collect();

        let tx_methods = TxMethod::get_all(&mut to_return)
            .unwrap()
            .into_iter()
            .map(|t| (t.id, t))
            .collect();

        to_return.tags = tags;
        to_return.tx_methods = tx_methods;

        to_return
    }
}

fn get_connection(db_url: &str) -> SqliteConnection {
    SqliteConnection::establish(db_url).expect("Failed to create connection to database")
}
