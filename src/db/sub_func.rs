use rusqlite::{Connection, Result as sqlResult};
use std::collections::HashMap;

// This file contains a number of functions that makes calls to the database
// to fetch relevant data which is later users in various structs. I didn't
// wanted to go around multiple files to find that one db call so just put them all together.
// The file also contains non-db functions for generating data and for general utilization as well.   
// All the DB calls are created keeping in mind that the program does not know the amount of
// Transaction Methods that will be added by the user.


/// Makes a call to the database to find out all the columns in the balance_all section
/// so we can determine the number of TX Methods that has been added.
/// `["source_1", "source_2", "source_3"]`
pub fn get_all_tx_methods(conn: &Connection) -> Vec<String> {
    // returns all transaction methods added to the database
    // example bank, cash.
    let column_names = conn
        .prepare("SELECT * FROM balance_all")
        .expect("could not prepare statement");
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

/// The function is used to create dates in the form of strings to use the WHERE statement
/// based on the index that has been passed to it. 
fn get_sql_dates(month: usize, year: usize) -> (String, String) {
    // returns dates from month and year to a format that is suitable for
    // database WHERE statement.

    let new_month: String;
    let mut new_year = year.to_string();

    if month < 10 {
        new_month = format!("0{}", month);
    } else {
        new_month = format!("{}", month);
    }

    if year + 1 < 10 {
        new_year = format!("202{}", year + 2);
    }
    let datetime_1 = format!("{}-{}-01", new_year, new_month);
    let datetime_2 = format!("{}-{}-31", new_year, new_month);
    (datetime_1, datetime_2)
}

/// A function that returns the balance that was saved in the database
/// in the previous month from the current working month. This is necessary
/// because the program tries to add or take away balance based on all the transactions
/// that happened in the current month. So take the previous month balance and do the calculations.
/// 
/// Return Value : `{"source_1": 10.50, "source_2": 100.0}`
fn get_last_month_balance(
    conn: &Connection,
    month: usize,
    year: usize,
    tx_method: &Vec<String>,
) -> HashMap<String, f32> {
    // We can get the id_num of the month which is saved in the database based on the
    // month and year index there is passed. 
    let mut target_id_num = month as i32 + (year as i32 * 12);

    let mut final_value = HashMap::new();
    let mut to_return: Vec<f32>;

    if target_id_num == 0 {
        for i in tx_method {
            final_value.insert(i.to_string(), 0.0);
        }
        return final_value;
    }

    loop {
        let mut query = format!("SELECT {:?} FROM balance_all WHERE id_num = ?", tx_method);
        query = query.replace("[", "");
        query = query.replace("]", "");
        let final_balance = conn.query_row(&query, [target_id_num], |row| {
            let mut final_data: Vec<f32> = Vec::new();

            // We don't know the amount of tx method so we need to loop
            for i in 0..tx_method.len() {
                let to_push: String = row.get(i).unwrap();
                let final_value = to_push.parse::<f32>().unwrap();
                final_data.push(final_value);
            }
            Ok(final_data)
        });
        target_id_num -= 1;
        to_return = final_balance.unwrap();

        // We will keep the loop ongoing until we hit a non-zero balance for all tx method or
        // the id number goes to zero. Why? Example: current working month is 6th month. So we did the last 
        // transaction on January and only consider the balance of the 5th month, that is a false balance
        // and is not the balance we are supposed to doing the calculations on.
        if to_return != vec![0.0, 0.0, 0.0, 0.0] || target_id_num == 0 {
            break;
        }
    }
    for i in 0..to_return.len() {
        final_value.insert(tx_method[i].to_string(), to_return[i]);
    }
    final_value
}

/// The functions sends all the changes that happened after transactions on the month and year provided
pub fn get_all_changes(conn: &Connection, month: usize, year: usize) -> Vec<Vec<String>> {
    // returns all balance changes recorded within a given date

    let mut final_result = Vec::new();
    let tx_methods = get_all_tx_methods(conn);

    let (datetime_1, datetime_2) = get_sql_dates(month + 1, year);

    let mut statement = conn
        .prepare("SELECT * FROM changes_all Where date BETWEEN date(?) AND date(?) ORDER BY id_num")
        .expect("could not prepare statement");

    let rows = statement
        .query_map([datetime_1, datetime_2], |row| {
            let mut balance_vec: Vec<String> = Vec::new();
            // Why start at 2? Because the first two rows are date and id_num
            for i in 2..tx_methods.len() + 2 {
                balance_vec.push(row.get(i).unwrap());
            }
            Ok(balance_vec)
        })
        .expect("Error");

    for i in rows {
        final_result.push(i.unwrap());
    }

    final_result
}

/// This is a multi-use function used to retrieving all Transaction within a given date and the id_num related to them.
/// Once the transactions are fetched, we immediately start calculating the current balance values after each transaction happened
/// and finally return all of them in a tuple
pub fn get_all_txs(
    conn: &Connection,
    month: usize,
    year: usize,
) -> (Vec<Vec<String>>, Vec<Vec<String>>, Vec<String>) {
    // returns all transactions recorded within a given date + balance changes + the relevant id_num

    let all_tx_methods = get_all_tx_methods(conn);

    let mut final_all_txs: Vec<Vec<String>> = Vec::new();
    let mut final_all_balances: Vec<Vec<String>> = Vec::new();
    let mut all_id_num = Vec::new();

    // we will go through the last month balances and add/subtract
    // current month's transactions to the related tx method. After each tx calculation, add whatever
    // balance for each tx method inside a vec for final return

    let mut last_month_balance = get_last_month_balance(conn, month, year, &all_tx_methods);

    let (datetime_1, datetime_2) = get_sql_dates(month + 1, year);
    let mut statement = conn
        .prepare("SELECT * FROM tx_all Where date BETWEEN date(?) AND date(?) ORDER BY id_num")
        .expect("could not prepare statement");
    let rows = statement
        .query_map([&datetime_1, &datetime_2], |row| {
            let date: String = row.get(0).unwrap();
            let id_num: i32 = row.get(5).unwrap();
            let splitted_date = date.split('-');
            let collected_date: Vec<&str> = splitted_date.collect();
            let new_date = format!(
                "{}-{}-{}",
                collected_date[2], collected_date[1], collected_date[0]
            );

            Ok(vec![
                new_date,
                row.get(1).unwrap(),
                row.get(2).unwrap(),
                row.get(3).unwrap(),
                row.get(4).unwrap(),
                id_num.to_string(),
            ])
        })
        .expect("Error");

    for i in rows {
        let mut data = i.unwrap();
        let id_num = &data.pop().unwrap();
        all_id_num.push(id_num.to_string());
        final_all_txs.push(data);
    }

    for i in &final_all_txs {
        // this is where the calculation for the balance happens. We will loop through each tx,
        // look at the tx type, tx method and add/subtract the amount on last month balance which was fetched earlier
        // while adding the balance data after each calculation is done inside a vector.

        let tx_type = &i[4];
        let amount = &i[3].to_string().parse::<f32>().unwrap();
        let tx_method = &i[2];
        let mut new_balance: f32 = 0.0;

        if tx_type == "Expense" {
            new_balance = last_month_balance[tx_method] - amount;
        } else if tx_type == "Income" {
            new_balance = last_month_balance[tx_method] + amount;
        }

        // make changes to the balance map based on the tx
        *last_month_balance.get_mut(tx_method).unwrap() = new_balance;

        let mut to_push = vec![];
        for i in &all_tx_methods {
            to_push.push(format!("{:.2}", last_month_balance[i]))
        }

        final_all_balances.push(to_push);
    }
    (final_all_txs, final_all_balances, all_id_num)
}

/// Returns the a vector with data required to create the Changes row for zero changes in the home page.
pub fn get_empty_changes() -> Vec<String> {
    // function for quick vec with 0 changes for adding in widget

    vec![
        "Changes".to_string(),
        format!("{:.2}", 0.0),
        format!("{:.2}", 0.0),
        format!("{:.2}", 0.0),
        format!("{:.2}", 0.0),
    ]
}

/// Returns the absolute final balance which is the balance saved after each transaction was counted.
pub fn get_last_balances(conn: &Connection, tx_method: &Vec<String>) -> Vec<String> {

    let mut query = format!(
        "SELECT {:?} FROM balance_all ORDER BY id_num DESC LIMIT 1",
        tx_method
    );
    query = query.replace("[", "");
    query = query.replace("]", "");
    let final_balance = conn.query_row(&query, [], |row| {
        let mut final_data: Vec<String> = Vec::new();
        for i in 0..tx_method.len() {
            final_data.push(row.get(i).unwrap());
        }
        Ok(final_data)
    });
    final_balance.unwrap()
}

/// Returns the last id_num recorded by tx_all table
fn get_last_tx_id(conn: &Connection) -> sqlResult<i32> {
    

    let last_id: sqlResult<i32> = conn.query_row(
        "SELECT id_num FROM tx_all ORDER BY id_num DESC LIMIT 1",
        [],
        |row| row.get(0),
    );
    last_id
}

/// Returns the last id_num recorded by balance_all table
fn get_last_balance_id(conn: &Connection) -> sqlResult<i32> {

    let last_id: sqlResult<i32> = conn.query_row(
        "SELECT id_num FROM balance_all ORDER BY id_num DESC LIMIT 1",
        [],
        |row| row.get(0),
    );
    last_id
}

/// Adds a transaction to the database with the given info. The flow of this goes like this:
/// - Add the new transaction to the database
/// - Calculate the changes that happened to the Tx Method
/// - Calculate the absolute final balance
/// - Find the Changes that happened due to the transaction
/// - Push them to the database
pub fn add_new_tx(
    conn: &Connection,
    date: &str,
    details: &str,
    tx_method: &str,
    amount: &str,
    tx_type: &str,
) -> sqlResult<()> {
    conn.execute(
        "INSERT INTO tx_all (date, details, tx_method, amount, tx_type) VALUES (?, ?, ?, ?, ?)",
        [date, details, tx_method, amount, tx_type],
    )?;

    let split = date.split("-");
    let vec = split.collect::<Vec<&str>>();
    let mut mnth = vec[1].to_string();
    if &mnth[0..0] == "0" {
        mnth = mnth.replace("0", "");
    }
    let month = mnth.parse::<i32>().unwrap();
    let year = vec[0][2..].parse::<i32>().unwrap() - 22;

    let target_id_num = month as i32 + (year as i32 * 12);

    // This is necessary for the foreign key field in the changes_all table
    // and must align with the latest transaction id_num
    let last_id = get_last_tx_id(conn)?;
    let last_balance_id = get_last_balance_id(conn)?;

    // we have to get these following data to push to the database
    // new_balance_data : the current month balance after the transaction
    // new_changes_data : the new changes data to push to the database
    // last_balance_data : the absolute final balance after all transaction
    let mut new_balance_data = Vec::new();
    let mut new_changes_data = Vec::new();
    let mut last_balance_data = Vec::new();

    let all_tx_methods = get_all_tx_methods(conn);
    let last_balance = get_last_balances(conn, &all_tx_methods);
    let mut cu_month_balance =
        get_last_month_balance(conn, month as usize, year as usize, &all_tx_methods);

    let mut new_balance = 0.0;
    let int_amount = amount.parse::<f32>().unwrap();
    let lower_tx_type = tx_type.to_lowercase();

    // makes changes to the current month balance and push them to vector
    if tx_type == "Expense" {
        new_balance = cu_month_balance[tx_method] - int_amount;
    } else if tx_type == "Income" {
        new_balance = cu_month_balance[tx_method] + int_amount;
    }

    *cu_month_balance.get_mut(tx_method).unwrap() = new_balance;

    for i in &all_tx_methods {
        new_balance_data.push(format!("{:.2}", cu_month_balance[i]))
    }

    for i in 0..all_tx_methods.len() {
        // the variable to keep track whether any changes were made to the tx method
        let cu_last_balance = last_balance[i].parse::<f32>().unwrap();
        let mut default_change = format!("{:.2}", 0.0);

        // we could have just used the tx_method from the argument but adding the default values
        // manually after that would make it tricky because have to maintain the tx method balance order
        // and the Changes order
        
        if &all_tx_methods[i] == &tx_method {
            if lower_tx_type == "expense" {
                default_change = format!("↓{}", &amount);
                let edited_balance = cu_last_balance - int_amount;
                last_balance_data.push(format!("{edited_balance:.2}"));

            } else if lower_tx_type == "income" {
                default_change = format!("↑{}", &amount);
                let edited_balance = cu_last_balance + int_amount;
                last_balance_data.push(format!("{edited_balance:.2}"));
            }
        }
        new_changes_data.push(default_change);
    }

    // the query kept on breaking for a single comma so had to follow this ugly way to do this.
    let mut balance_query = format!("UPDATE balance_all SET ");
    for i in 0..new_balance_data.len() {
        if i != new_balance_data.len() - 1 {
            balance_query.push_str(&format!(
                "{} = {}, ",
                all_tx_methods[i], new_balance_data[i]
            ))
        } else {
            balance_query.push_str(&format!("{} = {} ", all_tx_methods[i], new_balance_data[i]))
        }
    }
    balance_query.push_str(&format!("WHERE id_num = {target_id_num}"));

    // there is only 1 value in the last_balance_data, we already know on which tx method the changes happened
    let last_balance_query = format!("UPDATE balance_all SET {tx_method} = {} WHERE id_num = {}", last_balance_data[0], last_balance_id);
    
    let mut changes_query = format!("INSERT INTO changes_all (id_num, date, {all_tx_methods:?}) VALUES ({last_id}, ?, {new_changes_data:?})");
    changes_query = changes_query.replace("[", "");
    changes_query = changes_query.replace("]", "");
    conn.execute(&balance_query, [])?;
    conn.execute(&last_balance_query, [])?;
    conn.execute(&changes_query, [date])?;

    Ok(())
}

/// Updates the absolute final balance and deletes the selected transaction.
/// Foreign key cascade takes care of the Changes data in the database.
pub fn delete_tx(conn: &Connection, id_num: usize) -> sqlResult<()> {

    let tx_methods = get_all_tx_methods(conn);
    let last_balance = get_last_balances(conn, &tx_methods);

    let mut final_last_balance = Vec::new();

    let query = format!("SELECT * FROM tx_all Where id_num = {}", id_num);
    let data = conn
        .query_row(&query, [], |row| {
            let mut final_data: Vec<String> = Vec::new();
            final_data.push(row.get(2).unwrap());
            final_data.push(row.get(3).unwrap());
            final_data.push(row.get(4).unwrap());
            Ok(final_data)
        })
        .unwrap();

    let source = &data[0];
    let amount = &data[1].parse::<f32>().unwrap();
    let tx_type: &str = &data[2];

    for i in 0..tx_methods.len() {
        let mut cu_balance = last_balance[i].parse::<f32>().unwrap();
        if &tx_methods[i] == source {
            match tx_type {
                "Expense" => cu_balance += amount,
                "Income" => cu_balance -= amount,
                _ => {}
            }
        }
        final_last_balance.push(cu_balance.to_string());
    }

    let mut last_balance_query = format!("UPDATE balance_all SET ");
    for i in 0..final_last_balance.len() {
        if i != final_last_balance.len() - 1 {
            last_balance_query.push_str(&format!("{} = {}, ", tx_methods[i], final_last_balance[i]))
        } else {
            last_balance_query.push_str(&format!("{} = {} ", tx_methods[i], final_last_balance[i]))
        }
    }
    last_balance_query.push_str(&format!("WHERE id_num = 49"));

    let del_query = format!("DELETE FROM tx_all WHERE id_num = {id_num}");

    conn.execute(&last_balance_query, [])?;
    conn.execute(&del_query, [])?;
    Ok(())
}
