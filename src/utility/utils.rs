use rusqlite::{Connection, Result as sqlResult};

/// Gives the month and year name by index
pub fn get_month_name(month_index: usize, year_index: usize) -> (String, String) {
    let month_names = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];

    let years = ["2022", "2023", "2024", "2025"];

    (
        month_names[month_index].to_string(),
        years[year_index].to_string(),
    )
}

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

/// Gets all columns inside the tx_all table. Used to determine if the database needs to be migrated
pub fn get_all_tx_columns(file_name: &str) -> Vec<String> {
    let conn = Connection::open(file_name).expect("Could not connect to database");

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
    let new_month: String = if month < 10 {
        format!("0{}", month)
    } else {
        format!("{}", month)
    };
    let mut new_year = year.to_string();

    if year + 1 < 10 {
        new_year = format!("202{}", year + 2);
    }
    let datetime_1 = format!("{}-{}-01", new_year, new_month);
    let datetime_2 = format!("{}-{}-31", new_year, new_month);
    (datetime_1, datetime_2)
}
