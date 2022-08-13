use crate::popup_page::create_popup;
use tui::{backend::Backend, Frame};

pub fn add_popup<B: Backend>(f: &mut Frame<B>, popup_num: usize) {
    let mut data = Vec::new();
    if popup_num == 0 {
        data.push("New Update".to_string());
        data.push(
            "There is a new version available\n
'Enter' : Redirect to the new version\nPress Any Key to dismiss"
                .to_string(),
        );
        data.push("50".to_string());
        data.push("30".to_string());
    } else if popup_num == 1 {
        data.push("Help".to_string());
        data.push(
            "'Arrow Key' : Navigate
'A' : Add Transaction Page
'F' : Home Page
'D' : Delete selected Transaction (Home Page)
'J' : Add new Transaction Methods (Home Page)
'E' : Edit Selected Transaction (Home Page)
'H' : Open Hotkey Help
'Q' : Quit

Add Transaction Page:
'1' : Edit Date          '4' : Edit Amount
'2' : Edit TX details    '3' : Edit TX Method
'5' : Edit TX Type
'S' : Save the data as a Transaction
'Enter' : Submit field and continue
'Esc' : Stop editing filed\n
Press Any Key to dismiss"
                .to_string(),
        );
        data.push("50".to_string());
        data.push("65".to_string());
    } else {
        data.push("Delete Error".to_string());
        data.push(format!(
            "Error while deleting the transaction\nPress Any Key to dismiss"
        ));
        data.push("50".to_string());
        data.push("30".to_string());
    }
    create_popup(f, &data);
}
