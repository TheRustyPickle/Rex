use crate::page_handler::PopupState;
use crate::popup_page::create_popup;
use tui::backend::Backend;
use tui::Frame;

// TODO turn data into a struct for easier access
// TODO give specific help page for each tab?
pub fn add_popup<B: Backend>(f: &mut Frame<B>, popup_type: &PopupState) {
    let mut data = Vec::new();

    match popup_type {
        PopupState::NewUpdate => {
            data.push("New Update".to_string());
            data.push(
                "There is a new version available\n
'Enter' : Redirect to the new version\nPress Any Key to dismiss"
                    .to_string(),
            );
            data.push("50".to_string());
            data.push("30".to_string());
        }
        PopupState::Helper => {
            data.push("Help".to_string());
            data.push(
                "Arrow Key : Navigate
A: Add Transaction Page
T: Add Transfer Page
R: Balance Chart (Follows your selected year)
Z: Get Transaction Summary
F: Home Page
D: Delete selected Transaction (Home Page)
J: Add new Transaction Methods (Home Page)
E: Edit Selected Transaction (Home Page)
H: Open Hotkey Help
Q: Quit
                
Add Transaction/Transfer Page:
1: Edit Date           4: Edit Amount/To Method
2: Edit TX details     5: Edit TX Type/Amount
3: Edit TX/From Method 6: Edit Tags  

S: Save inputted data as a Transaction
Enter: Submit a field and continue
Esc: Stop editing a filed\n
Press Any Key To Dismiss"
                    .to_string(),
            );
            data.push("50".to_string());
            data.push("65".to_string());
        }
        PopupState::DeleteFailed(err) => {
            data.push(format!("Deletion failed. Error: {}", err));
            data.push(
                "Error while deleting the transaction\n\nPress Any Key to dismiss".to_string(),
            );
            data.push("40".to_string());
            data.push("25".to_string());
        }
        _ => {}
    }

    if !data.is_empty() {
        create_popup(f, &data);
    }
}
