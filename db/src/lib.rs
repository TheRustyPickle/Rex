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

        to_return.reload_methods();

        let tx_methods = TxMethod::get_all(&mut to_return)
            .unwrap()
            .into_iter()
            .map(|t| (t.id, t))
            .collect();

        to_return.tx_methods = tx_methods;

        to_return
    }

    pub fn reload_methods(&mut self) {
        let tx_methods = TxMethod::get_all(self)
            .unwrap()
            .into_iter()
            .map(|t| (t.id, t))
            .collect();

        self.tx_methods = tx_methods;
    }

    pub fn get_method_id(&self, name: &str) -> Option<i32> {
        self.tx_methods
            .values()
            .find(|m| m.name == name)
            .map(|m| m.id)
    }

    pub fn get_tag_id(&self, name: &str) -> Option<i32> {
        self.tags.values().find(|m| m.name == name).map(|m| m.id)
    }
}

fn get_connection(db_url: &str) -> SqliteConnection {
    SqliteConnection::establish(db_url).expect("Failed to create connection to database")
}
