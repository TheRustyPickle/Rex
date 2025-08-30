pub mod models;
mod schema;

use diesel::prelude::*;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use std::collections::HashMap;

use crate::models::{Tag, TxMethod};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../db/src/migrations");

pub trait ConnCache {
    fn conn(&mut self) -> &mut SqliteConnection;
    fn cache(&self) -> &Cache;
}

#[derive(Clone)]
pub struct Cache {
    pub tags: HashMap<i32, Tag>,
    pub tx_methods: HashMap<i32, TxMethod>,
}

impl Cache {
    pub fn get_method_id(&self, name: &str) -> Option<i32> {
        self.tx_methods
            .values()
            .find(|m| m.name == name)
            .map(|m| m.id)
    }

    pub fn get_tag_id(&self, name: &str) -> Option<i32> {
        self.tags.values().find(|m| m.name == name).map(|m| m.id)
    }

    pub fn new_tags(&mut self, tags: Vec<Tag>) {
        for tag in tags {
            self.tags.insert(tag.id, tag);
        }
    }

    pub fn new_tx_methods(&mut self, tx_methods: Vec<TxMethod>) {
        for tx_method in tx_methods {
            self.tx_methods.insert(tx_method.id, tx_method);
        }
    }
}

pub fn get_connection(db_url: &str) -> SqliteConnection {
    let mut conn =
        SqliteConnection::establish(db_url).expect("Failed to create connection to database");

    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run database migrations");

    diesel::sql_query("PRAGMA foreign_keys = ON;")
        .execute(&mut conn)
        .unwrap();

    conn
}

pub fn get_connection_no_migrations(db_url: &str) -> SqliteConnection {
    let mut conn =
        SqliteConnection::establish(db_url).expect("Failed to create connection to database");

    diesel::sql_query("PRAGMA foreign_keys = ON;")
        .execute(&mut conn)
        .unwrap();

    conn
}
