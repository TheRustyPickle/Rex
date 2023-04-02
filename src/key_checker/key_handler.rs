use crate::home_page::{
    AddTxTab, CurrentUi, PopupState, SelectedTab, TableData, TimeData, TransactionData, TransferTab,
};
use crate::transfer_page::TransferData;
use crate::tx_page::AddTxData;
use crossterm::event::{KeyCode, KeyEvent};
use rusqlite::Connection;

pub struct InputKeyHandler<'a> {
    pub key: KeyEvent,
    pub current_page: &'a mut CurrentUi,
    pub current_popup: &'a mut PopupState,
    pub current_tx_tab: &'a mut AddTxTab,
    pub current_transfer_tab: &'a mut TransferTab,
    add_tx_data: &'a mut AddTxData,
    transfer_data: &'a mut TransferData,
    tx_data: &'a mut TransactionData,
    table: &'a mut TableData,
    summary_table: &'a mut TableData,
    selected_tab: &'a mut SelectedTab,
    months: &'a mut TimeData,
    years: &'a mut TimeData,
    month_index: usize,
    year_index: usize,
    table_index: Option<usize>,
    total_tags: usize,
    conn: &'a Connection,
}

impl<'a> InputKeyHandler<'a> {
    pub fn new(
        key: KeyEvent,
        current_page: &'a mut CurrentUi,
        current_popup: &'a mut PopupState,
        current_tx_tab: &'a mut AddTxTab,
        current_transfer_tab: &'a mut TransferTab,
        add_tx_data: &'a mut AddTxData,
        transfer_data: &'a mut TransferData,
        tx_data: &'a mut TransactionData,
        table: &'a mut TableData,
        summary_table: &'a mut TableData,
        selected_tab: &'a mut SelectedTab,
        months: &'a mut TimeData,
        years: &'a mut TimeData,
        month_index: usize,
        year_index: usize,
        table_index: Option<usize>,
        total_tags: usize,
        conn: &'a Connection,
    ) -> InputKeyHandler<'a> {
        InputKeyHandler {
            key,
            current_page,
            current_popup,
            current_tx_tab,
            current_transfer_tab,
            add_tx_data,
            transfer_data,
            tx_data,
            table,
            summary_table,
            selected_tab,
            months,
            years,
            month_index,
            year_index,
            table_index,
            total_tags,
            conn,
        }
    }

    pub fn go_home(&mut self) {
        match self.current_page {
            CurrentUi::AddTx => {
                *self.add_tx_data = AddTxData::new();
                *self.current_tx_tab = AddTxTab::Nothing;
            }
            CurrentUi::Transfer => {
                *self.current_transfer_tab = TransferTab::Nothing;
                *self.transfer_data = TransferData::new();
            }
            _ => {}
        }
        *self.current_page = CurrentUi::Home;
    }

    pub fn go_add_tx(&mut self) {
        *self.current_page = CurrentUi::AddTx
    }

    pub fn go_transfer(&mut self) {
        *self.current_page = CurrentUi::Transfer
    }

    pub fn go_summary(&mut self) {
        *self.current_page = CurrentUi::Summary
    }

    pub fn go_chart(&mut self) {
        *self.current_page = CurrentUi::Chart
    }

    pub fn do_help_popup(&mut self) {
        *self.current_popup = PopupState::Helper
    }

    pub fn do_empty_popup(&mut self) {
        *self.current_popup = PopupState::Nothing
    }

    pub fn reload_home_table(&mut self) {
        *self.tx_data = TransactionData::new(self.conn, self.month_index, self.year_index);
        *self.table = TableData::new(self.tx_data.get_txs());
    }

    pub fn handle_update_popup(&mut self) {
        match self.key.code {
            KeyCode::Enter => {
                // If there is a new version, Enter will try to open the default browser with this link
                match open::that("https://github.com/WaffleMixer/Rex/releases/latest") {
                    Ok(_) => *self.current_popup = PopupState::Nothing,

                    // TODO handle this error
                    // if it fails for any reason, break interface and print the link
                    Err(_) => { //return Ok("Link".to_string())
                    }
                }
            }
            _ => *self.current_popup = PopupState::Nothing,
        }
    }

    pub fn add_tx(&mut self) {
        let status = self.add_tx_data.add_tx();
        if status.is_empty() {
            self.go_home();
            // we just added a new tx, select the month tab again + reload the data of balance and table widgets to get updated data
            *self.selected_tab = SelectedTab::Months;
            self.reload_home_table()
        } else {
            self.add_tx_data.add_tx_status(&status);
        }
    }

    pub fn edit_tx(&mut self) {
        if let Some(a) = self.table_index {
            let target_data = &self.tx_data.get_txs()[a];
            let target_id_num = self.tx_data.get_id_num(a);
            let tx_type = &target_data[4];

            // based on what kind of transaction is selected, passes the tx data to the struct
            // and changes the current interface
            if tx_type != "Transfer" {
                *self.add_tx_data = AddTxData::custom(
                    &target_data[0],
                    &target_data[1],
                    &target_data[2],
                    &target_data[3],
                    &target_data[4],
                    &target_data[5],
                    target_id_num,
                );
                *self.current_page = CurrentUi::AddTx;
            } else {
                let from_to = target_data[2].split(" to ").collect::<Vec<&str>>();
                let from_method = from_to[0];
                let to_method = from_to[1];

                *self.transfer_data = TransferData::custom(
                    &target_data[0],
                    &target_data[1],
                    from_method,
                    to_method,
                    &target_data[3],
                    &target_data[5],
                    target_id_num,
                );
                *self.current_page = CurrentUi::Transfer;
            }
        }
    }

    pub fn delete_tx(&mut self) {
        if let Some(index) = self.table.state.selected() {
            let status = self.tx_data.del_tx(index);
            match status {
                Ok(_) => {
                    // transaction deleted so reload the data again
                    self.reload_home_table();
                    self.table.state.select(None);
                    *self.selected_tab = SelectedTab::Months;
                }
                Err(err) => {
                    *self.current_popup = PopupState::DeleteFailed(err.to_string());
                }
            }
        }
    }

    pub fn add_transfer_tx(&mut self) {
        let status = self.transfer_data.add_tx();
        if status == *"" {
            // reload home page and switch UI
            *self.selected_tab = SelectedTab::Months;
            self.go_home();
            self.reload_home_table();
        } else {
            self.transfer_data.add_tx_status(&status);
        }
    }

    pub fn handle_number_press(&mut self) {
        match self.current_page {
            CurrentUi::AddTx => match self.key.code {
                KeyCode::Char('1') => *self.current_tx_tab = AddTxTab::Date,
                KeyCode::Char('2') => *self.current_tx_tab = AddTxTab::Details,
                KeyCode::Char('3') => *self.current_tx_tab = AddTxTab::TxMethod,
                KeyCode::Char('4') => *self.current_tx_tab = AddTxTab::Amount,
                KeyCode::Char('5') => *self.current_tx_tab = AddTxTab::TxType,
                KeyCode::Char('6') => *self.current_tx_tab = AddTxTab::Tags,
                _ => {}
            },
            CurrentUi::Transfer => match self.key.code {
                KeyCode::Char('1') => *self.current_transfer_tab = TransferTab::Date,
                KeyCode::Char('2') => *self.current_transfer_tab = TransferTab::Details,
                KeyCode::Char('3') => *self.current_transfer_tab = TransferTab::From,
                KeyCode::Char('4') => *self.current_transfer_tab = TransferTab::To,
                KeyCode::Char('5') => *self.current_transfer_tab = TransferTab::Amount,
                KeyCode::Char('6') => *self.current_transfer_tab = TransferTab::Tags,
                _ => {}
            },
            _ => {}
        }
    }

    pub fn handle_left_arrow(&mut self) {
        match self.current_page {
            CurrentUi::Home => match self.selected_tab {
                SelectedTab::Months => self.months.previous(),
                SelectedTab::Years => {
                    self.years.previous();
                    self.months.index = 0;
                }
                _ => {}
            },
            CurrentUi::AddTx => {}
            CurrentUi::Transfer => {}
            _ => {}
        }
    }

    pub fn handle_right_arrow(&mut self) {
        match self.current_page {
            CurrentUi::Home => match self.selected_tab {
                SelectedTab::Months => self.months.next(),
                SelectedTab::Years => {
                    self.years.next();
                    self.months.index = 0;
                }
                _ => {}
            },
            CurrentUi::AddTx => {}
            CurrentUi::Transfer => {}
            _ => {}
        }
    }

    pub fn handle_up_arrow(&mut self) {
        match self.current_page {
            CurrentUi::Home => self.do_home_up(),
            CurrentUi::AddTx => {}
            CurrentUi::Transfer => {}
            CurrentUi::Summary => self.do_summary_up(),
            _ => {}
        }
    }

    pub fn handle_down_arrow(&mut self) {
        match self.current_page {
            CurrentUi::Home => self.do_home_down(),
            CurrentUi::AddTx => {}
            CurrentUi::Transfer => {}
            CurrentUi::Summary => self.do_summary_down(),
            _ => {}
        }
    }

    pub fn handle_date(&mut self) {
        match self.current_page {
            CurrentUi::AddTx => self.check_add_tx_date(),
            CurrentUi::Transfer => self.check_transfer_date(),
            _ => {}
        }
    }

    pub fn handle_details(&mut self) {
        match self.current_page {
            CurrentUi::AddTx => self.check_add_tx_details(),
            CurrentUi::Transfer => self.check_transfer_details(),
            _ => {}
        }
    }

    pub fn handle_tx_method(&mut self) {
        match self.current_page {
            CurrentUi::AddTx => self.check_add_tx_method(),
            CurrentUi::Transfer => match self.current_transfer_tab {
                TransferTab::From => self.check_transfer_from(),
                TransferTab::To => self.check_transfer_to(),
                _ => {}
            },
            _ => {}
        }
    }

    pub fn handle_amount(&mut self) {
        match self.current_page {
            CurrentUi::AddTx => self.check_add_tx_amount(),
            CurrentUi::Transfer => self.check_transfer_amount(),
            _ => {}
        }
    }

    pub fn handle_tx_type(&mut self) {
        match self.current_page {
            CurrentUi::AddTx => self.check_add_tx_type(),
            _ => {}
        }
    }

    pub fn handle_tags(&mut self) {
        match self.current_page {
            CurrentUi::AddTx => self.check_add_tx_tags(),
            CurrentUi::Transfer => self.check_transfer_tags(),
            _ => {}
        }
    }
}

impl<'a> InputKeyHandler<'a> {
    fn do_home_up(&mut self) {
        match &self.selected_tab {
            SelectedTab::Table => {
                // Do not select any table rows in the table section If
                // there is no transaction
                if self.tx_data.all_tx.is_empty() {
                    *self.selected_tab = self.selected_tab.change_tab_up();
                }
                // executes when going from first table row to month widget
                else if self.table.state.selected() == Some(0) {
                    *self.selected_tab = SelectedTab::Months;
                    self.table.state.select(None);
                } else if !self.tx_data.all_tx.is_empty() {
                    self.table.previous();
                }
            }
            SelectedTab::Years => {
                // Do not select any table rows in the table section If
                // there is no transaction
                if self.tx_data.all_tx.is_empty() {
                    *self.selected_tab = self.selected_tab.change_tab_up();
                } else {
                    // Move to the selected value on table/Transaction widget
                    // to the last row if pressed up on Year section
                    self.table.state.select(Some(self.table.items.len() - 1));
                    *self.selected_tab = self.selected_tab.change_tab_up();
                    if self.tx_data.all_tx.is_empty() {
                        *self.selected_tab = self.selected_tab.change_tab_up();
                    }
                }
            }
            _ => *self.selected_tab = self.selected_tab.change_tab_up(),
        }
    }

    fn do_home_down(&mut self) {
        match &self.selected_tab {
            SelectedTab::Table => {
                // Do not proceed to the table section If
                // there is no transaction
                if self.tx_data.all_tx.is_empty() {
                    *self.selected_tab = self.selected_tab.change_tab_down();
                }
                // executes when pressed on last row of the table
                // moves to the year widget
                else if self.table.state.selected() == Some(self.table.items.len() - 1) {
                    *self.selected_tab = SelectedTab::Years;
                    self.table.state.select(None);
                } else if !self.tx_data.all_tx.is_empty() {
                    self.table.next();
                }
            }
            SelectedTab::Months => {
                if self.tx_data.all_tx.is_empty() {
                    *self.selected_tab = self.selected_tab.change_tab_up();
                } else {
                    *self.selected_tab = self.selected_tab.change_tab_down();
                    self.table.state.select(Some(0));
                };
            }
            _ => *self.selected_tab = self.selected_tab.change_tab_down(),
        }
    }

    fn do_summary_up(&mut self) {
        if self.total_tags > 0 {
            if self.summary_table.state.selected() == Some(0) {
                self.summary_table.state.select(Some(self.total_tags - 1));
            } else {
                self.summary_table.previous();
            }
        } else {
            self.summary_table.state.select(None)
        }
    }

    fn do_summary_down(&mut self) {
        if self.total_tags > 0 {
            if self.summary_table.state.selected() == Some(self.total_tags - 1) {
                self.summary_table.state.select(Some(0));
            } else {
                self.summary_table.next();
            }
        } else {
            self.summary_table.state.select(None)
        }
    }

    fn check_add_tx_date(&mut self) {
        match self.key.code {
            KeyCode::Enter => {
                let status = self.add_tx_data.check_date();
                match status {
                    Ok(a) => {
                        self.add_tx_data.add_tx_status(&a);
                        if a.contains("Accepted") || a.contains("Nothing") {
                            *self.current_tx_tab = AddTxTab::Details
                        }
                    }
                    Err(_) => self
                        .add_tx_data
                        .add_tx_status("Date: Error acquired or Date not acceptable."),
                }
            }
            KeyCode::Esc => {
                let status = self.add_tx_data.check_date();
                match status {
                    Ok(a) => {
                        self.add_tx_data.add_tx_status(&a);
                        if a.contains("Accepted") || a.contains("Nothing") {
                            *self.current_tx_tab = AddTxTab::Nothing
                        }
                    }
                    Err(_) => self
                        .add_tx_data
                        .add_tx_status("Date: Error acquired or Date not acceptable."),
                }
            }
            KeyCode::Backspace => self.add_tx_data.edit_date('a', true),
            KeyCode::Char(a) => self.add_tx_data.edit_date(a, false),
            _ => {}
        }
    }

    fn check_add_tx_method(&mut self) {
        match self.key.code {
            KeyCode::Enter => {
                let status = self.add_tx_data.check_tx_method(self.conn);

                match status {
                    Ok(a) => {
                        self.add_tx_data.add_tx_status(&a);
                        if a.contains("Accepted") || a.contains("Nothing") {
                            *self.current_tx_tab = AddTxTab::Amount
                        }
                    }
                    Err(_) => self
                        .add_tx_data
                        .add_tx_status("TX Method: Error acquired while checking."),
                }
            }
            KeyCode::Esc => {
                let status = self.add_tx_data.check_tx_method(self.conn);

                match status {
                    Ok(a) => {
                        self.add_tx_data.add_tx_status(&a);
                        if a.contains("Accepted") || a.contains("Nothing") {
                            *self.current_tx_tab = AddTxTab::Nothing
                        }
                    }
                    Err(_) => self
                        .add_tx_data
                        .add_tx_status("TX Method: Error acquired while checking."),
                }
            }
            KeyCode::Backspace => self.add_tx_data.edit_tx_method('a', true),
            KeyCode::Char(a) => self.add_tx_data.edit_tx_method(a, false),
            _ => {}
        }
    }

    fn check_add_tx_amount(&mut self) {
        match self.key.code {
            KeyCode::Enter => {
                let status = self.add_tx_data.check_amount(self.conn);
                match status {
                    Ok(a) => {
                        self.add_tx_data.add_tx_status(&a);
                        if a.contains("zero") || a.contains("determined") {
                        } else {
                            *self.current_tx_tab = AddTxTab::TxType;
                        }
                    }
                    Err(_) => self
                        .add_tx_data
                        .add_tx_status("Amount: Invalid Amount found"),
                }
            }
            KeyCode::Esc => {
                let status = self.add_tx_data.check_amount(self.conn);
                match status {
                    Ok(a) => {
                        self.add_tx_data.add_tx_status(&a);
                        if a.contains("zero") || a.contains("determined") {
                        } else {
                            *self.current_tx_tab = AddTxTab::Nothing;
                        }
                    }
                    Err(_) => self
                        .add_tx_data
                        .add_tx_status("Amount: Invalid Amount found"),
                }
            }
            KeyCode::Backspace => self.add_tx_data.edit_amount('a', true),
            KeyCode::Char(a) => self.add_tx_data.edit_amount(a, false),
            _ => {}
        }
    }

    fn check_add_tx_type(&mut self) {
        match self.key.code {
            KeyCode::Enter => {
                let status = self.add_tx_data.check_tx_type();
                match status {
                    Ok(a) => {
                        self.add_tx_data.add_tx_status(&a);
                        if a.contains("Accepted") || a.contains("Nothing") {
                            *self.current_tx_tab = AddTxTab::Tags
                        }
                    }
                    Err(_) => self
                        .add_tx_data
                        .add_tx_status("TX Type: Invalid Transaction Type Found"),
                }
            }
            KeyCode::Esc => {
                let status = self.add_tx_data.check_tx_type();
                match status {
                    Ok(a) => {
                        self.add_tx_data.add_tx_status(&a);
                        if a.contains("Accepted") || a.contains("Nothing") {
                            *self.current_tx_tab = AddTxTab::Nothing
                        }
                    }
                    Err(_) => self
                        .add_tx_data
                        .add_tx_status("TX Type: Invalid Transaction Type Found"),
                }
            }
            KeyCode::Backspace => self.add_tx_data.edit_tx_type('a', true),
            KeyCode::Char(a) => self.add_tx_data.edit_tx_type(a, false),
            _ => {}
        }
    }

    fn check_add_tx_details(&mut self) {
        match self.key.code {
            KeyCode::Enter => *self.current_tx_tab = AddTxTab::TxMethod,
            KeyCode::Esc => *self.current_tx_tab = AddTxTab::Nothing,
            KeyCode::Backspace => self.add_tx_data.edit_details('a', true),
            KeyCode::Char(a) => self.add_tx_data.edit_details(a, false),
            _ => {}
        }
    }

    fn check_add_tx_tags(&mut self) {
        match self.key.code {
            KeyCode::Enter => *self.current_tx_tab = AddTxTab::Nothing,
            KeyCode::Esc => *self.current_tx_tab = AddTxTab::Nothing,
            KeyCode::Backspace => self.add_tx_data.edit_tags('a', true),
            KeyCode::Char(a) => self.add_tx_data.edit_tags(a, false),
            _ => {}
        }
    }

    fn check_transfer_date(&mut self) {
        match self.key.code {
            KeyCode::Enter => {
                let status = self.transfer_data.check_date();
                match status {
                    Ok(a) => {
                        self.transfer_data.add_tx_status(&a);
                        if a.contains("Accepted") || a.contains("Nothing") {
                            *self.current_transfer_tab = TransferTab::Details
                        }
                    }
                    Err(_) => self
                        .transfer_data
                        .add_tx_status("Date: Error acquired or Date not acceptable."),
                }
            }
            KeyCode::Esc => {
                let status = self.transfer_data.check_date();
                match status {
                    Ok(a) => {
                        self.transfer_data.add_tx_status(&a);
                        if a.contains("Accepted") || a.contains("Nothing") {
                            *self.current_transfer_tab = TransferTab::Nothing
                        }
                    }
                    Err(_) => self
                        .transfer_data
                        .add_tx_status("Date: Error acquired or Date not acceptable."),
                }
            }
            KeyCode::Backspace => self.transfer_data.edit_date('a', true),
            KeyCode::Char(a) => self.transfer_data.edit_date(a, false),
            _ => {}
        }
    }

    fn check_transfer_details(&mut self) {
        match self.key.code {
            KeyCode::Enter => *self.current_transfer_tab = TransferTab::From,
            KeyCode::Esc => *self.current_transfer_tab = TransferTab::Nothing,
            KeyCode::Backspace => self.transfer_data.edit_details('a', true),
            KeyCode::Char(a) => self.transfer_data.edit_details(a, false),
            _ => {}
        }
    }

    fn check_transfer_from(&mut self) {
        match self.key.code {
            KeyCode::Enter => {
                let status = self.transfer_data.check_from(self.conn);

                match status {
                    Ok(a) => {
                        self.transfer_data.add_tx_status(&a);
                        if a.contains("Accepted") || a.contains("Nothing") {
                            *self.current_transfer_tab = TransferTab::To
                        }
                    }
                    Err(_) => self
                        .transfer_data
                        .add_tx_status("TX Method: Error acquired while checking."),
                }
            }
            KeyCode::Esc => {
                let status = self.transfer_data.check_from(self.conn);

                match status {
                    Ok(a) => {
                        self.transfer_data.add_tx_status(&a);
                        if a.contains("Accepted") || a.contains("Nothing") {
                            *self.current_transfer_tab = TransferTab::Nothing
                        }
                    }
                    Err(_) => self
                        .transfer_data
                        .add_tx_status("TX Method: Error acquired while checking."),
                }
            }
            KeyCode::Backspace => self.transfer_data.edit_from('a', true),
            KeyCode::Char(a) => self.transfer_data.edit_from(a, false),
            _ => {}
        }
    }

    fn check_transfer_to(&mut self) {
        match self.key.code {
            KeyCode::Enter => {
                let status = self.transfer_data.check_to(self.conn);

                match status {
                    Ok(a) => {
                        self.transfer_data.add_tx_status(&a);
                        if a.contains("Accepted") || a.contains("Nothing") {
                            *self.current_transfer_tab = TransferTab::Amount
                        }
                    }
                    Err(_) => self
                        .transfer_data
                        .add_tx_status("TX Method: Error acquired while checking."),
                }
            }
            KeyCode::Esc => {
                let status = self.transfer_data.check_to(self.conn);

                match status {
                    Ok(a) => {
                        self.transfer_data.add_tx_status(&a);
                        if a.contains("Accepted") || a.contains("Nothing") {
                            *self.current_transfer_tab = TransferTab::Nothing
                        }
                    }
                    Err(_) => self
                        .transfer_data
                        .add_tx_status("TX Method: Error acquired while checking."),
                }
            }
            KeyCode::Backspace => self.transfer_data.edit_to('a', true),
            KeyCode::Char(a) => self.transfer_data.edit_to(a, false),
            _ => {}
        }
    }

    fn check_transfer_amount(&mut self) {
        match self.key.code {
            KeyCode::Enter => {
                let status = self.transfer_data.check_amount();
                match status {
                    Ok(a) => {
                        self.transfer_data.add_tx_status(&a);
                        if a.contains("zero") {
                        } else {
                            *self.current_transfer_tab = TransferTab::Tags;
                        }
                    }
                    Err(_) => self
                        .transfer_data
                        .add_tx_status("Amount: Invalid Amount found"),
                }
            }
            KeyCode::Esc => {
                let status = self.transfer_data.check_amount();
                match status {
                    Ok(a) => {
                        self.transfer_data.add_tx_status(&a);
                        if a.contains("zero") {
                        } else {
                            *self.current_transfer_tab = TransferTab::Nothing;
                        }
                    }
                    Err(_) => self
                        .transfer_data
                        .add_tx_status("Amount: Invalid Amount found"),
                }
            }
            KeyCode::Backspace => self.transfer_data.edit_amount('a', true),
            KeyCode::Char(a) => self.transfer_data.edit_amount(a, false),
            _ => {}
        }
    }

    fn check_transfer_tags(&mut self) {
        match self.key.code {
            KeyCode::Enter => *self.current_transfer_tab = TransferTab::Nothing,
            KeyCode::Esc => *self.current_transfer_tab = TransferTab::Nothing,
            KeyCode::Backspace => self.transfer_data.edit_tags('a', true),
            KeyCode::Char(a) => self.transfer_data.edit_tags(a, false),
            _ => {}
        }
    }
}
