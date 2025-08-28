use db::DbConn;

pub fn get_conn(location: &str) -> DbConn {
    DbConn::new(location)
}
