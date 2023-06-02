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
    id_num: Option<&str>,
    conn: &mut Connection,
) -> sqlResult<()> {
    // create a connection and a savepoint
    let sp = conn.savepoint()?;

    // the process goes through 4 parts
    // Add the tx itself in the db
    // calculate the amount to add/subtract from the balance_all table
    // create changes amount and push it to db
    // Update final row balance which holds the balance after all tx

    // if Some(id) means it's a transaction editing
    // else it's a normal transaction
    if let Some(id) = id_num {
        let query = r#"INSERT INTO tx_all (date, details, "tx_method", amount, tx_type, id_num, tags) VALUES (?, ?, ?, ?, ?, ?, ?)"#;
        sp.execute(query, [date, details, tx_method, amount, tx_type, id, tags])?;
    } else {
        let query = r#"INSERT INTO tx_all (date, details, "tx_method", amount, tx_type, tags) VALUES (?, ?, ?, ?, ?, ?)"#;
        sp.execute(query, [date, details, tx_method, amount, tx_type, tags])?;
    }

    // 2025-05-10
    // take 2025 and subtract 2022 = 3, means the year number 3
    // take 05 -> 5 -> 5th month. 5 + (3 * 12) =  the row of this month's balance on balance_all table
    // we are not subtracting 1 from month because balance_all table starts at 1
    let splitted = date.split('-').collect::<Vec<&str>>();
    let (year, month) = (
        splitted[0].parse::<i32>().unwrap() - 2022,
        splitted[1].parse::<i32>().unwrap(),
    );

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
    // new_balance_data: the working month balance after the transaction
    // new_changes_data: the new changes data to push to the database after this tx
    // last_balance_data: the absolute final balance after all transaction
    let mut new_balance_data = Vec::new();
    let mut new_changes_data = Vec::new();
    let mut last_balance_data = HashMap::new();

    let all_tx_methods = get_all_tx_methods(&sp);
    let last_balance = get_last_balances(&sp);

    // Retrieve the current month's balance for each transaction method.
    let mut current_month_balance =
        get_last_time_balance(month as usize, year as usize, &all_tx_methods, &sp);

    let int_amount = amount.parse::<f64>().unwrap();

    // Update the current month's balance based on the transaction type.
    match tx_type {
        "Transfer" => {
            let new_balance_from = current_month_balance[&from_method] - int_amount;
            let new_balance_to = current_month_balance[&to_method] + int_amount;

            // Update the current month's balance for both the "from" and "to" methods.
            *current_month_balance.get_mut(&from_method).unwrap() = new_balance_from;
            *current_month_balance.get_mut(&to_method).unwrap() = new_balance_to;
        }
        "Expense" => {
            let new_balance = current_month_balance[tx_method] - int_amount;

            // Update the current month's balance for the relevant method.
            *current_month_balance.get_mut(tx_method).unwrap() = new_balance;
        }
        "Income" => {
            let new_balance = current_month_balance[tx_method] + int_amount;
            // Update the current month's balance for the relevant method.
            *current_month_balance.get_mut(tx_method).unwrap() = new_balance;
        }
        _ => {}
    }

    // Add the current month's balances to the new balance data vector.
    // * It's done this way to match the tx method location
    for i in &all_tx_methods {
        new_balance_data.push(format!("{:.2}", current_month_balance[i]))
    }

    //
    for i in 0..all_tx_methods.len() {
        // the variable to keep track whether any changes were made to the tx method
        let current_last_balance = last_balance[i].parse::<f64>().unwrap();
        let mut current_change = format!("{:.2}", 0.0);

        // add the proper values and changes based on the tx type
        if tx_type == "Transfer" && all_tx_methods[i] == from_method {
            current_change = format!("↓{:.2}", &int_amount);

            let edited_balance = current_last_balance - int_amount;
            last_balance_data.insert(&from_method, format!("{edited_balance:.2}"));
        } else if tx_type == "Transfer" && all_tx_methods[i] == to_method {
            current_change = format!("↑{:.2}", &int_amount);

            let edited_balance = current_last_balance + int_amount;
            last_balance_data.insert(&to_method, format!("{edited_balance:.2}"));
        } else if tx_type != "Transfer" && all_tx_methods[i] == tx_method {
            if tx_type == "Expense" {
                current_change = format!("↓{:.2}", &int_amount);

                let edited_balance = current_last_balance - int_amount;
                last_balance_data.insert(&all_tx_methods[i], format!("{edited_balance:.2}"));
            } else if tx_type == "Income" {
                current_change = format!("↑{:.2}", &int_amount);

                let edited_balance = current_last_balance + int_amount;
                last_balance_data.insert(&all_tx_methods[i], format!("{edited_balance:.2}"));
            }
        }
        new_changes_data.push(current_change);
    }

    let set_values = all_tx_methods
        .iter()
        .zip(new_balance_data.iter())
        .map(|(method, value)| format!(r#""{}" = "{}""#, method, value))
        .collect::<Vec<_>>()
        .join(", ");

    let balance_query = format!(
        "UPDATE balance_all SET {} WHERE id_num = {}",
        set_values, target_id_num
    );

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

    let changes_query = format!(
        "INSERT INTO changes_all (id_num, date, {}) VALUES ({}, ?, {})",
        all_tx_methods
            .iter()
            .map(|s| format!(r#""{}""#, s))
            .collect::<Vec<_>>()
            .join(", "),
        last_id,
        new_changes_data
            .iter()
            .map(|s| format!(r#""{}""#, s))
            .collect::<Vec<_>>()
            .join(", ")
    );

    sp.execute(&balance_query, [])?;
    sp.execute(&last_balance_query, [])?;
    sp.execute(&changes_query, [date])?;
    sp.commit()?;
    Ok(())
}
