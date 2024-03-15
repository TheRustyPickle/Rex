use ratatui::Frame;

use crate::page_handler::{DeletionStatus, PopupState};
use crate::popup_page::{create_deletion_popup, create_popup};

pub const F: &str = "F: Home Page";
pub const A: &str = "A: Add Transaction Page";
pub const R: &str = "R: Chart Page";
pub const Z: &str = "Z: Summary Page";
pub const Y: &str = "Y: Activity Page";
pub const W: &str = "W: Search Page";
pub const Q: &str = "Q: Quit";
pub const H: &str = "H: Show help";

/// Stores data to create a new popup
pub struct PopupData<'a> {
    title: &'a str,
    x_value: u16,
    y_value: u16,
}

impl<'a> PopupData<'a> {
    #[cfg(not(tarpaulin_include))]
    pub fn new() -> Self {
        PopupData {
            title: "",
            x_value: 0,
            y_value: 0,
        }
    }

    #[cfg(not(tarpaulin_include))]
    pub fn set(&mut self, title: &'a str, x_value: u16, y_value: u16) {
        self.title = title;
        self.x_value = x_value;
        self.y_value = y_value;
    }

    #[cfg(not(tarpaulin_include))]
    pub fn create_popup(
        &mut self,
        f: &mut Frame,
        popup_type: &PopupState,
        deletion_status: &DeletionStatus,
    ) {
        let status = match popup_type {
            PopupState::NewUpdate(data) => self.get_new_update_text(data),
            PopupState::HomeHelp => self.get_home_help_text(),
            PopupState::AddTxHelp => self.get_add_tx_help_text(),
            PopupState::ChartHelp => self.get_chart_help_text(),
            PopupState::SummaryHelp => self.get_summary_help_text(),
            PopupState::DeleteFailed(err) => self.get_delete_failed_text(err),
            PopupState::SearchHelp => self.get_search_help_text(),
            PopupState::ActivityHelp => self.get_activity_help_text(),
            PopupState::Nothing | PopupState::TxDeletion => String::new(),
        };

        if let PopupState::TxDeletion = popup_type {
            create_deletion_popup(f, deletion_status);
        } else if !status.is_empty() {
            create_popup(f, self.x_value, self.y_value, self.title, &status);
        }
    }

    #[cfg(not(tarpaulin_include))]
    fn get_new_update_text(&mut self, data: &[String]) -> String {
        let update_data_len = data[1].split('\n').collect::<Vec<&str>>().len() * 2;
        self.set("New Update", 50, 25 + update_data_len as u16);
        format!(
            "New version {} is now available\n
Updates:
{}
Enter: Redirect to the new version",
            data[0], data[1]
        )
    }

    #[cfg(not(tarpaulin_include))]
    fn get_add_tx_help_text(&mut self) -> String {
        self.set("Help", 80, 90);
        format!(
            "This page is for adding new transactions. Following are the supported keys here. \
On Transfer transaction there will be one additional field pushing Tags to the key 7. 

1: Date         Example: 2022-05-12, YYYY-MM-DD
2: TX details   Example: For Grocery, Salary
5: TX Type      Example: Income/Expense/I/E
3: TX Method    Example: Cash, Bank, Card
4: Amount       Example: 1000, 100+50, b - 100
6: Tags         Example: Food, Car. Add a Comma for a new tag

S: Save the inputted data as a Transaction
Enter: Submit field and continue. Also selects the first field if nothing is selected
Esc: Stop editing field
Tab: Accept Autocompletion. Pressing again will remove the autocompleted value

Arrow Up/Down: Steps value up/down by 1 when available
Arrow Left/Right: Move cursor on input fields

C: Clear all fields
b: On amount field 'b' gets replaced with the current balance of Tx Method field
Calculation: Amount field supports simple calculation with +, -, *, /
Tags: This field can be treated as the category of this transaction.
Empty tags field gets replaced with Unknown. Separate more than 1 tags with a comma

Example amount: 100 + b, b + b, 5 * b

{F}
{R}
{Z}
{Y}
{W}
{H}
{Q}
"
        )
    }

    #[cfg(not(tarpaulin_include))]
    fn get_chart_help_text(&mut self) -> String {
        self.set("Help", 60, 50);
        format!(
            "This page shows the movement of balances within the selected period of time
        
Following are the supported keys here

R: Hides the top widgets for full chart view

Arrow Up/Down: Cycle widgets
Arrow Left/Right: Move value of the widget

{F}
{A}
{Z}
{Y}
{W}
{H}
{Q}
"
        )
    }

    #[cfg(not(tarpaulin_include))]
    fn get_summary_help_text(&mut self) -> String {
        self.set("Help", 50, 60);
        format!(
            "This page shows various information based on all transactions \
            and is for tracking incomes and expenses based on tags \
            Transfer Transaction are not shown here

Following are the supported keys here

X: Sorts table by Tag, Total Income or Total Expense
Z: Hides the top widgets for full view

Arrow Up/Down: Cycle widgets/table value
Arrow Left/Right: Move value of the widget

{F}
{A}
{R}
{Y}
{W}
{H}
{Q}
"
        )
    }

    #[cfg(not(tarpaulin_include))]
    fn get_home_help_text(&mut self) -> String {
        self.set("Help", 70, 70);
        format!("This is the Home page where all txs added so far, the balances and the changes are shown

J: Take user input for various actions
E: Edit the selected transaction on the table
D: Delete the selected transaction on the table
,: Swaps the location of the selected transaction with the transaction above it
.: Swaps the location of the selected transaction with the transaction below it

Arrow Up/Down: Cycle widgets/table value
Arrow Left/Right: Move value of the widget

Swapping transaction location will only work if they are on the same date. 

{A}
{R}
{Z}
{Y}
{W}
{H}
{Q}
")
    }

    #[cfg(not(tarpaulin_include))]
    fn get_delete_failed_text(&mut self, err: &str) -> String {
        self.set("Delete Failed", 50, 25);
        err.to_string()
    }

    #[cfg(not(tarpaulin_include))]
    fn get_search_help_text(&mut self) -> String {
        self.set("Help", 70, 100);
        format!(
            "This page is for searching transactions. \
            On Transfer transaction there will be one additional field pushing Tags to the key 7.

1: Date         Example: 2022-05-12, YYYY-MM-DD
2: TX details   Example: For Grocery, Salary
5: TX Type      Example: Income/Expense/I/E
3: TX Method    Example: Cash, Bank, Card
4: Amount       Example: 1000, 100+50, b - 100
6: Tags         Example: Food, Car. Add a Comma for a new tag

Fields: Minimum 1 field must be filled to search for transactions. \
                    Fill up multiple fields for better accuracy

S: Search for transactions with the given data
X: Cycle date type for searching with exact date, month based or year based
Enter: Submit field and continue. Also selects the first field if nothing is selected
Esc: Stop editing field
Tab: Accept Autocompletion. Pressing again will remove the autocompleted value

Arrow Up/Down: Steps value up/down by 1
Arrow Left/Right: Move cursor on input fields
C: Clear all fields
b: On amount field 'b' gets replaced with the current balance of Tx Method field
Calculation: Amount field supports simple calculation with +, -, *, /

Example amount: 100 + b, b + b, 5 * b

Details Field: If details field is filled up, it will try to find transactions \
                that matches the given input. It doesn't have to be an exact match
Tags Field: If tags field is filled up with more than 1 tags, it will match all transactions \
                that has any one of the tags.
Amount Field: Amount field supports '>' '<' '>=' '<=' highlighting amount \
                Bigger than, Smaller than, Bigger or equal, Smaller or equal respectively. \
                No symbol will mean exact amount match. 

Example amount : <1000, >=10000

{F}
{A}
{R}
{Z}
{Y}
{H}
{Q}
"
        )
    }

    fn get_activity_help_text(&mut self) -> String {
        self.set("Help", 60, 50);
        format!(
            "This page shows the activities recorded in the selected period of time. \
            The bottom widget will show affected transaction details by an activity.

Following are the supported keys here

Arrow Up/Down: Cycle widgets
Arrow Left/Right: Move value of the widget

{F}
{A}
{R}
{Z}
{W}
{H}
{Q}
"
        )
    }
}
