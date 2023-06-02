use crate::page_handler::PopupState;
use crate::popup_page::create_popup;
use tui::backend::Backend;
use tui::Frame;

/// Stores data to create a new popup
pub struct PopupData<'a> {
    title: &'a str,
    x_value: u16,
    y_value: u16,
}

impl<'a> PopupData<'a> {
    pub fn new() -> Self {
        PopupData {
            title: "",
            x_value: 0,
            y_value: 0,
        }
    }

    pub fn set(&mut self, title: &'a str, x_value: u16, y_value: u16) {
        self.title = title;
        self.x_value = x_value;
        self.y_value = y_value;
    }

    pub fn create_popup<B: Backend>(&mut self, f: &mut Frame<B>, popup_type: &PopupState) {
        let status = match popup_type {
            PopupState::NewUpdate => self.get_new_update_text(),
            PopupState::HomeHelp => self.get_home_help_text(),
            PopupState::AddTxHelp => self.get_add_tx_help_text(),
            PopupState::TransferHelp => self.get_transfer_help_text(),
            PopupState::ChartHelp => self.get_chart_help_text(),
            PopupState::SummaryHelp => self.get_summary_help_text(),
            PopupState::DeleteFailed(err) => self.get_delete_failed_text(err),
            PopupState::Nothing => String::new(),
        };

        if !status.is_empty() {
            create_popup(f, self.x_value, self.y_value, self.title, status);
        }
    }

    fn get_new_update_text(&mut self) -> String {
        self.set("New Update", 30, 25);
        "There is a new version available\n
Enter: Redirect to the new version"
            .to_string()
    }

    fn get_add_tx_help_text(&mut self) -> String {
        self.set("Help", 60, 70);
        "This page is for adding new transactions. Following are the supported keys here

1: Date         Example: 2022-05-12, YYYY-MM-DD
2: TX details   Example: For Grocery, Salary
3: TX Method    Example: Cash, Bank, Card
4: Amount       Example: 1000, 100+50, b - 100
5: TX Type      Example: Income/Expense/I/E
6: TX Tags      Example: Empty, Food, Car. Add a Comma for a new tag
S: Save the inputted data as a Transaction
Enter: Submit field and continue
Esc: Stop editing filed

Arrow Up/Down: Steps value up/down by 1
Arrow Left/Right: Move cursor on input fields
C: Clear all fields
b: On amount field 'b' gets replaced with the current balance of Tx Method field
Calculation: Amount field supports simple calculation with +, -, *, /
Tags: This field can be treated as the category of this transaction.
Empty tags field gets replaced with Unknown.

Example: 100 + b, b + b, 5 * b

Other Keys:
F: Home Page
T: Add Transfer Page
R: Chart Page
Z: Summary Page
H: Show help page
Q: Quit
"
        .to_string()
    }

    fn get_transfer_help_text(&mut self) -> String {
        self.set("Help", 60, 70);
        "This page is for adding new Transfer Transaction.
Transfer refers to moving balance from one existing tx method to another.
        
Following are the supported keys here

1: Date         Example: 2022-05-12, YYYY-MM-DD
2: TX details   Example: For Grocery, Salary
3: From Method  Example: Cash, Bank, Card
4: To Method    Example: Cash, Bank, Card
5: Amount       Example: 1000, 100+50
6: TX Tags      Example: Empty, Food, Car. Add a Comma for a new tag
S: Save the inputted data as a Transaction
Enter: Submit field and continue
Esc: Stop editing filed

Arrow Up/Down: Steps value up/down by 1
Arrow Left/Right: Move cursor on input fields
C: Clear all fields
b: On amount field 'b' gets replaced with the current balance of From Method field
Calculation: Amount field supports simple calculation with +, -, *, /
Tags: This field can be treated as the category of this transaction
Empty tags field gets replaced with Unknown.

Example: 100 + b, b + b, 5 * b

Other Keys:
F: Home Page
A: Add Transaction Page
R: Chart Page
Z: Summary Page
H: Show help page
Q: Quit
"
        .to_string()
    }

    fn get_chart_help_text(&mut self) -> String {
        self.set("Help", 50, 40);
        "This page shows the movement of balances within the selected period of time
        
Following are the supported keys here

R: Hides the top widgets for full chart view
Arrow Up/Down: Cycle widgets
Arrow Left/Right: Move value of the widget

Other Keys:
F: Home Page
A: Add Transaction Page
T: Add Transfer Page
Z: Summary Page
H: Show help page
Q: Quit
"
        .to_string()
    }

    fn get_summary_help_text(&mut self) -> String {
        self.set("Help", 50, 45);
        "This page shows various information based on all transactions
and is for tracking incomes and expenses based on tags
Transfer Transaction are not shown here

Following are the supported keys here

Arrow Up/Down: Cycle widgets/table value
Arrow Left/Right: Move value of the widget
Z: Hides the top widgets for full view

Other Keys:
F: Home Page
A: Add Transaction Page
T: Add Transfer Page
R: Chart Page
H: Show help page
Q: Quit
"
        .to_string()
    }

    fn get_home_help_text(&mut self) -> String {
        self.set("Help", 50, 50);
        "This is the Home page where all txs added so far, the balances and the changes are shown
        
Following are the supported keys here

Arrow Up/Down: Cycle widgets/table value
Arrow Left/Right: Move value of the widget
J: Starts taking input to add/rename/reposition Transaction Method
E: Edit the selected transaction on the table
D: Delete the selected transaction on the table

Other Keys:
A: Add Transaction Page
T: Add Transfer Page
R: Chart Page
Z: Summary Page
H: Show help page
Q: Quit
"
        .to_string()
    }

    fn get_delete_failed_text(&mut self, err: &str) -> String {
        self.set("Delete Failed", 50, 25);
        format!("Deletion failed. Error: {}", err)
    }
}
