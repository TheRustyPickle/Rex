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
        let mut conn = get_connection(db_url);

        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run database migrations");

        diesel::sql_query("PRAGMA foreign_keys = ON;")
            .execute(&mut conn)
            .unwrap();

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
        let conn = get_connection(db_url);
        DbConn {
            conn,
            cache: Cache {
                tags: HashMap::new(),
                tx_methods: HashMap::new(),
            },
        }
    }

    pub fn reload_methods(&mut self) {
        let tx_methods = TxMethod::get_all(self)
            .unwrap()
            .into_iter()
            .map(|t| (t.id, t))
            .collect();

        self.cache.tx_methods = tx_methods;
    }

    pub fn reload_tags(&mut self) {
        let tags = Tag::get_all(self)
            .unwrap()
            .into_iter()
            .map(|t| (t.id, t))
            .collect();

        self.cache.tags = tags;
    }

    pub fn get_method_id(&self, name: &str) -> Option<i32> {
        self.cache
            .tx_methods
            .values()
            .find(|m| m.name == name)
            .map(|m| m.id)
    }

    pub fn get_tag_id(&self, name: &str) -> Option<i32> {
        self.cache
            .tags
            .values()
            .find(|m| m.name == name)
            .map(|m| m.id)
    }
}

fn get_connection(db_url: &str) -> SqliteConnection {
    SqliteConnection::establish(db_url).expect("Failed to create connection to database")
}
