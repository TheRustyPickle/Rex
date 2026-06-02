use chrono::NaiveDate;
use rex_app::conn::{DbConn, FetchNature, get_conn};
use rex_app::modifier::parse_tx_fields;
use rex_db::models::FullTx;
use std::fs;

#[must_use]
pub fn create_test_db(file_name: &str) -> DbConn {
    if let Ok(metadata) = fs::metadata(file_name)
        && metadata.is_file()
    {
        fs::remove_file(file_name).expect("Failed to delete existing file");
    }

    let mut conn = get_conn(file_name);
    let tx_methods = ["Cash", "Bank", "Other"]
        .iter()
        .map(|name| name.to_string())
        .collect::<Vec<String>>();

    conn.add_new_methods(&tx_methods).unwrap();

    conn
}

pub fn add_tx(
    db_conn: &mut DbConn,
    date: &str,
    details: &str,
    from_method: &str,
    to_method: &str,
    amount: &str,
    tx_type: &str,
    tags: &str,
) -> FullTx {
    let new_tx = parse_tx_fields(
        date,
        details,
        from_method,
        to_method,
        amount,
        tx_type,
        db_conn,
    )
    .unwrap();

    db_conn.add_new_tx(new_tx, tags).unwrap();

    let date = NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap();
    let tx_view = db_conn
        .fetch_txs_with_date(date, FetchNature::Monthly)
        .unwrap();

    tx_view.get_tx(tx_view.len() - 1).clone()
}
