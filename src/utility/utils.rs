use crate::db::{add_tags_column, create_db, update_balance_type, YEARS};
use crate::page_handler::{IndexedData, UserInputType, BACKGROUND, BOX, HIGHLIGHTED, TEXT};
use crate::utility::get_user_tx_methods;
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen,
};
use rusqlite::{Connection, Result as sqlResult};
use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::io::{stdout, Stdout, Write};
use std::time::Duration;
use std::{process, thread};
use tui::backend::CrosstermBackend;
use tui::style::{Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, BorderType, Borders, Tabs};
use tui::Terminal;

/// Makes a call to the database to find out all the columns in the balance_all section
/// so we can determine the number of TX Methods that has been added.
/// return example: `["source_1", "source_2", "source_3"]`
pub fn get_all_tx_methods(conn: &Connection) -> Vec<String> {
    // returns all transaction methods added to the database
    let column_names = conn
        .prepare("SELECT * FROM balance_all")
        .expect("could not prepare statement");

    let mut data: Vec<String> = column_names
        .column_names()
        .iter()
        .map(|c| c.to_string())
        .collect();
    data.remove(0);
    data
}

pub fn get_all_tags(conn: &Connection) -> Vec<String> {
    let mut query = conn
        .prepare("SELECT tags FROM tx_all")
        .expect("could not prepare statement");

    let mut tags_data: HashSet<String> = HashSet::new();

    if let Ok(rows) = query.query_map([], |row| {
        let row_data: String = row.get(0).unwrap();
        let splitted = row_data.split(',');
        let final_data = splitted
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

/// Gets all columns inside the tx_all table. Used to determine if the database needs to be migrated
pub fn get_all_tx_columns(conn: &Connection) -> Vec<String> {
    let column_names = conn
        .prepare("SELECT * FROM tx_all")
        .expect("could not prepare statement");

    column_names
        .column_names()
        .iter()
        .map(|c| c.to_string())
        .collect()
}

/// Returns the a vector with data required to create the Changes row for zero changes in the home page.
pub fn get_empty_changes(conn: &Connection) -> Vec<String> {
    // function for quick vec with 0 changes for adding in widget
    let tx_methods = get_all_tx_methods(conn);
    let mut changes = vec!["Changes".to_string()];
    for _i in tx_methods {
        changes.push(format!("{:.2}", 0.0))
    }
    changes
}

/// Returns the last id_num recorded by tx_all table
pub fn get_last_tx_id(conn: &Connection) -> sqlResult<i32> {
    let last_id: sqlResult<i32> = conn.query_row(
        "SELECT id_num FROM tx_all ORDER BY id_num DESC LIMIT 1",
        [],
        |row| row.get(0),
    );
    last_id
}

/// Returns the last id_num recorded by balance_all table or the id_num of the absolute final balance
pub fn get_last_balance_id(conn: &Connection) -> sqlResult<i32> {
    let last_id: sqlResult<i32> = conn.query_row(
        "SELECT id_num FROM balance_all ORDER BY id_num DESC LIMIT 1",
        [],
        |row| row.get(0),
    );
    last_id
}

/// The function is used to create dates in the form of strings to use the WHERE statement
/// based on the month and year index that has been passed to it. Will return two dates to use in the
/// WHERE statement. Will return the 1st and the 31st date of the given month and year.
/// return example: `(2022-01-01, 2022-01-31)`
pub fn get_sql_dates(month: usize, year: usize) -> (String, String) {
    let datetime_1 = format!("{}-{:02}-01", YEARS[year], month + 1);
    let datetime_2 = format!("{}-{:02}-31", YEARS[year], month + 1);
    (datetime_1, datetime_2)
}

/// Verifies the db version is up to date
pub fn check_old_sql(conn: &mut Connection) {
    // * earlier version of the database didn't had the Tag column
    if !get_all_tx_columns(conn).contains(&"tags".to_string()) {
        println!("Old database detected. Starting migration...");
        let status = add_tags_column(conn);
        match status {
            Ok(_) => start_timer("Database migration successfully complete."),
            Err(e) => {
                println!("Database migration failed. Try again. Error: {}", e);
                println!("Commits reversed. Exiting...");
                process::exit(1);
            }
        }
    }

    // * earlier version of the database's balance_all columns were all TEXT type.
    // * Convert to REAL type if found
    if check_old_balance_sql(conn) {
        println!("Outdated database detected. Updating...");
        let status = update_balance_type(conn);
        match status {
            Ok(_) => start_timer("Database updating successfully complete."),
            Err(e) => {
                println!("Database updating failed. Try again. Error: {}", e);
                println!("Commits reversed. Exiting...");
                process::exit(1);
            }
        }
    }
}

fn check_old_balance_sql(conn: &Connection) -> bool {
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

/// Enters raw mode so the Tui can render properly
pub fn enter_tui_interface() -> Result<Terminal<CrosstermBackend<Stdout>>, Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// Exits raw mode so the terminal starts working normally
pub fn exit_tui_interface() -> Result<(), Box<dyn Error>> {
    let stdout = stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    disable_raw_mode()?;
    Ok(())
}

pub fn check_n_create_db(verifying_path: &str) -> Result<(), Box<dyn Error>> {
    // checks the local folder and searches for data.sqlite
    let paths = fs::read_dir(".")?;
    let mut db_found = false;
    for path in paths {
        let path = path?.path().display().to_string();
        if path == verifying_path {
            db_found = true;
        }
    }
    if !db_found {
        let mut conn = Connection::open(verifying_path)?;
        let db_tx_methods =
            if let UserInputType::AddNewTxMethod(inner_value) = get_user_tx_methods(false, &conn) {
                inner_value
            } else {
                return Err("Failed to get tx methods.".into());
            };
        println!("Creating New Database. It may take some time...");
        let status = create_db(db_tx_methods, &mut conn);
        match status {
            Ok(_) => start_timer("Database creation successfully complete."),
            Err(e) => {
                println!("Database creation failed. Try again. Error: {}", e);
                fs::remove_file("data.sqlite")?;
                process::exit(1);
            }
        }
        conn.close().unwrap();
    }
    Ok(())
}

/// Returns a styled block for ui to use
pub fn styled_block(title: &str) -> Block {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().bg(BACKGROUND).fg(BOX))
        .title(Span::styled(
            title,
            Style::default().add_modifier(Modifier::BOLD),
        ))
}

pub fn main_block<'a>() -> Block<'a> {
    Block::default().style(Style::default().bg(BACKGROUND).fg(BOX))
}

/// takes a string and makes any word before the first occurrence of : to Bold
/// Used for rendering
pub fn create_bolded_text(text: &str) -> Vec<Spans> {
    let mut text_data = Vec::new();

    for line in text.split('\n') {
        let splitted = line.split_once(':');
        if let Some((first_part, rest)) = splitted {
            let first_data =
                Span::styled(first_part, Style::default().add_modifier(Modifier::BOLD));
            let rest_data = Span::from(format!(":{rest}"));
            text_data.push(Spans::from(vec![first_data, rest_data]));
        } else {
            text_data.push(Spans::from(vec![Span::from(line)]))
        }
    }

    text_data
}

/// Tabs from some given data for the UI
pub fn create_tab<'a>(data: &'a IndexedData, name: &'a str) -> Tabs<'a> {
    let titles = data
        .titles
        .iter()
        .map(|t| Spans::from(vec![Span::styled(t, Style::default().fg(TEXT))]))
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

/// Does the 5 second timer after input taking ends
pub fn start_timer<T: std::fmt::Display>(input: T) {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    for i in (1..6).rev() {
        write!(handle, "\r{input} Restarting in {i} seconds").unwrap();
        handle.flush().unwrap();
        thread::sleep(Duration::from_millis(1000));
    }
}

/// Takes a user input and returns the trimmed input as String
pub fn take_input() -> String {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

/// Clears the terminal of all text
pub fn clear_terminal(stdout: &mut Stdout) {
    execute!(stdout, Clear(ClearType::FromCursorUp)).unwrap();
}

/// Flushes output to the terminal
pub fn flush_output(stdout: &Stdout) {
    let mut handle = stdout.lock();
    handle.flush().unwrap();
}
