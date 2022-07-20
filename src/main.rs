extern crate rex;

// [x] Check current path for the db, create new db if necessary
// [x] create add transaction ui + editing box with inputs
// [x] func for saving & deleting txs
// [x] add creating tx button
// [x] add removing tx button
// [x] create a popup ui on Home window for commands list or if a new version is available
// [x] simple ui at the start of the program highlighting button
// [x] allow adding tx methods
// [x] change color scheme?
// [x] change balances to f32?
// [x] add date column to all_balance & all_changes
// [x] verify db cascade method working or not
// [x] add more panic handling
// [x] add save points for db commits
// [x] latest balance empty = all 0
// [x] limit add tx date between the available years
// [x] add status on add tx page
// [x] add monthly expense & income on home page
// [x] add more comments
// [x] check for empty fields if S is pressed
// [x] do not return to home if add tx is failed and show error on status section
// [x] check amount that it is not negative
// [ ] write tests
// [x] initial ui
// [x] change database location (nothing to do for now)
// [x] Need to update hotkey for the popup ui
// [x] run on terminal when using the binary
// [x] allow cancelling adding transaction method
// [ ] auto change add transaction page selected tab after enter
// [ ] add edit transaction

/// The starting function checks for the local database location and creates a new database
/// if not existing. Also checks if the user is trying to open the app via a terminal or the binary.
/// If trying to open using the binary, tries open the relevant terminal to execute the app.
/// Lastly, starts a loop that keeps the interface running until exit command is given.
fn main() {
    let mut is_windows = false;
    let mut verifying_path = "./data.sqlite";
    // change details if running on windows
    if cfg!(target_os = "windows") {
        is_windows = true;
        verifying_path = r#".\data.sqlite"#;
    }
    rex::initializer(is_windows, verifying_path).unwrap();
}
