use rusqlite::Connection;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::stdout;
use std::path::PathBuf;
use std::process::Command;

use crate::activity_page::{ActivityDetails, ActivityTx};
use crate::outputs::{ComparisonType, TerminalExecutionError};
use crate::page_handler::{ActivityType, DateType, ResetType, UserInputType};
use crate::utility::{
    add_new_activity, add_new_activity_tx, check_comparison, check_restricted, clear_terminal,
    flush_output, get_all_tx_methods, get_sql_dates, reverse_date_format, take_input,
};

/// Returns the balance of all methods based on year and month point.
/// If the balance is empty/0 at the given point for any one of the methods
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
        .map(|m| format!("\"{m}\""))
        .collect::<Vec<_>>()
        .join(", ");

    // the process goes like this:
    //
    // m1  m2  m3  id
    //  0   0  10   1
    // 10  10   0   2
    // 10   0   0   3
    // fetch all three rows, start checking from id 3
    // m1 is already found, save that, go to the previous row, save m2,
    // go to the previous row, save m3 -> break -> return the data

    let query = format!(
        "SELECT {tx_method_string} FROM balance_all WHERE id_num <= ? ORDER BY id_num DESC",
    );

    let mut stmt = conn.prepare(&query).unwrap();
    let mut rows = stmt.query([target_id_num]).unwrap();

    while let Some(row) = rows.next().unwrap() {
        // All methods checked = no longer necessary to check any more rows
        if checked_methods.len() == tx_method.len() {
            break;
        }
        // Check each tx_method column in the current row
        for (i, item) in tx_method.iter().enumerate() {
            if !checked_methods.contains(&item.as_str()) {
                let balance: f64 = row.get(i).unwrap();

                // We only need non-zero balance
                if balance != 0.0 {
                    *final_value.get_mut(item).unwrap() = balance;
                    checked_methods.push(item);
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

    let (datetime_1, datetime_2) = get_sql_dates(month, year, &DateType::Monthly);

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
        .unwrap();

    for i in rows {
        final_result.push(i.unwrap());
    }
    final_result
}

/// Used to retrieving all Transaction within a given date, balance and the `id_num` related to them.
pub fn get_all_txs(
    conn: &Connection,
    month: usize,
    year: usize,
) -> (Vec<Vec<String>>, Vec<Vec<String>>, Vec<String>) {
    // Returns all transactions recorded within a given date + balance changes + the relevant id_num

    let all_tx_methods = get_all_tx_methods(conn);

    let mut final_all_txs: Vec<Vec<String>> = Vec::new();
    let mut final_all_balances: Vec<Vec<String>> = Vec::new();
    let mut all_id_num = Vec::new();

    // We will go through the last month balances and add/subtract
    // current month's transactions to the related tx method. After each tx calculation, add whatever
    // balance for each tx method inside a vec to finally return them

    let mut last_month_balance = get_last_time_balance(month, year, &all_tx_methods, conn);

    let (datetime_1, datetime_2) = get_sql_dates(month, year, &DateType::Monthly);

    // preparing the query for db, getting current month's all transactions
    let mut statement = conn
        .prepare(
            "SELECT * FROM tx_all WHERE date BETWEEN date(?) AND date(?) ORDER BY date, id_num",
        )
        .expect("could not prepare statement");

    let rows = statement
        .query_map([&datetime_1, &datetime_2], |row| {
            // Collect the row data and put them in a vec
            let date = reverse_date_format(row.get(0).unwrap());
            let id_num: i32 = row.get(5).unwrap();

            Ok(vec![
                date,
                row.get(1).unwrap(),
                row.get(2).unwrap(),
                row.get(3).unwrap(),
                row.get(4).unwrap(),
                row.get(6).unwrap(),
                id_num.to_string(),
            ])
        })
        .unwrap();

    for i in rows.flatten() {
        // Data contains all tx data of a transaction
        let mut data = i;
        let id_num = &data.pop().unwrap();
        all_id_num.push(id_num.to_string());
        final_all_txs.push(data);
    }

    for i in &final_all_txs {
        // This is where the calculation for the balance happens. We will loop through each tx,
        // look at the tx type, tx method and add/subtract the amount on last month balance which was fetched earlier

        // collect data inside variables
        let tx_type = &i[4];
        let amount = &i[3].to_string().parse::<f64>().unwrap();
        let tx_method = &i[2];

        // If the transaction is not a transfer, default balance goes to new_balance_from
        // and new_balance_to remains empty. On transfer TX both of them are used

        let mut new_balance_from: f64 = 0.0;
        let mut new_balance_to: f64 = 0.0;

        let mut from_method = String::new();
        let mut to_method = String::new();

        // Add or subtract the amount based on the tx type
        if tx_type == "Expense" {
            new_balance_from = last_month_balance[tx_method] - amount;
        } else if tx_type == "Income" {
            new_balance_from = last_month_balance[tx_method] + amount;
        } else if tx_type == "Transfer" {
            let split_method = tx_method.split(" to ").collect::<Vec<&str>>();
            from_method = split_method[0].to_string();
            to_method = split_method[1].to_string();
            new_balance_from = last_month_balance[&from_method] - amount;
            new_balance_to = last_month_balance[&to_method] + amount;
        }

        // Make changes to the balance map based on the tx
        // for transfer TX first block executes
        // new_balance_to != 0 means it's a transfer transaction
        if new_balance_to == 0.0 {
            *last_month_balance.get_mut(tx_method).unwrap() = new_balance_from;
        } else {
            *last_month_balance.get_mut(&from_method).unwrap() = new_balance_from;
            *last_month_balance.get_mut(&to_method).unwrap() = new_balance_to;
        }

        // Push all the changes gathered to the return variable
        let mut to_push = vec![];
        for i in &all_tx_methods {
            to_push.push(format!("{:.2}", last_month_balance[i]));
        }

        final_all_balances.push(to_push);
    }

    // Pushes the final balance that was calculated just now to the db on the balance_all table
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

/// Returns the absolute final balance or the last row on `balance_all` table.
pub fn get_last_balances(conn: &Connection) -> Vec<String> {
    let tx_method = get_all_tx_methods(conn);
    let mut query = format!("SELECT {tx_method:?} FROM balance_all ORDER BY id_num DESC LIMIT 1");
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

/// Prompts the user to select and option and start taking relevant inputs
#[cfg(not(tarpaulin_include))]
pub fn start_taking_input(conn: &Connection) -> UserInputType {
    let mut stdout = stdout();
    clear_terminal(&mut stdout);

    loop {
        println!(
            "Enter an option number to proceed. Input 'Cancel' to cancel the operation

1. Add New Transaction Methods
2. Rename Transaction Method
3. Reposition Transactions Methods
4. Set a new location for app data
5. Set backup DB paths\n"
        );
        print!("Proceed with option number: ");
        flush_output(&stdout);

        let user_input = take_input();
        let input_type = UserInputType::from_string(&user_input.to_lowercase());

        match input_type {
            UserInputType::AddNewTxMethod(_) => return get_user_tx_methods(true, Some(conn)),
            UserInputType::RenameTxMethod(_) => return get_rename_data(conn),
            UserInputType::RepositionTxMethod(_) => return get_reposition_data(conn),
            UserInputType::SetNewLocation(_) => return get_new_location(),
            UserInputType::BackupDBPath(_) => return get_backup_db_paths(),
            UserInputType::CancelledOperation | UserInputType::ResetData(_) => return input_type,
            UserInputType::InvalidInput => clear_terminal(&mut stdout),
        }
    }
}

/// This function asks user to input one or more Transaction Method names.
/// Once the collection is done sends to the database for adding the columns.
/// This functions is both used when creating the initial db and when updating
/// the database with new transaction methods.
#[cfg(not(tarpaulin_include))]
pub fn get_user_tx_methods(add_new_method: bool, conn: Option<&Connection>) -> UserInputType {
    let mut stdout = stdout();

    // This command clears up the terminal. This is added so the terminal doesn't get
    // filled up with previous unnecessary texts.
    clear_terminal(&mut stdout);

    let mut current_tx_methods: Vec<String> = Vec::new();
    let mut db_tx_methods = vec![];

    let mut method_line = "Currently added Transaction Methods: \n".to_string();

    // If we are adding more tx methods to an existing database, we need
    // to get the existing columns to prevent duplicates/error.
    // This needs to be separated because if it's not adding new tx methods,
    // getting all tx methods will crash
    if add_new_method {
        current_tx_methods = get_all_tx_methods(conn.unwrap());
        for i in &current_tx_methods {
            method_line.push_str(&format!("\n- {i}"));
        }
    }

    // We will take input from the user and use the input data to create a new database
    // keep on looping until the methods are approved by sending y.
    'outer_loop: loop {
        let mut verify_input = "Inserted Transaction Methods:\n\n".to_string();
        if add_new_method {
            println!("{method_line}\n");
            print!("User input required for Transaction Methods. Separate methods by one comma.
Example input: Bank, Cash, PayPal.\n\nInput 'Cancel' to cancel the operation\n\nEnter Transaction Methods: ");
        } else {
            println!("Database not found. Follow the guide below to start the app.");
            print!(
                "\nUser input required for Transaction Methods. Separate methods by one comma.
Example input: Bank, Cash, PayPal.\n\nEnter Transaction Methods: "
            );
        }
        flush_output(&stdout);

        // Take user input for transaction methods
        let line = take_input();

        // cancel operation on cancel input
        if line.trim().to_lowercase().starts_with("cancel") && add_new_method {
            return UserInputType::CancelledOperation;
        }

        if line.to_lowercase().contains("to") {
            clear_terminal(&mut stdout);
            println!("'To' cannot be used in Transaction Methods.\n");
            continue;
        }

        // split them and remove duplicates
        let mut inputted_methods: Vec<&str> = line
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect::<Vec<&str>>();

        // Check if the input is not empty. If yes, start from the beginning
        if inputted_methods.is_empty() {
            clear_terminal(&mut stdout);
            println!("Transaction Method input cannot be empty.\n");
            continue;
        }

        // Restart the loop if the method is a restricted value or already exists
        for method in &inputted_methods {
            if check_restricted(method, None) {
                clear_terminal(&mut stdout);
                println!("Restricted method name. Value cannot be accepted.\n");
                continue 'outer_loop;
            }

            if !current_tx_methods.is_empty() && check_restricted(method, Some(&current_tx_methods))
            {
                clear_terminal(&mut stdout);
                println!("Transaction Methods already exists. Use a different value\n");
                continue 'outer_loop;
            }
        }

        // if input is example ["bank", "Bank"] remove the 2nd duplicate
        let mut seen = HashSet::new();
        let mut unique = Vec::new();

        for item in inputted_methods {
            if seen.insert(item.to_lowercase()) {
                unique.push(item);
            }
        }
        inputted_methods = unique;

        for i in &inputted_methods {
            verify_input.push_str(&format!("- {i}\n"));
        }

        println!("\n{verify_input}");
        print!("Accept the values? y/n: ");
        flush_output(&stdout);

        let verify_line = take_input();

        // Until the answer is y/cancel continue the loop
        if verify_line.to_lowercase().starts_with('y') {
            for i in inputted_methods {
                db_tx_methods.push(i.to_string());
            }
            break;
        }
        clear_terminal(&mut stdout);
    }
    UserInputType::AddNewTxMethod(db_tx_methods)
}

/// Gets a new tx method name from the user to replace an existing method
#[cfg(not(tarpaulin_include))]
fn get_rename_data(conn: &Connection) -> UserInputType {
    let mut stdout = stdout();

    clear_terminal(&mut stdout);
    let mut rename_data = Vec::new();

    let tx_methods = get_all_tx_methods(conn);

    loop {
        let mut method_line =
            "Select a Transaction Method to proceed. Input 'Cancel' to cancel the operation.
        
Currently added Transaction Methods: \n"
                .to_string();

        for (i, item) in tx_methods.iter().enumerate() {
            method_line.push_str(&format!("\n{}. {}", i + 1, item));
        }
        println!("{method_line}");
        print!("\nEnter the method number to edit: ");
        flush_output(&stdout);

        let user_input = take_input();

        // If no input, start from the beginning
        if user_input.is_empty() {
            clear_terminal(&mut stdout);
            continue;
        }

        // Cancel the process on cancel input
        if user_input.trim().to_lowercase().starts_with("cancel") {
            return UserInputType::CancelledOperation;
        }

        let method_number_result = user_input.parse::<usize>();

        let Ok(method_number) = method_number_result else {
            clear_terminal(&mut stdout);
            println!("Invalid method number. Example input: 1\n");
            continue;
        };

        // Start from the beginning if the number is beyond the tx method total index
        if method_number > tx_methods.len() {
            clear_terminal(&mut stdout);
            println!("Invalid method number. Example input: 1\n");
            continue;
        }

        println!(
            "\nSelected method: {}. Enter the new name for this Transaction Method.",
            tx_methods[method_number - 1]
        );
        print!("New method name: ");
        flush_output(&stdout);

        let new_method_name = take_input();

        if new_method_name.to_lowercase().contains("to") {
            clear_terminal(&mut stdout);
            println!("'To' cannot be used in Transaction Methods.\n");
            continue;
        }

        // Start from the beginning if the given tx method already exists
        if check_restricted(&new_method_name, Some(&tx_methods)) {
            clear_terminal(&mut stdout);
            println!("Transaction Methods already exists. Use a different value\n");
            continue;
        }

        // The tx method cannot be in the list of the restricted words otherwise UI may glitch
        if check_restricted(&new_method_name, None) {
            clear_terminal(&mut stdout);
            println!("Restricted method name. Value cannot be accepted.\n");
            continue;
        }

        println!(
            "\nRename {} to {new_method_name}.",
            tx_methods[method_number - 1]
        );
        print!("Accept the values? y/n: ");
        flush_output(&stdout);

        let confirm_operation = take_input();

        if confirm_operation.to_lowercase().starts_with('y') {
            rename_data.push(tx_methods[method_number - 1].clone());
            rename_data.push(new_method_name);
            break;
        }
        clear_terminal(&mut stdout);
    }
    UserInputType::RenameTxMethod(rename_data)
}

/// Gets a new sequence of tx methods to reformat their location
#[cfg(not(tarpaulin_include))]
fn get_reposition_data(conn: &Connection) -> UserInputType {
    let mut stdout = stdout();

    clear_terminal(&mut stdout);
    let mut reposition_data = Vec::new();

    let tx_methods = get_all_tx_methods(conn);

    'outer_loop: loop {
        let mut method_line = "Select Transaction Method number sequence to proceed. Input 'Cancel' to cancel the operation.

Example input: 4 2 1 3, 3412
        
Currently added Transaction Methods: \n".to_string();

        for (i, item) in tx_methods.iter().enumerate() {
            method_line.push_str(&format!("\n{}. {}", i + 1, item));
        }
        println!("{method_line}");
        print!("\nEnter Transaction Methods sequence: ");
        flush_output(&stdout);

        let sequence_input = take_input().replace(' ', "");

        if sequence_input.is_empty() {
            clear_terminal(&mut stdout);
            continue;
        }

        if sequence_input.trim().to_lowercase().starts_with("cancel") {
            return UserInputType::CancelledOperation;
        }

        if sequence_input.len() > tx_methods.len() {
            clear_terminal(&mut stdout);
            println!(
                "Sequence number length cannot be greater than existing Transaction Methods.\n"
            );
            continue;
        }

        for char in sequence_input.chars() {
            let sequence_num = char.to_digit(10);

            if let Some(num) = sequence_num {
                if num as usize > tx_methods.len() {
                    reposition_data.clear();
                    clear_terminal(&mut stdout);
                    println!("Invalid sequence number given.\n");
                    continue 'outer_loop;
                }

                if reposition_data.contains(&tx_methods[num as usize - 1]) {
                    reposition_data.clear();
                    clear_terminal(&mut stdout);
                    println!("Cannot enter the same Transaction Method position twice.\n");
                    continue 'outer_loop;
                }

                reposition_data.push(tx_methods[num as usize - 1].to_string());
            } else {
                reposition_data.clear();
                clear_terminal(&mut stdout);
                println!("Invalid sequence number given.\n");
                continue 'outer_loop;
            }
        }

        if reposition_data == tx_methods {
            reposition_data.clear();
            clear_terminal(&mut stdout);
            println!("No positions to change.\n");
            continue;
        }

        println!("\nNew Transaction Methods positions: ");
        for (i, item) in reposition_data.iter().enumerate() {
            print!("\n{}. {}", i + 1, item);
        }

        print!("\n\nAccept the values? y/n: ");
        flush_output(&stdout);

        let confirm_operation = take_input();

        if confirm_operation.to_lowercase().starts_with('y') {
            break;
        }

        reposition_data.clear();
        clear_terminal(&mut stdout);
    }

    UserInputType::RepositionTxMethod(reposition_data)
}

/// Asks the user for a location where the app data will be stored
#[cfg(not(tarpaulin_include))]
fn get_new_location() -> UserInputType {
    let mut stdout = stdout();

    clear_terminal(&mut stdout);

    loop {
        let initial_line = "Enter a new location where the app will look for data. The location must start from root. 
        
If the location does not exist, all missing folders will be created and app data will be copied. 

Empty input will be considered as reset any saved location.

Example location:
        
Linux: /mnt/sdb1/data/save/
Windows: C:\\data\\save\\";

        println!("{initial_line}");
        print!("\nEnter a new location: ");
        flush_output(&stdout);

        let given_location = take_input();

        if given_location.is_empty() {
            println!("Clearing saved location");
            return UserInputType::ResetData(ResetType::NewLocation);
        }

        if given_location.trim().to_lowercase().starts_with("cancel") {
            return UserInputType::CancelledOperation;
        }

        let target_path = PathBuf::from(given_location);

        if !target_path.is_absolute() {
            clear_terminal(&mut stdout);
            println!(
                "The path {} must be absolute/start from root of the filesystem.\n",
                target_path.to_string_lossy()
            );
            continue;
        }

        if let Err(e) = fs::create_dir_all(&target_path) {
            clear_terminal(&mut stdout);
            println!("The given path is not valid. Error: {e:?}\n");
            continue;
        }

        return UserInputType::SetNewLocation(target_path);
    }
}

#[cfg(not(tarpaulin_include))]
fn get_backup_db_paths() -> UserInputType {
    let mut stdout = stdout();

    clear_terminal(&mut stdout);

    loop {
        let mut backup_paths = Vec::new();

        let initial_line = "Enter one or more location where the current app DB will be saved as a backup. The location must start from root. 
        
If the location does not exist, all missing folders will be created. Separate multiple paths with a comma (,).

If previously saved paths exists, they will be overwritten.

Empty input will be considered as reset all saved backup paths.

Example input:

Linux: /mnt/sdb1/data/save/, /mnt/sdb1/another/backup/, /mnt/sdb1/backup/
Windows: C:\\data\\save\\, C:\\backup\\save\\, C:\\folder\\app\\";

        println!("{initial_line}");
        print!("\nEnter backup locations: ");
        flush_output(&stdout);

        let given_location = take_input();

        if given_location.is_empty() {
            println!("Clearing all backup paths");
            return UserInputType::ResetData(ResetType::BackupDB);
        }

        if given_location.trim().to_lowercase().starts_with("cancel") {
            return UserInputType::CancelledOperation;
        }

        let all_locations = given_location.split(',');

        let mut failed_path_verification = false;

        for location in all_locations {
            let target_path = PathBuf::from(location.trim());

            if !target_path.is_absolute() {
                clear_terminal(&mut stdout);
                println!(
                    "The path {} must be absolute/start from root of the filesystem.\n",
                    target_path.to_string_lossy()
                );
                failed_path_verification = true;
                break;
            }

            if let Err(e) = fs::create_dir_all(&target_path) {
                clear_terminal(&mut stdout);
                println!(
                    "The path {} is not valid. Error: {e:?}\n",
                    target_path.to_string_lossy()
                );
                failed_path_verification = true;
                break;
            }
            backup_paths.push(target_path);
        }

        if failed_path_verification {
            continue;
        }
        return UserInputType::BackupDBPath(backup_paths);
    }
}

/// Tries to open terminal/cmd and run this app
/// Currently supports windows cmd, konsole, gnome-terminal, kgx (also known as gnome-console)
#[cfg(not(tarpaulin_include))]
pub fn start_terminal(original_dir: &str) -> Result<(), TerminalExecutionError> {
    if cfg!(target_os = "windows") {
        Command::new("cmd.exe")
            .arg("start")
            .arg("rex")
            .output()
            .map_err(TerminalExecutionError::ExecutionFailed)?;
    } else {
        let mut all_terminals = HashMap::new();
        let gnome_dir = format!("--working-directory={original_dir}");

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
                gnome_dir.clone(),
                "--maximize".to_string(),
                "--".to_string(),
                "./rex".to_string(),
            ],
        );

        all_terminals.insert(
            "kgx",
            vec![gnome_dir, "-e".to_string(), "./rex".to_string()],
        );

        let mut terminal_opened = false;
        let mut result = None;

        for (key, value) in all_terminals {
            let status = Command::new(key).args(value).output();
            match status {
                Ok(out) => {
                    if out.stderr.len() > 2 {
                        result = Some(TerminalExecutionError::NotFound(out));
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
    }
    Ok(())
}

/// Creates the query to search for specific tx, gathers all rows and id numbers
pub fn get_search_data(
    date: &str,
    details: &str,
    from_method: &str,
    to_method: &str,
    amount: &str,
    tx_type: &str,
    tags: &str,
    date_type: &DateType,
    conn: &Connection,
) -> (Vec<Vec<String>>, Vec<String>) {
    let mut all_txs = Vec::new();
    let mut all_ids = Vec::new();

    // This will be used for the activity tx
    let tx_method = if tx_type == "Transfer" && !from_method.is_empty() && !to_method.is_empty() {
        format!("{from_method} to {to_method}").trim().to_string()
    } else if tx_type == "Transfer" && !from_method.is_empty() && to_method.is_empty() {
        format!("{from_method} to ?").trim().to_string()
    } else if tx_type == "Transfer" && from_method.is_empty() && !to_method.is_empty() {
        format!("? to {to_method}").trim().to_string()
    } else if from_method.is_empty() && to_method.is_empty() {
        String::new()
    } else {
        from_method.to_string()
    };

    let mut valid_fields = 0;

    let mut query = "SELECT * FROM tx_all WHERE 1=1".to_string();

    if !date.is_empty() {
        valid_fields += 1;
        match date_type {
            DateType::Exact => query.push_str(&format!(r#" AND date = "{date}""#)),
            DateType::Monthly => {
                let split_date: Vec<usize> = date.split('-').map(|s| s.parse().unwrap()).collect();

                let month_index = split_date[1] - 1;
                let year_index = split_date[0] - 2022;

                let (date_1, date_2) = get_sql_dates(month_index, year_index, date_type);

                query.push_str(&format!(
                    r#" AND date BETWEEN date("{date_1}") AND date("{date_2}")"#,
                ));
            }
            DateType::Yearly => {
                let year_index = date.parse::<usize>().unwrap() - 2022;

                let (date_1, date_2) = get_sql_dates(0, year_index, date_type);

                query.push_str(&format!(
                    r#" AND date BETWEEN date("{date_1}") AND date("{date_2}")"#
                ));
            }
        }
    }

    if !details.is_empty() {
        valid_fields += 1;
        query.push_str(&format!(r#" AND details LIKE "%{details}%""#,));
    }

    if !tx_type.is_empty() {
        valid_fields += 1;
        query.push_str(&format!(r#" AND tx_type = "{tx_type}""#,));
    }

    if !amount.is_empty() {
        valid_fields += 1;
        let comparison_type = check_comparison(amount);

        let comparison_symbol = match comparison_type {
            ComparisonType::BiggerThan => ">",
            ComparisonType::SmallerThan => "<",
            ComparisonType::Equal => "",
            ComparisonType::EqualOrBigger => ">=",
            ComparisonType::EqualOrSmaller => "<=",
        };
        let amount = amount.replace(comparison_symbol, "");

        query.push_str(&format!(r#" AND {comparison_type} "{amount}""#));
    }

    if tx_type == "Transfer" {
        // If neither are empty, look for exact matches
        // Otherwise do partial matching
        if !from_method.is_empty() && !to_method.is_empty() {
            valid_fields += 1;
            query.push_str(&format!(
                r#" AND tx_method = "{from_method} to {to_method}""#
            ));
        } else if !from_method.is_empty() {
            valid_fields += 1;
            query.push_str(&format!(r#" AND tx_method LIKE "{from_method} to %" "#));
        } else if !to_method.is_empty() {
            valid_fields += 1;
            query.push_str(&format!(r#" AND tx_method LIKE "% to {to_method}""#));
        }
    } else if tx_type != "Transfer" && !from_method.is_empty() {
        valid_fields += 1;
        query.push_str(&format!(r#" AND tx_method = "{from_method}""#));
    }

    if !tags.is_empty() {
        valid_fields += 1;
        let all_tags = tags.split(", ");
        let tag_conditions = all_tags
            .map(|tag| {
                format!(
                    r#"CASE 
                          WHEN tags LIKE "{tag}, %" THEN 1
                          WHEN tags LIKE "%, {tag}" THEN 1
                          WHEN tags LIKE "%, {tag}," THEN 1
                          WHEN tags = "{tag}" THEN 1
                          ELSE 0
                      END = 1"#,
                )
            })
            .collect::<Vec<String>>()
            .join(" OR ");
        query.push_str(&format!(" AND ({tag_conditions})"));
    }

    let mut statement = conn.prepare(&query).unwrap();

    let rows = statement
        .query_map([], |row| {
            let date = reverse_date_format(row.get(0).unwrap());
            let id_num: i32 = row.get(5).unwrap();

            Ok(vec![
                date,
                row.get(1).unwrap(),
                row.get(2).unwrap(),
                row.get(3).unwrap(),
                row.get(4).unwrap(),
                row.get(6).unwrap(),
                id_num.to_string(),
            ])
        })
        .unwrap();

    for i in rows.flatten() {
        let mut data = i;
        let id_num = &data.pop().unwrap();
        all_ids.push(id_num.to_string());
        all_txs.push(data);
    }

    let activity_type = ActivityType::SearchTX(Some(valid_fields));
    let search_data = vec![date, details, &tx_method, amount, tx_type, tags, ""];

    let activity_num = add_new_activity(activity_type, conn);
    add_new_activity_tx(&search_data, activity_num, conn);

    (all_txs, all_ids)
}

/// Returns all activities recorded within a given month and a year and all the activity txs related to the activities
pub fn get_all_activities(
    month: usize,
    year: usize,
    conn: &Connection,
) -> (Vec<ActivityDetails>, HashMap<i32, Vec<ActivityTx>>) {
    let (datetime_1, datetime_2) = get_sql_dates(month, year, &DateType::Monthly);

    let mut statement = conn
        .prepare("SELECT * from activities WHERE date BETWEEN date(?) AND date(?)")
        .unwrap();

    // Activity tx fetching happens based on the minimum activity num and the maximum activity num
    let mut min_activity_num = i32::MAX;
    let mut max_activity_num = i32::MIN;

    let rows: Vec<ActivityDetails> = statement
        .query_map([datetime_1, datetime_2], |row| {
            let date = reverse_date_format(row.get(0).unwrap());
            Ok(ActivityDetails::new(
                date,
                row.get(1).unwrap(),
                row.get(2).unwrap(),
                row.get(3).unwrap(),
            ))
        })
        .unwrap()
        .map(|wrapped_data| {
            let data = wrapped_data.unwrap();
            let id = data.activity_num();

            if id > max_activity_num {
                max_activity_num = id;
            }

            if id < min_activity_num {
                min_activity_num = id;
            }
            data
        })
        .collect();

    let activity_txs = get_all_activity_txs(min_activity_num, max_activity_num, conn);

    (rows, activity_txs)
}

/// Returns all activity txs within the given activity number range
pub fn get_all_activity_txs(
    min_num: i32,
    max_num: i32,
    conn: &Connection,
) -> HashMap<i32, Vec<ActivityTx>> {
    let mut statement = conn
        .prepare("SELECT * from activity_txs WHERE activity_num >= ? AND activity_num <= ?")
        .unwrap();

    // Contains data in the format {activity_num: Vec<ActivityTx>}
    // In case of search or edit txs, a single activity can impact multiple txs
    // The rest will always have only 1 tx
    let mut activity_tx_data = HashMap::new();

    for wrapped_data in statement
        .query_map([min_num, max_num], |row| {
            let date = reverse_date_format(row.get(0).unwrap());
            Ok(ActivityTx::new(
                date,
                row.get(1).unwrap(),
                row.get(2).unwrap(),
                row.get(3).unwrap(),
                row.get(4).unwrap(),
                row.get(5).unwrap(),
                row.get(6).unwrap(),
                row.get(7).unwrap(),
                row.get(8).unwrap(),
            ))
        })
        .unwrap()
    {
        let data = wrapped_data.unwrap();
        activity_tx_data
            .entry(data.activity_num())
            .or_insert(Vec::new())
            .push(data);
    }

    activity_tx_data
}
