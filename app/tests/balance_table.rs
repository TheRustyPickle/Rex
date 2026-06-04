use chrono::NaiveDate;
use rex_app::conn::FetchNature;
use rex_db::ConnCache;
use rex_db::models::Balance;
use rex_shared::models::Cent;
use std::collections::HashMap;
use std::fs;

use crate::common::{add_tx, create_test_db};

mod common;

fn get_month_balance_map(
    db_conn: &mut rex_app::conn::DbConn,
    year: i32,
    month: u32,
) -> HashMap<String, Cent> {
    let date = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let map = Balance::get_balance_map(date, db_conn).unwrap();
    let mut result = HashMap::new();
    for method in db_conn.cache().tx_methods.values() {
        let balance = map
            .get(&method.id)
            .map(|b| Cent::new(b.balance))
            .unwrap_or(Cent::new(0));
        result.insert(method.name.clone(), balance);
    }
    result
}

fn get_last_balance(
    db_conn: &mut rex_app::conn::DbConn,
    year: i32,
    month: u32,
    nature: FetchNature,
) -> HashMap<String, Cent> {
    let date = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let map = Balance::get_last_balance(date, nature, db_conn).unwrap();
    let mut result = HashMap::new();
    for method in db_conn.cache().tx_methods.values() {
        result.insert(method.name.clone(), map[&method.id]);
    }
    result
}

fn get_final_balance_map(db_conn: &mut rex_app::conn::DbConn) -> HashMap<String, Cent> {
    let map = Balance::get_final_balance(db_conn).unwrap();
    let mut result = HashMap::new();
    for method in db_conn.cache().tx_methods.values() {
        result.insert(method.name.clone(), Cent::new(map[&method.id].balance));
    }
    result
}

#[test]
fn balance_table_add_tx_updates_current_month() {
    let file_name = "test_balance_add.sqlite";
    let mut db_conn = create_test_db(file_name);

    assert_eq!(get_final_balance_map(&mut db_conn)["Cash"], Cent::new(0));

    add_tx(
        &mut db_conn,
        "2024-07-15",
        "Income",
        "Cash",
        "",
        "500.00",
        "Income",
        "Salary",
    );

    assert_eq!(
        get_month_balance_map(&mut db_conn, 2024, 7)["Cash"],
        Cent::new(50000)
    );
    assert_eq!(
        get_final_balance_map(&mut db_conn)["Cash"],
        Cent::new(50000)
    );

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn balance_table_sequential_months_cascade() {
    let file_name = "test_balance_months.sqlite";
    let mut db_conn = create_test_db(file_name);

    add_tx(
        &mut db_conn,
        "2024-07-01",
        "July",
        "Cash",
        "",
        "500.00",
        "Income",
        "A",
    );
    add_tx(
        &mut db_conn,
        "2024-08-15",
        "August",
        "Cash",
        "",
        "300.00",
        "Income",
        "B",
    );

    assert_eq!(
        get_month_balance_map(&mut db_conn, 2024, 7)["Cash"],
        Cent::new(50000)
    );
    assert_eq!(
        get_month_balance_map(&mut db_conn, 2024, 8)["Cash"],
        Cent::new(80000)
    );

    assert_eq!(
        get_final_balance_map(&mut db_conn)["Cash"],
        Cent::new(80000)
    );

    let sep_last = get_last_balance(&mut db_conn, 2024, 9, FetchNature::Monthly);
    assert_eq!(sep_last["Cash"], Cent::new(80000));

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn balance_table_delete_cascades_forward() {
    let file_name = "test_balance_delete.sqlite";
    let mut db_conn = create_test_db(file_name);

    add_tx(
        &mut db_conn,
        "2024-07-01",
        "July",
        "Cash",
        "",
        "500.00",
        "Income",
        "A",
    );
    let aug_tx = add_tx(
        &mut db_conn,
        "2024-08-01",
        "August",
        "Cash",
        "",
        "300.00",
        "Income",
        "B",
    );
    add_tx(
        &mut db_conn,
        "2024-09-01",
        "Sept",
        "Cash",
        "",
        "200.00",
        "Income",
        "C",
    );

    assert_eq!(
        get_month_balance_map(&mut db_conn, 2024, 7)["Cash"],
        Cent::new(50000)
    );
    assert_eq!(
        get_month_balance_map(&mut db_conn, 2024, 8)["Cash"],
        Cent::new(80000)
    );
    assert_eq!(
        get_month_balance_map(&mut db_conn, 2024, 9)["Cash"],
        Cent::new(100000)
    );

    db_conn.delete_tx(&aug_tx).unwrap();

    assert_eq!(
        get_month_balance_map(&mut db_conn, 2024, 7)["Cash"],
        Cent::new(50000)
    );
    assert_eq!(
        get_month_balance_map(&mut db_conn, 2024, 8)["Cash"],
        Cent::new(50000)
    );
    assert_eq!(
        get_month_balance_map(&mut db_conn, 2024, 9)["Cash"],
        Cent::new(70000)
    );
    assert_eq!(
        get_final_balance_map(&mut db_conn)["Cash"],
        Cent::new(70000)
    );

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn balance_table_edit_cascades_forward() {
    let file_name = "test_balance_edit.sqlite";
    let mut db_conn = create_test_db(file_name);

    add_tx(
        &mut db_conn,
        "2024-07-01",
        "July",
        "Cash",
        "",
        "500.00",
        "Income",
        "A",
    );
    let aug_tx = add_tx(
        &mut db_conn,
        "2024-08-01",
        "August",
        "Cash",
        "",
        "300.00",
        "Income",
        "B",
    );
    add_tx(
        &mut db_conn,
        "2024-09-01",
        "Sept",
        "Cash",
        "",
        "200.00",
        "Income",
        "C",
    );

    let new_tx = rex_app::modifier::parse_tx_fields(
        "2024-08-01",
        "August",
        "Cash",
        "",
        "100.00",
        "Income",
        &db_conn,
    )
    .unwrap();
    db_conn.edit_tx(&aug_tx, new_tx, "B").unwrap();

    assert_eq!(
        get_month_balance_map(&mut db_conn, 2024, 7)["Cash"],
        Cent::new(50000)
    );
    assert_eq!(
        get_month_balance_map(&mut db_conn, 2024, 8)["Cash"],
        Cent::new(60000)
    );
    assert_eq!(
        get_month_balance_map(&mut db_conn, 2024, 9)["Cash"],
        Cent::new(80000)
    );
    assert_eq!(
        get_final_balance_map(&mut db_conn)["Cash"],
        Cent::new(80000)
    );

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn balance_table_mid_insert_updates_forward() {
    let file_name = "test_balance_mid.sqlite";
    let mut db_conn = create_test_db(file_name);

    add_tx(
        &mut db_conn,
        "2024-01-15",
        "Jan",
        "Cash",
        "",
        "1000.00",
        "Income",
        "A",
    );
    add_tx(
        &mut db_conn,
        "2024-03-10",
        "Mar",
        "Cash",
        "",
        "200.00",
        "Expense",
        "B",
    );

    assert_eq!(
        get_month_balance_map(&mut db_conn, 2024, 1)["Cash"],
        Cent::new(100000)
    );
    assert_eq!(
        get_month_balance_map(&mut db_conn, 2024, 3)["Cash"],
        Cent::new(80000)
    );

    add_tx(
        &mut db_conn,
        "2024-02-10",
        "Feb",
        "Cash",
        "",
        "300.00",
        "Income",
        "C",
    );

    assert_eq!(
        get_month_balance_map(&mut db_conn, 2024, 1)["Cash"],
        Cent::new(100000)
    );
    assert_eq!(
        get_month_balance_map(&mut db_conn, 2024, 2)["Cash"],
        Cent::new(130000)
    );
    assert_eq!(
        get_month_balance_map(&mut db_conn, 2024, 3)["Cash"],
        Cent::new(110000)
    );
    assert_eq!(
        get_final_balance_map(&mut db_conn)["Cash"],
        Cent::new(110000)
    );

    let may_last = get_last_balance(&mut db_conn, 2024, 5, FetchNature::Monthly);
    assert_eq!(may_last["Cash"], Cent::new(110000));

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn balance_table_multiple_methods() {
    let file_name = "test_balance_methods.sqlite";
    let mut db_conn = create_test_db(file_name);

    add_tx(
        &mut db_conn,
        "2024-04-01",
        "Income",
        "Cash",
        "",
        "1000.00",
        "Income",
        "Work",
    );
    add_tx(
        &mut db_conn,
        "2024-04-15",
        "Transfer",
        "Cash",
        "Bank",
        "300.00",
        "Transfer",
        "Move",
    );

    let apr = get_month_balance_map(&mut db_conn, 2024, 4);
    assert_eq!(apr["Cash"], Cent::new(70000));
    assert_eq!(apr["Bank"], Cent::new(30000));
    assert_eq!(apr["Other"], Cent::new(0));

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn balance_table_year_boundary_cascades() {
    let file_name = "test_balance_year.sqlite";
    let mut db_conn = create_test_db(file_name);

    add_tx(
        &mut db_conn,
        "2023-12-15",
        "Dec",
        "Cash",
        "",
        "500.00",
        "Income",
        "A",
    );
    add_tx(
        &mut db_conn,
        "2024-01-10",
        "Jan",
        "Cash",
        "",
        "300.00",
        "Income",
        "B",
    );

    assert_eq!(
        get_month_balance_map(&mut db_conn, 2023, 12)["Cash"],
        Cent::new(50000)
    );
    assert_eq!(
        get_month_balance_map(&mut db_conn, 2024, 1)["Cash"],
        Cent::new(80000)
    );
    assert_eq!(
        get_final_balance_map(&mut db_conn)["Cash"],
        Cent::new(80000)
    );

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn balance_table_out_of_order_insert_maintains_cascade() {
    let file_name = "test_balance_ooo.sqlite";
    let mut db_conn = create_test_db(file_name);

    add_tx(
        &mut db_conn,
        "2024-04-01",
        "Apr",
        "Cash",
        "",
        "400.00",
        "Income",
        "D",
    );
    add_tx(
        &mut db_conn,
        "2024-02-01",
        "Feb",
        "Cash",
        "",
        "200.00",
        "Income",
        "B",
    );
    add_tx(
        &mut db_conn,
        "2024-03-01",
        "Mar",
        "Cash",
        "",
        "300.00",
        "Income",
        "C",
    );
    add_tx(
        &mut db_conn,
        "2024-01-01",
        "Jan",
        "Cash",
        "",
        "100.00",
        "Income",
        "A",
    );

    assert_eq!(
        get_month_balance_map(&mut db_conn, 2024, 1)["Cash"],
        Cent::new(10000)
    );
    assert_eq!(
        get_month_balance_map(&mut db_conn, 2024, 2)["Cash"],
        Cent::new(30000)
    );
    assert_eq!(
        get_month_balance_map(&mut db_conn, 2024, 3)["Cash"],
        Cent::new(60000)
    );
    assert_eq!(
        get_month_balance_map(&mut db_conn, 2024, 4)["Cash"],
        Cent::new(100000)
    );
    assert_eq!(
        get_final_balance_map(&mut db_conn)["Cash"],
        Cent::new(100000)
    );

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}

#[test]
fn balance_table_transfer_across_months() {
    let file_name = "test_balance_transfer.sqlite";
    let mut db_conn = create_test_db(file_name);

    add_tx(
        &mut db_conn,
        "2024-05-01",
        "Income",
        "Cash",
        "",
        "1000.00",
        "Income",
        "Salary",
    );
    add_tx(
        &mut db_conn,
        "2024-05-10",
        "Move",
        "Cash",
        "Bank",
        "300.00",
        "Transfer",
        "Move",
    );
    add_tx(
        &mut db_conn,
        "2024-06-01",
        "Move more",
        "Cash",
        "Bank",
        "100.00",
        "Transfer",
        "Move",
    );

    let may = get_month_balance_map(&mut db_conn, 2024, 5);
    assert_eq!(may["Cash"], Cent::new(70000));
    assert_eq!(may["Bank"], Cent::new(30000));

    let jun = get_month_balance_map(&mut db_conn, 2024, 6);
    assert_eq!(jun["Cash"], Cent::new(60000));
    assert_eq!(jun["Bank"], Cent::new(40000));

    drop(db_conn);
    fs::remove_file(file_name).unwrap();
}
