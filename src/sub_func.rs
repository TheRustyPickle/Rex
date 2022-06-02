use rusqlite::{Connection, Result as sqlResult};

pub fn get_all_tx_methods(conn: &Connection) -> Vec<String> {
    // returns all transaction methods added to the database
    // example bank, cash.
    let column_names = conn.prepare("SELECT * FROM balance_all").expect("could not prepare statement");
    let mut tx_methods = vec![];
    for i in 2..99 {
        let column = column_names.column_name(i);
        match column {
            Ok(c) => tx_methods.push(c.to_string()),
            Err(_) => break,
        }
    }
    tx_methods
}

fn get_sql_dates(month: usize, year: usize) -> (String, String) {
    // returns dates from month and year to a format that is suitable for
    // database WHERE statement.

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
    (datetime_1, datetime_2)
}

pub fn get_all_balance(conn: &Connection, month: usize, year: usize) -> Vec<Vec<String>> {
    // retunrs all balance recorded within a given date

    let mut final_result = Vec::new();
    let tx_methods = get_all_tx_methods(conn);
    

    let (datetime_1, datetime_2) = get_sql_dates(month, year);
    let mut statement = conn.prepare("SELECT * FROM balance_all Where date BETWEEN date(?) AND date(?) ORDER BY id_num").expect("could not prepare statement");

    let rows = statement.query_map([datetime_1, datetime_2], |row| {
        let mut balance_vec: Vec<String> = Vec::new();
        for i in 2..tx_methods.len()+2 {
            balance_vec.push(row.get(i).unwrap());
        }
        Ok(balance_vec)
    }).expect("Error");
    
    for i in rows {
        final_result.push(i.unwrap());
    }
    final_result
}

pub fn get_all_changes(conn: &Connection, month: usize, year: usize) -> Vec<Vec<String>> {
    // retunrs all balance changes recorded within a given date

    let mut final_result = Vec::new();
    let tx_methods = get_all_tx_methods(conn);

    let (datetime_1, datetime_2) = get_sql_dates(month, year);

    let mut statement = conn.prepare("SELECT * FROM changes_all Where date BETWEEN date(?) AND date(?) ORDER BY id_num").expect("could not prepare statement");

    let rows = statement.query_map([datetime_1, datetime_2], |row| {
        let mut balance_vec: Vec<String> = Vec::new();
        for i in 2..tx_methods.len()+2 {
            balance_vec.push(row.get(i).unwrap());
        }
        Ok(balance_vec)
    }).expect("Error");

    for i in rows {
        final_result.push(i.unwrap());
    }

    final_result
}

pub fn get_all_txs(conn: &Connection, month: usize, year: usize) -> Vec<Vec<String>> {
    // retunrs all transactions recorded within a given date

    let mut final_result: Vec<Vec<String>> = Vec::new();

    let (datetime_1, datetime_2) = get_sql_dates(month, year);

    let mut statement = conn.prepare("SELECT * FROM tx_all Where date BETWEEN date(?) AND date(?) ORDER BY id_num").expect("could not prepare statement");
    
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

pub fn get_empty_changes() -> Vec<String> {
    // function for quick vec with 0 changes for adding in widget

    vec![
        "Changes".to_string(), "0".to_string(), "0".to_string(), "0".to_string(), "0".to_string(),
        ]
}

pub fn get_last_balances(conn: &Connection, tx_method: &Vec<String>) -> Vec<String> {
    // returns the last recorded balance of all tx methods 

    let mut query = format!("SELECT {:?} FROM balance_all ORDER BY id_num DESC LIMIT 1", tx_method);
    query = query.replace("[", "");
    query = query.replace("]", "");
    let final_balance = conn.query_row(
        &query,
        [],
        |row| {
            let mut final_data: Vec<String> = Vec::new();
            for i in 0..tx_method.len() {
                final_data.push(row.get(i).unwrap());
            }
            Ok(final_data)
        },
    );
    final_balance.unwrap()
}

pub fn get_last_tx_id(conn: &Connection) -> sqlResult<i32> {
    // returns the last id_num recorded by tx_all table

    let last_id: sqlResult<i32> = conn.query_row(
        "SELECT id_num FROM tx_all ORDER BY id_num DESC LIMIT 1",
        [],
        |row| row.get(0),
    );
    last_id
}

pub fn add_new_tx(conn: &Connection, date: &str, details: &str, tx_method: &str, amount: &str, tx_type: &str) -> sqlResult<()> {
    // used for adding transaction on the database

    conn.execute("INSERT INTO tx_all (date, details, tx_method, amount, tx_type) VALUES (?, ?, ?, ?, ?)",
        [date, details, tx_method, amount, tx_type])?;

    let mut new_balance = Vec::new();
    let mut new_changes = Vec::new();

    let last_id = get_last_tx_id(conn)?;
    let all_tx_methods = get_all_tx_methods(conn);
    let last_balance = get_last_balances(conn, &all_tx_methods);
    
    let int_amount: i32 = amount.parse().unwrap();
    let lower_tx_type = tx_type.to_lowercase();

    for i in 0..last_balance.len() {
        let mut int_balance: i32 = last_balance[i].parse().unwrap();
        let mut default_change = "0".to_string();
        
        if &all_tx_methods[i] == &tx_method {
            if lower_tx_type == "expense" || lower_tx_type == "e" {
                int_balance -= int_amount;
                default_change = format!("↓{}", &amount);
            }
            else if lower_tx_type == "income" || lower_tx_type == "i" {
                int_balance += int_amount;
                default_change = format!("↑{}", &amount);
            }
        }
        new_balance.push(int_balance.to_string());
        new_changes.push(default_change);
    }


    let mut balance_query = format!("INSERT INTO balance_all (id_num, date, {all_tx_methods:?}) VALUES ({last_id}, ?, {new_balance:?})");
    balance_query = balance_query.replace("[", "");
    balance_query = balance_query.replace("]", "");

    let mut changes_query = format!("INSERT INTO changes_all (id_num, date, {all_tx_methods:?}) VALUES ({last_id}, ?, {new_changes:?})");
    changes_query = changes_query.replace("[", "");
    changes_query = changes_query.replace("]", "");

    conn.execute(&balance_query, [date])?;
    conn.execute(&changes_query, [date])?;

    Ok(())
}