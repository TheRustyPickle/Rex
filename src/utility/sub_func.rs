use crate::outputs::TerminalExecutionError;
use crate::utility::{get_all_tx_methods, get_sql_dates};
use crossterm::execute;
use crossterm::terminal::{Clear, ClearType};
use rusqlite::Connection;
use std::collections::{HashMap, HashSet};
use std::io;
use std::process::Command;

/// Returns the balance of all methods based on year and month point.
/// if the balance is empty/0 at the given point for any one of the methods
/// it will try to find the balance for that method from one of the earlier points.
pub fn get_last_time_balance(
    month: usize,
    year: usize,
    tx_method: &Vec<String>,
    conn: &Connection,
) -> HashMap<String, f64> {
    // We can get the id_num of the month which is saved in the database based on the
    // month and year index there is passed.
    let target_id_num = month as i32 + (year as i32 * 12);

    let mut final_value = HashMap::new();
    for i in tx_method {
        final_value.insert(i.to_string(), 0.0);
    }

    // balance_all starts at point 1. 1 means month 1, year 2022/0.
    // There is no earlier balance than this
    if target_id_num == 0 {
        return final_value;
    }

    let mut checked_methods: Vec<&str> = Vec::new();

    let tx_method_string = tx_method
        .iter()
        .map(|m| format!("\"{}\"", m))
        .collect::<Vec<_>>()
        .join(", ");

    // the process goes like this
    // m1  m2  m3  id
    //  0   0  10   1
    // 10  10   0   2
    // 10   0   0   3
    // fetch all three rows, start checking from id 3
    // m1 is already found, save that, go to the previous row, save m2,
    // go to the previous row, save m3 -> break -> return the data

    let query = format!(
        "SELECT {} FROM balance_all WHERE id_num <= ? ORDER BY id_num DESC",
        tx_method_string
    );

    let mut stmt = conn.prepare(&query).unwrap();
    let mut rows = stmt.query([target_id_num]).unwrap();

    while let Some(row) = rows.next().unwrap() {
        // all methods checked = no longer necessary to check any more rows
        if checked_methods.len() == tx_method.len() {
            break;
        }
        // check each tx_method column in the current row
        for i in 0..tx_method.len() {
            if !checked_methods.contains(&tx_method[i].as_str()) {
                let balance: f64 = row.get(i).unwrap();

                // we only need non-zero balance
                if balance != 0.0 {
                    *final_value.get_mut(&tx_method[i]).unwrap() = balance;
                    checked_methods.push(&tx_method[i]);
                }
            }
        }
    }

    final_value
}

/// The functions sends all the changes that happened after transactions on the month and year provided
pub fn get_all_changes(month: usize, year: usize, conn: &Connection) -> Vec<Vec<String>> {
    let mut final_result = Vec::new();
    let tx_methods = get_all_tx_methods(conn);

    let (datetime_1, datetime_2) = get_sql_dates(month, year);

    let mut statement = conn
        .prepare("SELECT * FROM changes_all Where date BETWEEN date(?) AND date(?) ORDER BY date, id_num")
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

/// Used to retrieving all Transaction within a given date, balance and the id_num related to them.
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
    // balance for each tx method inside a vec to finally return them

    let mut last_month_balance = get_last_time_balance(month, year, &all_tx_methods, conn);

    let (datetime_1, datetime_2) = get_sql_dates(month, year);

    // preparing the query for db, getting current month's all transactions
    let mut statement = conn
        .prepare(
            "SELECT * FROM tx_all Where date BETWEEN date(?) AND date(?) ORDER BY date, id_num",
        )
        .expect("could not prepare statement");

    let rows = statement
        .query_map([&datetime_1, &datetime_2], |row| {
            // collect the row data and put them in a vec
            let date: String = row.get(0).unwrap();
            let id_num: i32 = row.get(5).unwrap();
            let collected_date = date.split('-').collect::<Vec<&str>>();
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
                row.get(6).unwrap(),
                id_num.to_string(),
            ])
        })
        .expect("Error");

    for i in rows.flatten() {
        // data contains all tx data of a transaction
        let mut data = i;
        let id_num = &data.pop().unwrap();
        all_id_num.push(id_num.to_string());
        final_all_txs.push(data);
    }

    for i in &final_all_txs {
        // this is where the calculation for the balance happens. We will loop through each tx,
        // look at the tx type, tx method and add/subtract the amount on last month balance which was fetched earlier

        // collect data inside variables
        let tx_type = &i[4];
        let amount = &i[3].to_string().parse::<f64>().unwrap();
        let tx_method = &i[2];

        // If the transaction is not a transfer, default balance goes to new_balance_from
        // and new_balance_to remains empty. On transfer TX both of them are used

        let mut new_balance_from: f64 = 0.0;
        let mut new_balance_to: f64 = 0.0;

        let mut from_method = "".to_string();
        let mut to_method = "".to_string();

        // add or subtract the amount based on the tx type
        if tx_type == "Expense" {
            new_balance_from = last_month_balance[tx_method] - amount;
        } else if tx_type == "Income" {
            new_balance_from = last_month_balance[tx_method] + amount;
        } else if tx_type == "Transfer" {
            let splitted = tx_method.split(" to ").collect::<Vec<&str>>();
            from_method = splitted[0].to_string();
            to_method = splitted[1].to_string();
            new_balance_from = last_month_balance[&from_method] - amount;
            new_balance_to = last_month_balance[&to_method] + amount;
        }

        // make changes to the balance map based on the tx
        // for transfer TX first block executes
        // * new_balance_to != 0 means it's a transfer transaction
        if new_balance_to != 0.0 {
            *last_month_balance.get_mut(&from_method).unwrap() = new_balance_from;
            *last_month_balance.get_mut(&to_method).unwrap() = new_balance_to;
        } else {
            *last_month_balance.get_mut(tx_method).unwrap() = new_balance_from;
        }

        // push all the changes gathered to the return variable
        let mut to_push = vec![];
        for i in &all_tx_methods {
            to_push.push(format!("{:.2}", last_month_balance[i]))
        }

        final_all_balances.push(to_push);
    }

    // pushes the final balance that was calculated just now to the db on the balance_all table
    if !final_all_balances.is_empty() {
        let target_id_num = month as i32 + 1 + (year as i32 * 12);
        let final_index = final_all_balances.len() - 1;

        let balance_query = format!(
            "UPDATE balance_all SET {} WHERE id_num = {}",
            final_all_balances[final_index]
                .iter()
                .enumerate()
                .map(|(i, balance)| format!(r#""{}" = "{}""#, all_tx_methods[i], balance))
                .collect::<Vec<String>>()
                .join(", "),
            target_id_num
        );
        conn.execute(&balance_query, [])
            .expect("Error updating balance query");
    }

    (final_all_txs, final_all_balances, all_id_num)
}

/// Returns the absolute final balance or the last row on balance_all table.
pub fn get_last_balances(tx_method: &Vec<String>, conn: &Connection) -> Vec<String> {
    let mut query = format!(
        "SELECT {:?} FROM balance_all ORDER BY id_num DESC LIMIT 1",
        tx_method
    );
    query = query.replace('[', "");
    query = query.replace(']', "");

    let final_balance = conn.query_row(&query, [], |row| {
        let mut final_data: Vec<String> = Vec::new();
        for i in 0..tx_method.len() {
            let row_data: f64 = row.get(i).unwrap();
            final_data.push(row_data.to_string());
        }
        Ok(final_data)
    });
    final_balance.unwrap()
}

/// This function asks user to input one or more Transaction Method names.
/// Once the collection is done sends to the database for adding the columns.
/// This functions is both used when creating the initial db and when updating
/// the database with new transaction methods.
pub fn get_user_tx_methods(add_new_method: bool, conn: &Connection) -> Option<Vec<String>> {
    let restricted_methods = ["Total", "Balance", "Changes", "Income", "Expense"];
    let mut stdout = io::stdout();

    // this command clears up the terminal. This is added so the terminal doesn't get
    //filled up with previous unnecessary texts.
    execute!(stdout, Clear(ClearType::FromCursorUp)).unwrap();

    let mut current_tx_methods: Vec<String> = Vec::new();
    let mut db_tx_methods = vec![];

    let mut method_line = "Currently added Transaction Methods: ".to_string();

    // if we are adding more tx methods to an existing database, we need to
    // to get the existing columns to prevent duplicates/error.
    if add_new_method {
        current_tx_methods = get_all_tx_methods(conn);
        for i in &current_tx_methods {
            method_line.push_str(&format!("\n- {i}"))
        }
    }

    // we will take input from the user and use the input data to create a new database
    // keep on looping until the methods are approved by sending y.
    loop {
        let mut line = String::new();
        let mut verify_line = String::new();
        let mut verify_input = "Inserted Transaction Methods:\n".to_string();
        if add_new_method {
            println!("{method_line}\n");
            println!("\nUser input required for Transaction Methods. Must be separated by one comma \
or ','. Example: Bank, Cash, PayPal.\n\nInput 'Cancel' to cancel the operation\n\nEnter Transaction Methods:");
        } else {
            println!("Database not found. Follow the guide below to start the app.");
            println!(
                "\nUser input required for Transaction Methods. Must be separated by one comma \
or ','. Example: Bank, Cash, PayPal.\n\nEnter Transaction Methods:"
            );
        }

        // take user input for transaction methods
        std::io::stdin().read_line(&mut line).unwrap();

        // extremely important to prevent crashes on Windows
        line = line.trim().to_string();

        if line.to_lowercase().starts_with("cancel") && add_new_method {
            return None;
        }

        // split them and remove duplicates
        let mut splitted = line.split(',').map(|s| s.trim()).collect::<Vec<&str>>();
        splitted.retain(|s| !s.is_empty() && !restricted_methods.contains(s));

        let mut seen = HashSet::new();
        let mut unique = Vec::new();

        // remove duplicates from the splitted vec
        for item in splitted {
            if seen.insert(item) {
                unique.push(item);
            }
        }
        splitted = unique;

        let mut filtered_splitted = vec![];

        // If adding new transactions methods, remove the existing methods
        // from in the inputted data
        if add_new_method {
            for i in &splitted {
                let user_tx_method = i.to_string();
                if !current_tx_methods.contains(&user_tx_method) {
                    filtered_splitted.push(user_tx_method)
                }
            }
        }
        // Check if the input is not empty. If yes, start from the beginning
        if (splitted.is_empty() || add_new_method)
            && (filtered_splitted.is_empty() || !add_new_method)
        {
            execute!(stdout, Clear(ClearType::FromCursorUp)).unwrap();
            println!("\nTransaction Method input cannot be empty and existing Transaction Methods cannot be used twice");
        } else {
            if add_new_method {
                for i in &filtered_splitted {
                    verify_input.push_str(&format!("- {i}\n"));
                }
            } else {
                for i in &splitted {
                    verify_input.push_str(&format!("- {i}\n"));
                }
            }
            verify_input.push_str("Accept the values? y/n");
            println!("\n{verify_input}");

            std::io::stdin().read_line(&mut verify_line).unwrap();

            verify_line = verify_line.trim().to_string();

            // until the answer is y/yes/cancel continue the loop
            if verify_line.to_lowercase().starts_with('y') {
                if add_new_method {
                    for i in filtered_splitted {
                        db_tx_methods.push(i);
                    }
                } else {
                    for i in splitted {
                        let value = i.to_string();
                        db_tx_methods.push(value);
                    }
                }
                break;
            } else {
                execute!(stdout, Clear(ClearType::FromCursorUp)).unwrap();
            }
        }
    }
    Some(db_tx_methods)
}

/// Tries to open terminal/cmd and run this app
pub fn start_terminal(original_dir: &str) -> Result<(), TerminalExecutionError> {
    if cfg!(target_os = "windows") {
        Command::new("cmd.exe")
            .arg("start")
            .arg("rex")
            .output()
            .map_err(TerminalExecutionError::ExecutionFailed)?;
    } else {
        let mut all_terminals = HashMap::new();
        let gnome_dir = format!("--working-directory={}", original_dir);

        // TODO add more terminal support
        all_terminals.insert(
            "konsole",
            vec![
                "--new-tab".to_string(),
                "--workdir".to_string(),
                original_dir.to_string(),
                "-e".to_string(),
                "./rex".to_string(),
            ],
        );

        all_terminals.insert(
            "gnome-terminal",
            vec![
                gnome_dir,
                "--maximize".to_string(),
                "--".to_string(),
                "./rex".to_string(),
            ],
        );

        let mut terminal_opened = false;
        let mut result = None;

        for (key, value) in all_terminals {
            let status = Command::new(key).args(value).output();
            match status {
                Ok(out) => {
                    if out.stderr.len() > 2 {
                        result = Some(TerminalExecutionError::NotFound(out))
                    } else {
                        terminal_opened = true;
                        break;
                    }
                }
                Err(err) => result = Some(TerminalExecutionError::ExecutionFailed(err)),
            }
        }
        if !terminal_opened {
            return Err(result.unwrap());
        }
    };
    Ok(())
}
