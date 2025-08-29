pub use db::DbConn;

pub fn get_conn(location: &str) -> DbConn {
    DbConn::new(location)
}

pub fn get_conn_old(location: &str) -> DbConn {
    DbConn::new_no_migrations(location)
}

