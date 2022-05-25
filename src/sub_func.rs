use rusqlite::{Connection, Result};
use std::collections::HashMap;

pub fn get_all_tx_methods(conn: &Connection) -> Vec<String> {
    let column_names = conn.prepare("SELECT * FROM balance_all").expect("could not prepare statement");
    let mut tx_methods = vec![];
    for i in 1..99 {
        let column = column_names.column_name(i);
        match column {
            Ok(c) => tx_methods.push(c.to_string()),
            Err(_) => break,
        }
    }
    tx_methods
}

pub fn get_all_txs(conn: &Connection, month: usize, year: usize) -> HashMap<i32, Vec<String>> {
    let mut id_result: Vec<i32> = Vec::new();
    let mut final_result = HashMap::new();
    
    let mut new_month:String = month.to_string();
    let mut new_year:String = year.to_string();

    if month+1 < 10 {
        new_month = format!("0{}", month+1);
    }

    if year+1 < 10 {
        new_year = format!("202{}", year+1);
    }

    let datetime_1 = format!("{}-{}-01", new_year, new_month);
    let datetime_2 = format!("{}-{}-31", new_year, new_month);

    let mut statement = conn.prepare("SELECT id_num FROM tx_all Where date BETWEEN date(?) AND date(?)").expect("could not prepare statement");
    let rows = statement.query_map([&datetime_1,&datetime_2], |row| {
        Ok(row.get(0).unwrap())
    }).expect("Error");

    for i in rows {
        id_result.push(i.unwrap());
    }

    let mut statement = conn.prepare("SELECT * FROM tx_all Where date BETWEEN date(?) AND date(?)").expect("could not prepare statement");
    
    let rows = statement.query_map([&datetime_1,&datetime_2], |row| {
        let date: String = row.get(0).unwrap();
        let splited_date = date.split('-');
        let collected_date: Vec<&str> = splited_date.collect(); 
        let new_date = format!("{}-{}-{}", collected_date[2], collected_date[1], collected_date[0]);
        Ok(vec![new_date, row.get(1).unwrap(), row.get(2).unwrap(), row.get(3).unwrap(), row.get(4).unwrap()])
    }).expect("Error");

    let mut cu_index = 0;
    for i in rows {
        final_result.insert(id_result[cu_index], i.unwrap());
        cu_index += 1;
    }

    final_result
}

pub fn get_all_balance(conn: &Connection, month: usize, year: usize) -> HashMap<i32, Vec<String>> {
    let mut id_result: Vec<i32> = Vec::new();
    let mut final_result = HashMap::new();
    let tx_methods = get_all_tx_methods(conn);

    let mut new_month:String = month.to_string();
    let mut new_year:String = year.to_string();

    if month+1 < 10 {
        new_month = format!("0{}", month+1);
    }

    if year+1 < 10 {
        new_year = format!("202{}", year+1);
    }

    let datetime_1 = format!("{}-{}-01", new_year, new_month);
    let datetime_2 = format!("{}-{}-31", new_year, new_month);

    let mut statement = conn.prepare("SELECT id_num FROM balance_all").expect("could not prepare statement");
    let rows = statement.query_map([], |row| {
        Ok(row.get(0).unwrap())
    }).expect("Error");

    for i in rows {
        id_result.push(i.unwrap());
    }

    let mut statement = conn.prepare("SELECT * FROM balance_all").expect("could not prepare statement");

    let rows = statement.query_map([], |row| {
        let mut balance_vec: Vec<String> = Vec::new();
        for i in 1..tx_methods.len() {
            balance_vec.push(row.get(i).unwrap());
        }
        Ok(balance_vec)
    }).expect("Error");

    let mut cu_index = 0;
    for i in rows {
        final_result.insert(id_result[cu_index], i.unwrap());
        cu_index += 1;
    }

    final_result
}

pub fn get_all_changes(conn: &Connection, month: usize, year: usize) -> HashMap<i32, Vec<String>> {
    let mut id_result: Vec<i32> = Vec::new();
    let mut final_result = HashMap::new();
    let tx_methods = get_all_tx_methods(conn);

    let mut statement = conn.prepare("SELECT id_num FROM changes_all").expect("could not prepare statement");
    let rows = statement.query_map([], |row| {
        Ok(row.get(0).unwrap())
    }).expect("Error");

    for i in rows {
        id_result.push(i.unwrap());
    }

    let mut statement = conn.prepare("SELECT * FROM changes_all").expect("could not prepare statement");

    let rows = statement.query_map([], |row| {
        let mut balance_vec: Vec<String> = Vec::new();
        for i in 1..tx_methods.len() {
            balance_vec.push(row.get(i).unwrap());
        }
        Ok(balance_vec)
    }).expect("Error");

    let mut cu_index = 0;
    for i in rows {
        final_result.insert(id_result[cu_index], i.unwrap());
        cu_index += 1;
    }

    final_result
}

pub fn test_get_all_txs (conn: &Connection, month: usize, year: usize) -> Vec<Vec<String>> {
    let mut final_result: Vec<Vec<String>> = Vec::new();
    
    let mut new_month:String = month.to_string();
    let mut new_year:String = year.to_string();
    
    if month < 10 {
        new_month = format!("0{}", month+1);
    }

    if year+1 < 10 {
        new_year = format!("202{}", year+1);
    }

    let datetime_1 = format!("{}-{}-01", new_year, new_month);
    let datetime_2 = format!("{}-{}-31", new_year, new_month);

    let mut statement = conn.prepare("SELECT * FROM tx_all Where date BETWEEN date(?) AND date(?)").expect("could not prepare statement");
    
    let rows = statement.query_map([&datetime_1,&datetime_2], |row| {
        let date: String = row.get(0).unwrap();
        let splited_date = date.split('-');
        let collected_date: Vec<&str> = splited_date.collect(); 
        let new_date = format!("{}-{}-{}", collected_date[2], collected_date[1], collected_date[0]);

        Ok(vec![new_date, row.get(1).unwrap(), row.get(2).unwrap(), row.get(3).unwrap(), row.get(4).unwrap()])
    }).expect("Error");

    for i in rows {
        final_result.push(i.unwrap())
        
    }

    final_result
}