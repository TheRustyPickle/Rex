use chrono::{Local, Months, NaiveDate};
use crossterm::execute;
use crossterm::terminal::{
    Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Tabs};
use rusqlite::{Connection, Result as sqlResult};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::Display;
use std::fs::{self, File};
use std::io::{Read, Result as ioResult, Stdout, Write, stdout};
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{process, thread};
use strsim::normalized_levenshtein;

use crate::db::{YEARS, add_tags_column, create_db, migrate_to_activities, update_balance_type};
use crate::outputs::ComparisonType;
use crate::page_handler::{
    ActivityType, BACKGROUND, BOX, DateType, HIGHLIGHTED, IndexedData, RED, SortingType, TEXT,
    UserInputType,
};
use crate::utility::get_user_tx_methods;

const RESTRICTED: [&str; 6] = ["Total", "Balance", "Changes", "Income", "Expense", "Cancel"];

/// Makes a call to the database to find out all the columns in the `balance_all` section
/// so we can determine the number of TX Methods that has been added.
/// Return example: `["source_1", "source_2", "source_3"]`
pub fn get_all_tx_methods(conn: &Connection) -> Vec<String> {
    // Returns all transaction methods added to the database
    let column_names = conn
        .prepare("SELECT * FROM balance_all")
        .expect("could not prepare statement");

    let mut data: Vec<String> = column_names
        .column_names()
        .iter()
        .map(ToString::to_string)
        .collect();
    data.remove(0);
    data
}

pub fn get_all_tx_methods_cumulative(conn: &Connection) -> Vec<String> {
    // Returns all transaction methods added to the database
    let column_names = conn
        .prepare("SELECT * FROM balance_all")
        .expect("could not prepare statement");

    let mut data: Vec<String> = column_names
        .column_names()
        .iter()
        .map(ToString::to_string)
        .collect();
    data.remove(0);
    data.push("Cumulative".to_string());
    data
}

/// Returns all unique tags from the db
pub fn get_all_tags(conn: &Connection) -> Vec<String> {
    let mut query = conn
        .prepare("SELECT tags FROM tx_all")
        .expect("could not prepare statement");

    let mut tags_data: HashSet<String> = HashSet::new();

    if let Ok(rows) = query.query_map([], |row| {
        let row_data: String = row.get(0).unwrap();
        let split_tags = row_data.split(',');
        let final_data = split_tags
            .into_iter()
            .map(|s| s.trim().to_string())
            .collect::<Vec<String>>();
        Ok(final_data)
    }) {
        for inner_data in rows.flatten() {
            for x in inner_data {
                tags_data.insert(x);
            }
        }
    }

    let mut sorted_tags = tags_data.into_iter().collect::<Vec<String>>();
    sorted_tags.sort();

    sorted_tags
}

/// Returns all unique details from the db
pub fn get_all_details(conn: &Connection) -> Vec<String> {
    let mut query = conn
        .prepare("SELECT details FROM tx_all")
        .expect("could not prepare statement");

    let mut details_data: HashSet<String> = HashSet::new();

    if let Ok(rows) = query.query_map([], |row| {
        let row_data: String = row.get(0).unwrap();
        let split_text = row_data.split(',');
        let final_data = split_text
            .into_iter()
            .map(|s| s.trim().to_string())
            .collect::<Vec<String>>();
        Ok(final_data)
    }) {
        for inner_data in rows.flatten() {
            for x in inner_data {
                details_data.insert(x);
            }
        }
    }

    let mut sorted_details = details_data.into_iter().collect::<Vec<String>>();
    sorted_details.sort();
    sorted_details
}

/// Gets all columns inside the `tx_all` table. Used to determine if the database needs to be migrated
pub fn get_all_tx_columns(conn: &Connection) -> Vec<String> {
    let column_names = conn
        .prepare("SELECT * FROM tx_all")
        .expect("could not prepare statement");

    column_names
        .column_names()
        .iter()
        .map(ToString::to_string)
        .collect()
}

/// Returns the a vector with data required to create the Changes row for zero changes in the homepage.
pub fn get_empty_changes(conn: &Connection) -> Vec<String> {
    // Function for quick vec with 0 changes for adding in widget
    let tx_methods = get_all_tx_methods(conn);
    let mut changes = vec!["Changes".to_string()];
    for _i in tx_methods {
        changes.push(format!("{:.2}", 0.0));
    }
    changes
}

/// Returns the last `id_num` recorded by `tx_all` table
pub fn get_last_tx_id(conn: &Connection) -> sqlResult<i32> {
    let last_id: sqlResult<i32> = conn.query_row(
        "SELECT id_num FROM tx_all ORDER BY id_num DESC LIMIT 1",
        [],
        |row| row.get(0),
    );
    last_id
}

/// Returns the last `id_num` recorded by `balance_all` table or the `id_num` of the absolute final balance
pub fn get_last_balance_id(conn: &Connection) -> sqlResult<i32> {
    let last_id: sqlResult<i32> = conn.query_row(
        "SELECT id_num FROM balance_all ORDER BY id_num DESC LIMIT 1",
        [],
        |row| row.get(0),
    );
    last_id
}

/// Returns two dates based on the month and year index. Used for the purpose of searching
/// tx based on date
#[must_use]
pub fn get_sql_dates(month: usize, year: usize, date_type: &DateType) -> (String, String) {
    match date_type {
        DateType::Monthly => {
            let datetime_1 = format!("{}-{:02}-01", YEARS[year], month + 1);
            let last_date =
                NaiveDate::from_ymd_opt(YEARS[year].parse().unwrap(), (month + 1) as u32, 1)
                    .unwrap()
                    .checked_add_months(Months::new(1))
                    .unwrap()
                    .pred_opt()
                    .unwrap();
            (datetime_1, last_date.to_string())
        }
        DateType::Yearly => {
            let datetime_1 = format!("{}-01-01", YEARS[year]);
            let datetime_2 = format!("{}-12-31", YEARS[year]);
            (datetime_1, datetime_2)
        }
        DateType::Exact => (String::new(), String::new()),
    }
}

/// Verifies the db version is up to date
#[cfg(not(tarpaulin_include))]
pub fn check_old_sql(conn: &mut Connection) {
    // Earlier version of the database didn't had the Tag column
    if !get_all_tx_columns(conn).contains(&"tags".to_string()) {
        println!("Old database detected. Starting migration...");
        let status = add_tags_column(conn);
        match status {
            Ok(()) => start_timer("Database migration successfully complete."),
            Err(e) => {
                println!("Database migration failed. Try again. Error: {e}");
                println!("Commits reversed. Exiting...");
                process::exit(1);
            }
        }
    }

    // Earlier version of the database's balance_all columns were all TEXT type.
    // Convert to REAL type if found
    if check_old_balance_sql(conn) {
        println!("Outdated database detected. Updating...");
        let status = update_balance_type(conn);
        match status {
            Ok(()) => start_timer("Database updating successfully complete."),
            Err(e) => {
                println!("Database updating failed. Try again. Error: {e}");
                println!("Commits reversed. Exiting...");
                process::exit(1);
            }
        }
    }

    if !get_all_table_names(conn).contains(&"activities".to_string()) {
        println!("Outdated database detected. Updating...");
        let status = migrate_to_activities(conn);
        match status {
            Ok(()) => start_timer("Database updating successfully complete."),
            Err(e) => {
                println!("Database updating failed. Try again. Error: {e}");
                println!("Commits reversed. Exiting...");
                process::exit(1);
            }
        }
    }
}

/// Checks if the `balance_all` table is outdated
pub fn check_old_balance_sql(conn: &Connection) -> bool {
    let mut query = conn.prepare("PRAGMA table_info(balance_all)").unwrap();

    let columns = query
        .query_map([], |row| Ok((row.get(1).unwrap(), row.get(2).unwrap())))
        .unwrap();

    let mut result = false;

    for column in columns {
        let (name, data_type): (String, String) = column.unwrap();
        if name != "id_num" && data_type == "TEXT" {
            result = true;
            break;
        }
    }
    result
}

pub fn get_all_table_names(conn: &Connection) -> Vec<String> {
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table'")
        .unwrap();
    let table_names = stmt.query_map([], |row| row.get(0)).unwrap();

    let mut result = Vec::new();
    for table_name in table_names {
        result.push(table_name.unwrap());
    }

    result
}

/// Enters raw mode so the TUI can render properly
#[cfg(not(tarpaulin_include))]
pub fn enter_tui_interface() -> Result<Terminal<CrosstermBackend<Stdout>>, Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// Exits raw mode so the terminal starts working normally
#[cfg(not(tarpaulin_include))]
pub fn exit_tui_interface() -> Result<(), Box<dyn Error>> {
    let stdout = stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    disable_raw_mode()?;
    Ok(())
}

/// Checks if a db already exists or prompts to create a new one
#[cfg(not(tarpaulin_include))]
pub fn check_n_create_db(verifying_path: &PathBuf) -> Result<(), Box<dyn Error>> {
    if !verifying_path.exists() {
        let UserInputType::AddNewTxMethod(db_tx_methods) = get_user_tx_methods(false, None) else {
            return Err("Failed to get tx methods.".into());
        };
        println!("Creating New Database. It may take some time...");

        let mut conn = Connection::open(verifying_path)?;
        let status = create_db(&db_tx_methods, &mut conn);
        conn.close().unwrap();
        match status {
            Ok(()) => start_timer("Database creation successful."),
            Err(e) => {
                println!("Database creation failed. Try again. Error: {e}");
                fs::remove_file("data.sqlite")?;
                process::exit(1);
            }
        }
    }
    Ok(())
}

/// Returns a styled block for UI to use
#[cfg(not(tarpaulin_include))]
#[must_use]
pub fn styled_block(title: &str) -> Block<'_> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().bg(BACKGROUND).fg(BOX))
        .title(Span::styled(
            title,
            Style::default().add_modifier(Modifier::BOLD),
        ))
}

pub fn styled_block_no_top(title: &str) -> Block<'_> {
    Block::default()
        .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
        .border_type(BorderType::Rounded)
        .style(Style::default().bg(BACKGROUND).fg(BOX))
        .title(Span::styled(
            title,
            Style::default().add_modifier(Modifier::BOLD),
        ))
}

pub fn styled_block_no_bottom(title: &str) -> Block<'_> {
    Block::default()
        .borders(Borders::LEFT | Borders::RIGHT | Borders::TOP)
        .border_type(BorderType::Rounded)
        .style(Style::default().bg(BACKGROUND).fg(BOX))
        .title(Span::styled(
            title,
            Style::default().add_modifier(Modifier::BOLD),
        ))
}

#[cfg(not(tarpaulin_include))]
#[must_use]
pub fn main_block<'a>() -> Block<'a> {
    Block::default().style(Style::default().bg(BACKGROUND).fg(BOX))
}

/// Takes a string and makes any word before the first occurrence of : to Bold
/// Used for rendering
#[cfg(not(tarpaulin_include))]
#[must_use]
pub fn create_bolded_text(text: &str) -> Vec<Line<'_>> {
    let mut text_data = Vec::new();

    for line in text.split('\n') {
        let split_text = line.split_once(':');
        if let Some((first_part, rest)) = split_text {
            let first_data =
                Span::styled(first_part, Style::default().add_modifier(Modifier::BOLD));
            let rest_data = Span::from(format!(":{rest}"));
            text_data.push(Line::from(vec![first_data, rest_data]));
        } else {
            text_data.push(Line::from(vec![Span::from(line)]));
        }
    }

    text_data
}

/// Tabs from some given data for the UI
#[cfg(not(tarpaulin_include))]
#[must_use]
pub fn create_tab<'a>(data: &'a IndexedData, name: &'a str) -> Tabs<'a> {
    let titles: Vec<Line> = data
        .titles
        .iter()
        .map(|t| Line::from(vec![Span::styled(t, Style::default().fg(TEXT))]))
        .collect();

    Tabs::new(titles)
        .block(styled_block(name))
        .select(data.index)
        .style(Style::default().fg(BOX))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(HIGHLIGHTED),
        )
}

/// Create a tab with some values where each value's color will depend on the provided `HashMap` bool value
#[cfg(not(tarpaulin_include))]
pub fn create_tab_activation<'a, S: ::std::hash::BuildHasher>(
    data: &'a IndexedData,
    name: &'a str,
    // No idea what the BuildHasher does. Clippy pedantic said it so I did it.
    activation: &HashMap<String, bool, S>,
) -> Tabs<'a> {
    let titles: Vec<Line> = data
        .titles
        .iter()
        .map(|t| {
            if activation[t] {
                Line::from(vec![Span::styled(t, Style::default().fg(TEXT))])
            } else {
                Line::from(vec![Span::styled(t, Style::default().fg(RED))])
            }
        })
        .collect();

    Tabs::new(titles)
        .block(styled_block(name))
        .select(data.index)
        .style(Style::default().fg(BOX))
        .highlight_style(Style::default())
}

/// Does the 5 second timer after input taking ends
#[cfg(not(tarpaulin_include))]
pub fn start_timer<T: std::fmt::Display>(input: T) {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    for i in (1..6).rev() {
        write!(handle, "\r{input} Restarting in {i} seconds").unwrap();
        handle.flush().unwrap();
        thread::sleep(Duration::from_millis(1000));
    }
    println!("\n");
}

/// Takes a user input and returns the trimmed input as String
#[cfg(not(tarpaulin_include))]
#[must_use]
pub fn take_input() -> String {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

/// Clears the terminal of all text
#[cfg(not(tarpaulin_include))]
pub fn clear_terminal(stdout: &mut Stdout) {
    execute!(stdout, Clear(ClearType::FromCursorUp)).unwrap();
}

/// Flushes output to the terminal
#[cfg(not(tarpaulin_include))]
pub fn flush_output(stdout: &Stdout) {
    let mut handle = stdout.lock();
    handle.flush().unwrap();
}

/// Checks if the input is a restricted word or inside a given vector
#[must_use]
pub fn check_restricted(item: &str, restricted: Option<&Vec<String>>) -> bool {
    if let Some(restricted_words) = restricted {
        for restricted_item in restricted_words {
            if restricted_item.to_lowercase() == item.to_lowercase() {
                return true;
            }
        }
    } else {
        for &restricted_item in &RESTRICTED {
            if restricted_item.to_lowercase() == item.to_lowercase() {
                return true;
            }
        }
    }

    false
}

/// Parse GitHub release information for popup menu
#[must_use]
pub fn parse_github_body(body: &str) -> String {
    let body = body.replace("## Updates", "");
    let body = body.replace('*', "•");
    let body = body.replace('\r', "");
    let end_point = body.find("## Changes").unwrap();
    format!("\n{}\n", &body[..end_point].trim())
}

/// Uses Levenshtein algorithm to get the best match of a string in a vec of strings
#[must_use]
pub fn get_best_match(data: &str, matching_set: &[String]) -> String {
    let mut best_match = &matching_set[0];
    let mut best_score = -1.0;

    for x in matching_set {
        let new_score = normalized_levenshtein(&x.to_lowercase(), &data.to_lowercase());

        if new_score > best_score {
            best_match = x;
            best_score = new_score;
        }
    }
    best_match.to_string()
}

/// Used for sorting summary table data
#[must_use]
pub fn sort_table_data(mut data: Vec<Vec<String>>, sort_type: &SortingType) -> Vec<Vec<String>> {
    match sort_type {
        SortingType::ByTags => data.sort(),
        SortingType::ByIncome => data.sort_by(|a, b| {
            let val_a: f64 = a[1].parse().unwrap();
            let val_b: f64 = b[1].parse().unwrap();
            val_b.partial_cmp(&val_a).unwrap()
        }),
        SortingType::ByExpense => {
            data.sort_by(|a, b| {
                let val_a: f64 = a[2].parse().unwrap();
                let val_b: f64 = b[2].parse().unwrap();
                val_b.partial_cmp(&val_a).unwrap()
            });
        }
    }

    data
}

/// Adds a char to the given index on the given string
pub fn add_char_to(to_add: Option<char>, current_index: &mut usize, current_data: &mut String) {
    if *current_index > current_data.len() {
        *current_index = current_data.len();
    } else {
        match to_add {
            Some(ch) => {
                current_data.insert(*current_index, ch);
                *current_index += 1;
            }
            None => {
                if !current_data.is_empty() && *current_index != 0 {
                    current_data.remove(*current_index - 1);
                    *current_index -= 1;
                }
            }
        }
    }
}

/// Checks if the string contains any symbol indicating comparison
#[must_use]
pub fn check_comparison(input: &str) -> ComparisonType {
    // Need to handle 2 letter ones first otherwise in case of >=
    // it will match with >
    if input.starts_with("<=") {
        ComparisonType::EqualOrSmaller
    } else if input.starts_with(">=") {
        ComparisonType::EqualOrBigger
    } else if input.starts_with('<') {
        ComparisonType::SmallerThan
    } else if input.starts_with('>') {
        ComparisonType::BiggerThan
    } else {
        ComparisonType::Equal
    }
}

#[derive(Serialize, Deserialize)]
struct LocationInfo {
    location: String,
}

#[derive(Serialize, Deserialize)]
struct BackupPaths {
    locations: Vec<String>,
}

/// Checks if location.json exists and returns a path if it exists
#[must_use]
pub fn is_location_changed(working_dir: &PathBuf) -> Option<PathBuf> {
    let mut json_path = working_dir.to_owned();
    json_path.pop();
    json_path.push("location.json");

    if json_path.exists() {
        let mut file = File::open(json_path).unwrap();
        let mut file_content = String::new();
        file.read_to_string(&mut file_content).unwrap();
        let location_info: LocationInfo = serde_json::from_str(&file_content).unwrap();
        Some(PathBuf::from(location_info.location))
    } else {
        None
    }
}

/// Creates a location.json file to store the new app data location
pub fn create_change_location_file(working_dir: &PathBuf, new_path: &Path) {
    let mut target_dir = working_dir.to_owned();
    target_dir.pop();

    let location = LocationInfo {
        location: new_path.to_str().unwrap().to_string(),
    };

    target_dir.push("location.json");

    let mut file = File::create(target_dir).unwrap();

    serde_json::to_writer(&mut file, &location).unwrap();
}

/// Create a `backup_paths.json` file to store the location of where backup db will be located
pub fn create_backup_location_file(original_db_path: &PathBuf, backup_paths: Vec<PathBuf>) {
    let mut target_dir = original_db_path.to_owned();
    target_dir.pop();

    let backup = BackupPaths {
        locations: backup_paths
            .into_iter()
            .map(|path| path.to_str().unwrap().to_owned())
            .collect(),
    };

    target_dir.push("backup_paths.json");
    let mut file = File::create(target_dir).unwrap();
    serde_json::to_writer(&mut file, &backup).unwrap();
}

/// Copies the latest DB to the backup location specified in `backend_paths.json`
#[cfg(not(tarpaulin_include))]
pub fn save_backup_db(db_path: &PathBuf, original_db_path: &PathBuf) {
    let mut json_path = original_db_path.to_owned();
    json_path.pop();

    json_path.push("backup_paths.json");

    if !json_path.exists() {
        return;
    }

    let mut file = File::open(json_path).unwrap();
    let mut file_content = String::new();
    file.read_to_string(&mut file_content).unwrap();
    let location_info: BackupPaths = serde_json::from_str(&file_content).unwrap();

    for path in location_info.locations {
        let mut target_path = PathBuf::from(path);

        if !target_path.exists() {
            println!("Failed to find path {}", target_path.to_string_lossy());
            continue;
        }
        target_path.push("data.sqlite");
        if let Err(e) = fs::copy(db_path, &target_path) {
            println!(
                "Failed to copy DB to backup path {}. Error: {e:?}",
                target_path.to_string_lossy()
            );
            continue;
        }
    }
}

/// Deletes `backup_paths.json` which contains all locations where backup DB is located.
#[cfg(not(tarpaulin_include))]
pub fn delete_backup_db(original_db_path: &PathBuf) -> ioResult<()> {
    let mut json_path = original_db_path.to_owned();
    json_path.pop();

    json_path.push("backup_paths.json");

    if !json_path.exists() {
        return Ok(());
    }

    fs::remove_file(json_path)
}

/// Deletes `locations.json` file which stores alternative location information of the DB.
#[cfg(not(tarpaulin_include))]
pub fn delete_location_change(original_db_path: &PathBuf) -> ioResult<()> {
    let mut json_path = original_db_path.to_owned();
    json_path.pop();

    json_path.push("location.json");

    if !json_path.exists() {
        return Ok(());
    }

    fs::remove_file(json_path)
}

/// Returns a transaction detail from a given ID number
pub fn get_tx_id_num(id_num: i32, conn: &Connection) -> Vec<String> {
    let query = format!("SELECT * FROM tx_all WHERE id_num = {id_num}");

    let tx_data = conn.query_row(&query, [], |row| {
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
    });

    tx_data.unwrap()
}

/// Returns the details of the last added transaction
pub fn get_last_tx(conn: &Connection) -> Vec<String> {
    let query = "SELECT * FROM tx_all ORDER BY id_num DESC LIMIT 1";

    let tx_data = conn.query_row(query, [], |row| {
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
    });

    tx_data.unwrap()
}

/// Add a new activity row to the DB
pub fn add_new_activity(activity_type: ActivityType, conn: &Connection) -> i32 {
    let activity_type_str = activity_type.to_str();
    let activity_details = activity_type.to_details();
    let current_date = Local::now().date_naive().to_string();

    let query = format!(
        r#"INSERT INTO activities 
    (date, activity_type, description) 
    VALUES ("{current_date}", "{activity_type_str}", "{activity_details}")"#
    );

    conn.execute(&query, []).unwrap();

    // Fetch the latest row's activity num so this can be used to reference activity txs
    let query = "SELECT activity_num FROM activities ORDER BY activity_num DESC LIMIT 1";

    let activity_num = conn.query_row(query, [], |row| {
        let id_num: i32 = row.get(0).unwrap();
        Ok(id_num)
    });

    activity_num.unwrap()
}

/// Add a new tx that is related to a given activity number
pub fn add_new_activity_tx<T: AsRef<str> + Display>(
    tx_data: &[T],
    activity_num: i32,
    conn: &Connection,
) {
    let date = &tx_data[0];
    let details = &tx_data[1];
    let tx_method = &tx_data[2];
    let amount = &tx_data[3];
    let tx_type = &tx_data[4];
    let tags = &tx_data[5];
    let id_num = &tx_data[6];

    let mut string_date = date.to_string();
    let split_date = string_date.split('-').collect::<Vec<&str>>();

    if split_date[0].len() == 2 {
        string_date = reverse_date_format(string_date);
    }

    let query = format!(
        r#"INSERT INTO activity_txs 
    (date, details, tx_method, amount, tx_type, tags, id_num, activity_num)
    VALUES ("{string_date}", "{details}", "{tx_method}", "{amount}", "{tx_type}", "{tags}", "{id_num}", "{activity_num}")"#
    );

    conn.execute(&query, []).unwrap();
}

/// Switch from YYYY-MM-DD to DD-MM-YYYY or vice versa.
/// Will return the original value if either empty or does not have 2 dashes in the string
#[must_use]
pub fn reverse_date_format(date: String) -> String {
    if date.is_empty() {
        return date;
    }
    let collected_date = date.split('-').collect::<Vec<&str>>();
    if collected_date.len() != 3 {
        return date;
    }
    let new_date = format!(
        "{}-{}-{}",
        collected_date[2], collected_date[1], collected_date[0]
    );
    new_date
}
