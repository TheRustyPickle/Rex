pub const F: &str = "F: Home Page";
pub const A: &str = "A: Add Transaction Page";
pub const R: &str = "R: Chart Page";
pub const Z: &str = "Z: Summary Page";
pub const Y: &str = "Y: Activity Page";
pub const W: &str = "W: Search Page";
pub const Q: &str = "Q: Quit";
pub const H: &str = "H: Show help";
pub const V: &str = "V: Show selected transaction details";
pub const J: &str = "J: Configuration";

pub fn new_update_text(data: &[String]) -> String {
    format!(
        "New version {} is now available\n
Updates:
{}
Enter: Redirect to the new version",
        data[0], data[1]
    )
}

pub fn add_tx_help_text() -> String {
    format!(
        "This page is for adding new transactions. Following are the supported keys here. \
On Transfer transaction there will be one additional field pushing Tags to the key 7.

1: Date         Example: 2022-05-12, YYYY-MM-DD
2: TX details   Example: For Grocery, Salary
5: TX Type      Example: Income/Expense/Transfer/I/E/T
5: New TX Type  Example: Borrow/Borrow Repay/Lend/Lend Repay/b/br/l/lr
3: TX Method    Example: Cash, Bank, Card
4: Amount       Example: 1000, 100+50, b - 100
6: Tags         Example: Food, Car. Add a Comma for a new tag

S: Save the inputted data as a Transaction
Enter: Submit field and continue. Also selects the first field if nothing is selected
Esc: Stop editing field
Tab: Accept Autocompletion. Pressing again will remove the autocompleted value

Arrow Up/Down: Steps value up/down by 1 when available
Arrow Left/Right: Move cursor on input fields

C: Clear all fields/Reset all changes, including discarding a tx if was being edited
b: On amount field 'b' gets replaced with the current balance of Tx Method field
k: On amount field 'k' is considered as 1000 or a thousand
m: On amount field 'm' is considered as 1,000,000 or a million
Calculation: Amount field supports simple calculation with +, -, *, /
Tags: This field can be treated as the category of this transaction.
Empty tags field gets replaced with Unknown. Separate more than 1 tags with a comma

Example amount: 100 + b, b + b, 5 * b, 1.2k + 1m

{F}
{R}
{Z}
{Y}
{W}
{H}
{J}
{Q}
"
    )
}

pub fn chart_help_text() -> String {
    format!(
        "This page shows the movement of balances within the selected period of time
        
Following are the supported keys here

r (Lower case): Shows/Hides the top widgets for full chart view
R (Upper case): Shows/Hides the chart legends
Space: Enable/Disable tx method from the chart

Arrow Up/Down: Cycle widgets
Arrow Left/Right: Move value of the widget

{F}
{A}
{Z}
{Y}
{W}
{H}
{J}
{Q}
"
    )
}

pub fn summary_help_text() -> String {
    format!(
        "This page shows various information based on all transactions primary tag within a given period. Transfer Transaction are not shown here

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
{J}
{Q}
"
    )
}

pub fn home_help_text() -> String {
    format!(
        "This is the Home page where all txs added so far, the balances and the changes are shown

J: Configuration
E: Edit the selected transaction on the table
D: Delete the selected transaction on the table
,: Swaps the location of the selected transaction with the transaction above it
.: Swaps the location of the selected transaction with the transaction below it
{V}

Arrow Up/Down: Cycle widgets/table value
Arrow Left/Right: Move value of the widget

Swapping transaction location will only work if they are on the same date. 

{A}
{R}
{Z}
{Y}
{W}
{H}
{J}
{Q}
"
    )
}

pub fn search_help_text() -> String {
    format!(
        "This page is for searching transactions. \
            On Transfer transaction there will be one additional field pushing Tags to the key 7.

1: Date         Example: 2022-05-12, YYYY-MM-DD
2: TX details   Example: For Grocery, Salary
5: TX Type      Example: Income/Expense/Transfer/I/E/T
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
k: On amount field 'k' is considered as 1000 or a thousand
m: On amount field 'm' is considered as 1,000,000 or a million
Calculation: Amount field supports simple calculation with +, -, *, /

Example amount: 100 + b, b + b, 5 * b, 1.2k + 5m

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
{J}
{Q}
"
    )
}

pub fn activity_help_text() -> String {
    format!(
        "This page shows the activities recorded in the selected period of time. \
            The bottom widget will show affected transaction details by an activity.

Following are the supported keys here

{V}

Arrow Up/Down: Cycle widgets
Arrow Left/Right: Move value of the widget

{F}
{A}
{R}
{Z}
{W}
{H}
{J}
{Q}
"
    )
}

pub fn choice_help() -> String {
    "Arrow Up/Down: Change Choice
Enter: Select the highlighted choice
Any other key: Cancel the operation"
        .to_string()
}

pub fn reposition_help() -> String {
    "Arrow Up/Down: Change Choice
Enter: Confirm the operation when confirmation option is selected
, (comma): Move the selected transaction method up
. (dot): Move the selected transaction method down
Any other key: Cancel the operation"
        .to_string()
}
