mod table_data;
mod table_ui;
mod ui_data_state;

pub use table_data::TransactionData;
pub use table_ui::ui;
pub use ui_data_state::{
    CurrentUi, PopupState, SelectedTab, TableData, TimeData, TransferTab, TxTab,
};
