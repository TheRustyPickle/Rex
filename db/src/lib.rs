pub mod models;
mod schema;

use anyhow::{Result, anyhow};
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use std::collections::{HashMap, HashSet};

use crate::models::{Tag, TxMethod};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("src/migrations");

pub trait ConnCache {
    fn conn(&mut self) -> &mut SqliteConnection;
    fn cache(&self) -> &Cache;
}

#[derive(Clone)]
pub struct Cache {
    pub tags: HashMap<i32, Tag>,
    pub tx_methods: HashMap<i32, TxMethod>,
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

    pub fn new_details(&mut self, details: String) {
        self.details.insert(details);
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

/// Why a database could not be opened
#[derive(Debug, thiserror::Error)]
pub enum OpenError {
    /// SQLite could not read the file. With an encrypted database this is the only signal a wrong
    /// or missing password gives, and the same goes for a plain database opened with a password
    #[error("Wrong password, or the file is not a database")]
    NotReadable,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Quotes text for use inside a SQL string literal
fn sql_quote(text: &str) -> String {
    format!("'{}'", text.replace('\'', "''"))
}

/// Opens the database, unlocking it with `key` if given, then runs pending migrations. A wrong
/// key is reported as [`OpenError::NotReadable`] instead of a panic or a failing later query.
pub fn open_connection(db_url: &str, key: Option<&str>) -> Result<SqliteConnection, OpenError> {
    let mut conn = SqliteConnection::establish(db_url).map_err(|e| OpenError::Other(e.into()))?;

    if let Some(key) = key {
        conn.batch_execute(&format!("PRAGMA key = {};", sql_quote(key)))
            .map_err(|e| OpenError::Other(e.into()))?;
    }

    // Opening never reads the file, so a wrong key only shows on the first real query
    if conn
        .batch_execute("SELECT count(*) FROM sqlite_master;")
        .is_err()
    {
        return Err(OpenError::NotReadable);
    }

    conn.run_pending_migrations(MIGRATIONS)
        .map_err(|e| OpenError::Other(anyhow!("Failed to run database migrations: {e}")))?;

    conn.batch_execute("PRAGMA foreign_keys = ON;")
        .map_err(|e| OpenError::Other(e.into()))?;

    Ok(conn)
}

/// Changes the key of an already unlocked encrypted database
pub fn change_key(conn: &mut SqliteConnection, new_key: &str) -> Result<()> {
    conn.batch_execute(&format!("PRAGMA rekey = {};", sql_quote(new_key)))?;
    Ok(())
}

/// Copies the whole database into a new encrypted file at `target` using `key`
pub fn export_encrypted(conn: &mut SqliteConnection, target: &str, key: &str) -> Result<()> {
    conn.batch_execute(&format!(
        "ATTACH DATABASE {} AS encrypted KEY {};
         SELECT sqlcipher_export('encrypted');
         DETACH DATABASE encrypted;",
        sql_quote(target),
        sql_quote(key)
    ))?;
    Ok(())
}

#[must_use]
pub fn get_connection(db_url: &str) -> SqliteConnection {
    open_connection(db_url, None).expect("Failed to create connection to database")
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
