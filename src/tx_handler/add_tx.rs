use crate::utility::{
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
pub fn add_tx(
    date: &str,
    details: &str,
    tx_method: &str,
    amount: &str,
    tx_type: &str,
    tags: &str,
    path: &str,
    id_num: Option<&str>,
) -> sqlResult<()> {
    // create a connection and a savepoint
    let mut conn = Connection::open(path)?;
    let sp = conn.savepoint()?;

    if let Some(id) = id_num {
        let query = r#"INSERT INTO tx_all (date, details, "tx_method", amount, tx_type, id_num, tags) VALUES (?, ?, ?, ?, ?, ?, ?)"#;
        sp.execute(query, [date, details, tx_method, amount, tx_type, id, tags])?;
    } else {
        let query = r#"INSERT INTO tx_all (date, details, "tx_method", amount, tx_type, tags) VALUES (?, ?, ?, ?, ?, ?)"#;
        sp.execute(query, [date, details, tx_method, amount, tx_type, tags])?;
    }

    let splitted = date.split('-').collect::<Vec<&str>>();
    let mut mnth = splitted[1].to_string();
    if &mnth[0..0] == "0" {
        mnth = mnth.replace('0', "");
    }
    let month = mnth.parse::<i32>().unwrap();
    let year = splitted[0][2..].parse::<i32>().unwrap() - 22;

    let mut from_method = String::new();
    let mut to_method = String::new();

    if tx_type == "Transfer" {
        let splitted = tx_method.split(" to ").collect::<Vec<&str>>();
        from_method = splitted[0].to_string();
        to_method = splitted[1].to_string();
    }

    let target_id_num = month + (year * 12);

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
    let mut current_month_balance =
        get_last_time_balance(&sp, month as usize, year as usize, &all_tx_methods);

    let mut new_balance = 0.0;
    let int_amount = amount.parse::<f64>().unwrap();

    if tx_type == "Transfer" {
        let new_balance_from = current_month_balance[&from_method] - int_amount;
        let new_balance_to = current_month_balance[&to_method] + int_amount;
        *current_month_balance.get_mut(&from_method).unwrap() = new_balance_from;
        *current_month_balance.get_mut(&to_method).unwrap() = new_balance_to;
    } else {
        // makes changes to the current month balance and push them to vector
        if tx_type == "Expense" {
            new_balance = current_month_balance[tx_method] - int_amount;
        } else if tx_type == "Income" {
            new_balance = current_month_balance[tx_method] + int_amount;
        }

        *current_month_balance.get_mut(tx_method).unwrap() = new_balance;
    }

    for i in &all_tx_methods {
        new_balance_data.push(format!("{:.2}", current_month_balance[i]))
    }

    for i in 0..all_tx_methods.len() {
        // the variable to keep track whether any changes were made to the tx method
        let current_last_balance = last_balance[i].parse::<f64>().unwrap();
        let mut default_change = format!("{:.2}", 0.0);

        // we could have just used the tx_method from the argument but adding the default values
        // manually after that would make it tricky because have to maintain the tx method balance order
        // and the Changes order

        // add the proper values and changes based on the tx type
        if tx_type == "Transfer" && all_tx_methods[i] == from_method {
            default_change = format!("↓{:.2}", &int_amount);
            let edited_balance = current_last_balance - int_amount;
            last_balance_data.insert(&from_method, format!("{edited_balance:.2}"));
        } else if tx_type == "Transfer" && all_tx_methods[i] == to_method {
            default_change = format!("↑{:.2}", &int_amount);
            let edited_balance = current_last_balance + int_amount;
            last_balance_data.insert(&to_method, format!("{edited_balance:.2}"));
        } else if tx_type != "Transfer" && all_tx_methods[i] == tx_method {
            if tx_type == "Expense" {
                default_change = format!("↓{:.2}", &int_amount);
                let edited_balance = current_last_balance - int_amount;
                last_balance_data.insert(&all_tx_methods[i], format!("{edited_balance:.2}"));
            } else if tx_type == "Income" {
                default_change = format!("↑{:.2}", &int_amount);
                let edited_balance = current_last_balance + int_amount;
                last_balance_data.insert(&all_tx_methods[i], format!("{edited_balance:.2}"));
            }
        }
        new_changes_data.push(default_change);
    }

    // the query kept on breaking for a single comma so had to follow this ugly way to do this.
    // loop and add a comma until the last index and ignore it in the last time
    let mut balance_query = "UPDATE balance_all SET ".to_string();
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

    let last_balance_query: String = if tx_type == "Transfer" {
        format!(
            r#"UPDATE balance_all SET "{from_method}" = "{}", "{to_method}" = "{}" WHERE id_num = {}"#,
            last_balance_data[&from_method], last_balance_data[&to_method], last_balance_id
        )
    } else {
        format!(
            r#"UPDATE balance_all SET "{tx_method}" = "{}" WHERE id_num = {}"#,
            last_balance_data[&tx_method.to_string()],
            last_balance_id
        )
    };

    let mut changes_query = format!("INSERT INTO changes_all (id_num, date, {all_tx_methods:?}) VALUES ({last_id}, ?, {new_changes_data:?})");
    changes_query = changes_query.replace('[', "");
    changes_query = changes_query.replace(']', "");

    sp.execute(&balance_query, [])?;
    sp.execute(&last_balance_query, [])?;
    sp.execute(&changes_query, [date])?;
    sp.commit()?;
    Ok(())
}
