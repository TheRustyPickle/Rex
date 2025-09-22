use rusqlite::Connection;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::stdout;
use std::path::PathBuf;
use std::process::Command;

use crate::outputs::TerminalExecutionError;
use crate::page_handler::{ResetType, UserInputType};
use crate::utility::{
    check_restricted, clear_terminal, flush_output, get_all_tx_methods, take_input,
};

/// Prompts the user to select and option and start taking relevant inputs
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
