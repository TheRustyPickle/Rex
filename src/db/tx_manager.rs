use crate::db::{
    get_all_tx_methods, get_last_balance_id, get_last_balances, get_last_time_balance,
    get_last_tx_id,
};
use rusqlite::{Connection, Result as sqlResult};
use std::collections::HashMap;

/// Adds a transaction to the database with the given info. The flow of this goes like this:
/// - Add the new transaction to the database
/// - Calculate the changes that happened to the Tx Method
/// - Calculate the absolute final balance
/// - Find the Changes that happened due to the transaction
/// - Push them to the database
pub fn add_new_tx(
    date: &str,
    details: &str,
    tx_method: &str,
    amount: &str,
    tx_type: &str,
    path: &str,
    id_num: Option<&str>,
) -> sqlResult<()> {
    // create a connection and a savepoint
    let mut conn = Connection::open(path)?;
    let sp = conn.savepoint()?;

    if let Some(id) = id_num {
        let query = r#"INSERT INTO tx_all (date, details, "tx_method", amount, tx_type, id_num) VALUES (?, ?, ?, ?, ?, ?)"#;
        sp.execute(&query, [date, details, tx_method, amount, tx_type, id])?;
    } else {
        let query = r#"INSERT INTO tx_all (date, details, "tx_method", amount, tx_type) VALUES (?, ?, ?, ?, ?)"#;
        sp.execute(&query, [date, details, tx_method, amount, tx_type])?;
    }

    let split = date.split("-");
    let vec = split.collect::<Vec<&str>>();
    let mut mnth = vec[1].to_string();
    if &mnth[0..0] == "0" {
        mnth = mnth.replace("0", "");
    }
    let month = mnth.parse::<i32>().unwrap();
    let year = vec[0][2..].parse::<i32>().unwrap() - 22;

    let mut from_method = String::new();
    let mut to_method = String::new();

    if tx_type == "Transfer" {
        let split = tx_method.split(" to ");
        let vec = split.collect::<Vec<&str>>();
        from_method = vec[0].to_string();
        to_method = vec[1].to_string();
    }

    let target_id_num = month as i32 + (year as i32 * 12);

    // This is necessary for the foreign key field in the changes_all table
    // and must align with the latest transaction id_num
    let mut last_id = get_last_tx_id(&sp)?;
    if let Some(id) = id_num {
        last_id = id.parse().unwrap();
    }
    let last_balance_id = get_last_balance_id(&sp)?;

    // we have to get these following data to push to the database
    // new_balance_data : the current month balance after the transaction
    // new_changes_data : the new changes data to push to the database
    // last_balance_data : the absolute final balance after all transaction
    let mut new_balance_data = Vec::new();
    let mut new_changes_data = Vec::new();
    let mut last_balance_data = HashMap::new();

    let all_tx_methods = get_all_tx_methods(&sp);
    let last_balance = get_last_balances(&sp, &all_tx_methods);
    let mut cu_month_balance =
        get_last_time_balance(&sp, month as usize, year as usize, &all_tx_methods);

    let mut new_balance = 0.0;
    let int_amount = amount.parse::<f64>().unwrap();

    if tx_type == "Transfer" {
        let new_balance_from = cu_month_balance[&from_method] - int_amount;
        let new_balance_to = cu_month_balance[&to_method] + int_amount;
        *cu_month_balance.get_mut(&from_method).unwrap() = new_balance_from;
        *cu_month_balance.get_mut(&to_method).unwrap() = new_balance_to;
    } else {
        // makes changes to the current month balance and push them to vector
        if tx_type == "Expense" {
            new_balance = cu_month_balance[tx_method] - int_amount;
        } else if tx_type == "Income" {
            new_balance = cu_month_balance[tx_method] + int_amount;
        }

        *cu_month_balance.get_mut(tx_method).unwrap() = new_balance;
    }

    for i in &all_tx_methods {
        new_balance_data.push(format!("{:.2}", cu_month_balance[i]))
    }

    for i in 0..all_tx_methods.len() {
        // the variable to keep track whether any changes were made to the tx method
        let cu_last_balance = last_balance[i].parse::<f64>().unwrap();
        let mut default_change = format!("{:.2}", 0.0);

        // we could have just used the tx_method from the argument but adding the default values
        // manually after that would make it tricky because have to maintain the tx method balance order
        // and the Changes order

        // add the proper values and changes based on the tx type
        if tx_type == "Transfer" && &all_tx_methods[i] == &from_method {
            default_change = format!("↓{:.2}", &int_amount);
            let edited_balance = cu_last_balance - int_amount;
            last_balance_data.insert(&from_method, format!("{edited_balance:.2}"));
        } else if tx_type == "Transfer" && &all_tx_methods[i] == &to_method {
            default_change = format!("↑{:.2}", &int_amount);
            let edited_balance = cu_last_balance + int_amount;
            last_balance_data.insert(&to_method, format!("{edited_balance:.2}"));
        } else if tx_type != "Transfer" && &all_tx_methods[i] == &tx_method {
            if tx_type == "Expense" {
                default_change = format!("↓{:.2}", &int_amount);
                let edited_balance = cu_last_balance - int_amount;
                last_balance_data.insert(&all_tx_methods[i], format!("{edited_balance:.2}"));
            } else if tx_type == "Income" {
                default_change = format!("↑{:.2}", &int_amount);
                let edited_balance = cu_last_balance + int_amount;
                last_balance_data.insert(&all_tx_methods[i], format!("{edited_balance:.2}"));
            }
        }
        new_changes_data.push(default_change);
    }

    // the query kept on breaking for a single comma so had to follow this ugly way to do this.
    // loop and add a comma until the last index and ignore it in the last time
    let mut balance_query = format!("UPDATE balance_all SET ");
    for i in 0..new_balance_data.len() {
        if i != new_balance_data.len() - 1 {
            balance_query.push_str(&format!(
                r#""{}" = "{}", "#,
                all_tx_methods[i], new_balance_data[i]
            ))
        } else {
            balance_query.push_str(&format!(
                r#""{}" = "{}" "#,
                all_tx_methods[i], new_balance_data[i]
            ))
        }
    }
    balance_query.push_str(&format!("WHERE id_num = {target_id_num}"));

    let last_balance_query: String;

    if tx_type == "Transfer" {
        last_balance_query = format!(
            r#"UPDATE balance_all SET "{from_method}" = "{}", "{to_method}" = "{}" WHERE id_num = {}"#,
            last_balance_data[&from_method], last_balance_data[&to_method], last_balance_id
        );
    } else {
        last_balance_query = format!(
            r#"UPDATE balance_all SET "{tx_method}" = "{}" WHERE id_num = {}"#,
            last_balance_data[&tx_method.to_string()],
            last_balance_id
        );
    }

    let mut changes_query = format!("INSERT INTO changes_all (id_num, date, {all_tx_methods:?}) VALUES ({last_id}, ?, {new_changes_data:?})");
    changes_query = changes_query.replace("[", "");
    changes_query = changes_query.replace("]", "");

    sp.execute(&balance_query, [])?;
    sp.execute(&last_balance_query, [])?;
    sp.execute(&changes_query, [date])?;
    sp.commit()?;
    Ok(())
}

/// Updates the absolute final balance, balance data and deletes the selected transaction.
/// Foreign key cascade takes care of the Changes data in the database.
pub fn delete_tx(id_num: usize, path: &str) -> sqlResult<()> {
    let mut conn = Connection::open(path)?;
    let sp = conn.savepoint()?;

    let tx_methods = get_all_tx_methods(&sp);
    let last_balance = get_last_balances(&sp, &tx_methods);
    let last_balance_id = get_last_balance_id(&sp)?;

    let mut final_last_balance = Vec::new();

    // get the deletion tx data
    let query = format!("SELECT * FROM tx_all Where id_num = {}", id_num);
    let data = sp.query_row(&query, [], |row| {
        let mut final_data: Vec<String> = Vec::new();
        final_data.push(row.get(0)?);
        final_data.push(row.get(2)?);
        final_data.push(row.get(3)?);
        final_data.push(row.get(4)?);
        Ok(final_data)
    })?;

    let split = data[0].split("-");
    let splitted = split.collect::<Vec<&str>>();
    let (year, month) = (
        splitted[0].parse::<i32>().unwrap(),
        splitted[1].parse::<i32>().unwrap(),
    );

    let year = year - 2022;

    let mut target_id_num = month as i32 + (year as i32 * 12);

    //
    let mut from_method = "";
    let mut to_method = "";

    // the tx_method of the tx
    let source = &data[1];

    // execute this block to get block tx method if the tx type is a Transfer
    if source.contains(" to ") {
        let from_to = data[1].split(" to ").collect::<Vec<&str>>();

        from_method = from_to[0];
        to_method = from_to[1];
    }

    let amount = &data[2].parse::<f64>().unwrap();
    let tx_type: &str = &data[3];

    // loop through all rows in the balance_all table from the deletion point and update balance
    loop {
        let mut query = format!(
            "SELECT {:?} FROM balance_all WHERE id_num = {}",
            tx_methods, target_id_num
        );
        query = query.replace("[", "");
        query = query.replace("]", "");

        let cu_month_balance = sp.query_row(&query, [], |row| {
            let mut final_data: Vec<String> = Vec::new();
            for i in 0..tx_methods.len() {
                final_data.push(row.get(i)?)
            }
            Ok(final_data)
        })?;

        let mut updated_month_balance = vec![];

        // reverse that amount that was previously added and commit them to db
        // add or subtract based on the tx type to the relevant method

        // check the month balance as not zero because if it is 0, there was never any transaction
        // done on that month
        for i in 0..tx_methods.len() {
            if &tx_methods[i] == source && cu_month_balance[i] != "0.00" {
                let mut cu_int_amount = cu_month_balance[i].parse::<f64>().unwrap();
                if tx_type == "Expense" {
                    cu_int_amount += amount;
                } else if tx_type == "Income" {
                    cu_int_amount -= amount;
                }
                updated_month_balance.push(format!("{:.2}", cu_int_amount));
            } else if &tx_methods[i] == from_method && cu_month_balance[i] != "0.00" {
                let mut cu_int_amount = cu_month_balance[i].parse::<f64>().unwrap();
                cu_int_amount += amount;
                updated_month_balance.push(format!("{:.2}", cu_int_amount));
            } else if &tx_methods[i] == to_method && cu_month_balance[i] != "0.00" {
                let mut cu_int_amount = cu_month_balance[i].parse::<f64>().unwrap();
                cu_int_amount -= amount;
                updated_month_balance.push(format!("{:.2}", cu_int_amount));
            } else {
                updated_month_balance.push(format!(
                    "{:.2}",
                    cu_month_balance[i].parse::<f64>().unwrap()
                ));
            }
        }

        // the query kept on breaking for a single comma so had to follow this ugly way to do this.
        // loop and add a comma until the last index and ignore it in the last time
        let mut balance_query = format!("UPDATE balance_all SET ");
        for i in 0..updated_month_balance.len() {
            if i != updated_month_balance.len() - 1 {
                balance_query.push_str(&format!(
                    r#""{}" = "{}", "#,
                    tx_methods[i], updated_month_balance[i]
                ))
            } else {
                balance_query.push_str(&format!(
                    r#""{}" = "{}" "#,
                    tx_methods[i], updated_month_balance[i]
                ))
            }
        }
        balance_query.push_str(&format!("WHERE id_num = {target_id_num}"));
        sp.execute(&balance_query, [])?;

        // 49 is the absolute final balance which we don't need to modify
        target_id_num += 1;
        if target_id_num == 49 {
            break;
        }
    }

    // we are deleting 1 transaction, so loop through all tx methods, and whichever method matches
    // with the one we are deleting, add/subtract from the amount.
    // Calculate the balance/s for the absolute final balance and create the query
    for i in 0..tx_methods.len() {
        let mut cu_balance = last_balance[i].parse::<f64>().unwrap();
        if &tx_methods[i] == source && tx_type != "Transfer" {
            match tx_type {
                "Expense" => cu_balance += amount,
                "Income" => cu_balance -= amount,
                _ => {}
            }
        } else if &tx_methods[i] == from_method && tx_type == "Transfer" {
            cu_balance += amount;
        } else if &tx_methods[i] == to_method && tx_type == "Transfer" {
            cu_balance -= amount;
        }
        final_last_balance.push(format!("{:.2}", cu_balance));
    }

    let del_query = format!("DELETE FROM tx_all WHERE id_num = {id_num}");

    let mut last_balance_query = format!("UPDATE balance_all SET ");
    for i in 0..final_last_balance.len() {
        if i != final_last_balance.len() - 1 {
            last_balance_query.push_str(&format!(
                r#""{}" = "{}", "#,
                tx_methods[i], final_last_balance[i]
            ))
        } else {
            last_balance_query.push_str(&format!(
                r#""{}" = "{}" "#,
                tx_methods[i], final_last_balance[i]
            ))
        }
    }
    last_balance_query.push_str(&format!("WHERE id_num = {last_balance_id}"));
    sp.execute(&last_balance_query, [])?;
    sp.execute(&del_query, [])?;

    sp.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::create_db;
    use std::fs;

    fn create_test_db(file_name: &str) -> Connection {
        create_db(file_name, vec!["test1".to_string(), "test 2".to_string()]).unwrap();
        return Connection::open(file_name).unwrap();
    }

    #[test]
    fn check_last_month_balance_2() {
        let file_name = "last_month_balance_2.sqlite".to_string();
        let conn = create_test_db(&file_name);
        let tx_methods = get_all_tx_methods(&conn);

        add_new_tx(
            "2022-07-19",
            "Testing transaction",
            "test1",
            "100.00",
            "Income",
            &file_name,
            None,
        )
        .unwrap();

        add_new_tx(
            "2022-07-19",
            "Testing transaction",
            "test 2",
            "100.00",
            "Income",
            &file_name,
            None,
        )
        .unwrap();

        add_new_tx(
            "2022-08-19",
            "Testing transaction",
            "test1",
            "100.00",
            "Income",
            &file_name,
            None,
        )
        .unwrap();

        add_new_tx(
            "2022-09-19",
            "Testing transaction",
            "test1",
            "100.00",
            "Income",
            &file_name,
            None,
        )
        .unwrap();

        add_new_tx(
            "2022-10-19",
            "Testing transaction",
            "test1",
            "100.00",
            "Income",
            &file_name,
            None,
        )
        .unwrap();

        let data_1 = get_last_time_balance(&conn, 8, 0, &tx_methods);
        let expected_data_1 =
            HashMap::from([("test 2".to_string(), 100.0), ("test1".to_string(), 200.0)]);

        delete_tx(1, &file_name).unwrap();
        delete_tx(2, &file_name).unwrap();

        let data_2 = get_last_time_balance(&conn, 10, 3, &tx_methods);
        let expected_data_2 =
            HashMap::from([("test 2".to_string(), 0.0), ("test1".to_string(), 300.0)]);

        conn.close().unwrap();
        fs::remove_file(file_name).unwrap();

        assert_eq!(data_1, expected_data_1);
        assert_eq!(data_2, expected_data_2);
    }

    #[test]
    fn check_last_tx_id_2() {
        let file_name = "last_tx_id_2.sqlite".to_string();
        let conn = create_test_db(&file_name);

        add_new_tx(
            "2022-09-19",
            "Testing transaction",
            "test1",
            "100.00",
            "Income",
            &file_name,
            None,
        )
        .unwrap();

        let data = get_last_tx_id(&conn);
        let expected_data: sqlResult<i32> = Ok(1);

        conn.close().unwrap();
        fs::remove_file(file_name).unwrap();

        assert_eq!(data, expected_data);
    }
}
