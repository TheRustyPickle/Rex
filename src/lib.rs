pub mod db;
pub mod home_page;
mod initial_page;
mod interface;
mod popup_page;
pub mod tx_page;
use atty::Stream;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use db::{add_new_tx_methods, create_db, get_user_tx_methods};
use home_page::TimeData;
use initial_page::check_version;
use interface::run_app;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::process::Command;
use std::{error::Error, io, process, thread, time::Duration};
use tui::{backend::CrosstermBackend, Terminal};

/// The starting function checks for the local database location and creates a new database
/// if not existing. Also checks if the user is trying to open the app via a terminal or the binary.
/// If trying to open using the binary, tries open the relevant terminal to execute the app.
/// Lastly, starts a loop that keeps the interface running until exit command is given.
pub fn initializer(is_windows: bool, verifying_path: &str) -> Result<(), Box<dyn Error>> {

    let version_status = check_version();
    let mut new_version_available = false;

    if let Ok(a) = version_status {
        new_version_available = a;
    }

    // atty verifies whether a terminal is being used or not.
    if atty::is(Stream::Stdout) {
    } else {
        let cu_directory = std::env::current_dir()?.display().to_string();
        let output = if is_windows {
            // NOTE f*** windows. Unknown errors everywhere.
            Command::new("cmd.exe").arg("start").arg("rex").output()?
        } else {
            let linux_dir = format!("--working-directory={}", cu_directory);
            Command::new("gnome-terminal")
                .arg(linux_dir)
                .arg("--")
                .arg("./rex")
                .output()?
        };
        // TODO add checking for common and most used terminal among different O
        // Windows cmd, Konsole, other to be found out.
        if output.stderr != Vec::<u8>::new() {
            let full_text = format!(
                "Error while trying to run console/terminal. Output: \n\n{:?}",
                output
            );
            let mut open = File::create("info.txt")?;
            open.write_all(full_text.as_bytes())?;
        }
        return Ok(());
    }
    // checks the local folder and searches for data.sqlite
    let paths = fs::read_dir(".")?;
    let mut db_found = false;
    for path in paths {
        let path = path?.path().display().to_string();
        if path == verifying_path {
            db_found = true;
        }
    }
    // create a new db if not found. If there is an error, delete the failed data.sqlite file
    if db_found != true {
        let db_tx_methods = get_user_tx_methods(false);
        println!("Creating New Database. It may take some time...");
        let status = create_db("data.sqlite", db_tx_methods);
        match status {
            Ok(_) => {}
            Err(e) => {
                println!("Database creation failed. Try again. Error: {}", e);
                fs::remove_file("data.sqlite")?;
                process::exit(1);
            }
        }
    }
    loop {
        // Continue to loop to the main interface until the ending command or "break" is given
        let status = check_app(start_interface(new_version_available));
        // turn it false here so if the interface restarts, it doesn't open the popup again.
        new_version_available = false;
        if &status == "break" {
            break;
        }
    }
    Ok(())
}

/// The function to start run_app along with executing commands for switching to an alternate screen,
/// mouse capturing and passing months and year data to the function and starts the interface
fn start_interface(new_version_available: bool) -> Result<String, Box<dyn Error>> {
    // TUI magic functions starts here with multiple calls
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let months = TimeData::new(vec![
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
    ]);
    let years = TimeData::new(vec!["2022", "2023", "2024", "2025"]);

    // pass a few data to the main function and loop forever or until quit/faced with an error
    let res = run_app(&mut terminal, months, years, new_version_available)?;

    Ok(res)
}

/// The function is used to exit out of the interface
fn exit_tui_interface() -> Result<(), Box<dyn Error>> {
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    disable_raw_mode()?;
    Ok(())
}

/// The function is used to check the output which caused the tui interface to end. This
/// is used for quitting the app or do something outside of the main tui interface.
fn check_app(res: Result<String, Box<dyn Error>>) -> String {
    exit_tui_interface().expect("Error exiting the interface");

    match res {
        Err(e) => {
            println!("Error: {:?}", e);
        }
        Ok(a) => {
            // the string is gotten from run_app to process the data here.
            if &a == "Change" {
                let db_data = get_user_tx_methods(true);
                if db_data == vec!["".to_string()] {
                    println!("Operation Cancelled. Restarting in 5 seconds");
                    thread::sleep(Duration::from_millis(5000));
                } else {
                    let status = add_new_tx_methods("data.sqlite", db_data);
                    match status {
                        Ok(_) => {
                            println!("Added Transaction Methods Successfully. The app will restart in 5 seconds");
                            thread::sleep(Duration::from_millis(5000));
                        }
                        Err(e) => {
                            println!("Error while adding new transaction methods. Error: {e:?}");
                            thread::sleep(Duration::from_millis(5000));
                        }
                    }
                }
            } else if &a == "Link" {
                println!(
                    "Could not open new version link.\n\nLink: https://github.com/WaffleMixer/Rex"
                );
                return "break".to_string();
            } else {
                return "break".to_string();
            }
        }
    }
    return "".to_string();
}
