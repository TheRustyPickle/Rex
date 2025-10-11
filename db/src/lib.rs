pub mod models;
mod schema;

use anyhow::{Result, anyhow};
use diesel::prelude::*;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use std::collections::{HashMap, HashSet};

use crate::models::{FullTx, Tag, TxMethod};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../db/src/migrations");

pub trait ConnCache {
    fn conn(&mut self) -> &mut SqliteConnection;
    fn cache(&self) -> &Cache;
}

#[derive(Clone)]
pub struct Cache {
    pub tags: HashMap<i32, Tag>,
    pub tx_methods: HashMap<i32, TxMethod>,
    pub txs: Option<HashMap<i32, Vec<FullTx>>>,
    pub details: HashSet<String>,
}

impl Cache {
    pub fn get_method_id(&self, name: &str) -> Result<i32> {
        self.tx_methods
            .values()
            .find(|m| m.name == name)
            .map(|m| m.id)
            .ok_or_else(|| anyhow!("method '{name}' not found"))
    }

    pub fn get_method_by_name(&self, name: &str) -> Result<&TxMethod> {
        self.tx_methods
            .values()
            .find(|m| m.name == name)
            .ok_or_else(|| anyhow!("method '{name}' not found"))
    }

    pub fn get_method_by_name_mut(&mut self, name: &str) -> Result<&mut TxMethod> {
        let method = self.get_method_by_name(name)?.id;

        Ok(self.tx_methods.get_mut(&method).unwrap())
    }

    pub fn get_tag_id(&self, name: &str) -> Result<i32> {
        self.tags
            .values()
            .find(|m| m.name == name)
            .map(|m| m.id)
            .ok_or_else(|| anyhow!("tag '{name}' not found"))
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

    // TODO: Start using cache
    pub fn set_txs(&mut self, txs: HashMap<i32, Vec<FullTx>>) {
        self.txs = Some(txs);
    }

    #[must_use]
    pub fn get_txs(&self, id: i32) -> Option<&Vec<FullTx>> {
        if let Some(txs) = &self.txs {
            return txs.get(&id);
        }

        None
    }

    #[must_use]
    pub fn get_methods(&self) -> Vec<&TxMethod> {
        let mut methods = self.tx_methods.values().collect::<Vec<&TxMethod>>();
        methods.sort_by_key(|value| value.position);
        methods
    }

    #[must_use]
    pub fn get_tags_set(&self) -> HashSet<String> {
        self.tags
            .values()
            .map(|m| m.name.clone())
            .collect::<HashSet<String>>()
    }

    #[must_use]
    pub fn get_tags_sorted(&self) -> Vec<&Tag> {
        let mut tags = self.tags.values().collect::<Vec<&Tag>>();
        tags.sort_by_key(|value| &value.name);
        tags
    }
}

#[must_use]
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

#[must_use]
pub fn get_connection_no_migrations(db_url: &str) -> SqliteConnection {
    let mut conn =
        SqliteConnection::establish(db_url).expect("Failed to create connection to database");

    diesel::sql_query("PRAGMA foreign_keys = ON;")
        .execute(&mut conn)
        .unwrap();

    conn
}
