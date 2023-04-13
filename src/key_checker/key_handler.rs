use crate::chart_page::ChartData;
use crate::home_page::TransactionData;
use crate::outputs::{HandlingOutput, VerifyingOutput};
use crate::page_handler::{
    AddTxTab, ChartTab, CurrentUi, HomeTab, IndexedData, PopupState, SummaryTab, TableData,
    TransferTab,
};
use crate::summary_page::SummaryData;
use crate::tx_handler::TxData;
use crossterm::event::{KeyCode, KeyEvent};
use rusqlite::Connection;

pub struct InputKeyHandler<'a> {
    pub key: KeyEvent,
    pub page: &'a mut CurrentUi,
    pub popup: &'a mut PopupState,
    pub tx_tab: &'a mut AddTxTab,
    pub transfer_tab: &'a mut TransferTab,
    chart_tab: &'a mut ChartTab,
    summary_tab: &'a mut SummaryTab,
    home_tab: &'a mut HomeTab,
    add_tx_data: &'a mut TxData,
    transfer_data: &'a mut TxData,
    all_tx_data: &'a mut TransactionData,
    chart_data: &'a mut ChartData,
    summary_data: &'a mut SummaryData,
    table: &'a mut TableData,
    summary_table: &'a mut TableData,
    home_months: &'a mut IndexedData,
    home_years: &'a mut IndexedData,
    chart_months: &'a mut IndexedData,
    chart_years: &'a mut IndexedData,
    chart_modes: &'a mut IndexedData,
    summary_months: &'a mut IndexedData,
    summary_years: &'a mut IndexedData,
    summary_modes: &'a mut IndexedData,
    total_tags: usize,
    chart_index: &'a mut Option<usize>,
    chart_hidden_mode: &'a mut bool,
    conn: &'a Connection,
}

impl<'a> InputKeyHandler<'a> {
    pub fn new(
        key: KeyEvent,
        page: &'a mut CurrentUi,
        popup: &'a mut PopupState,
        tx_tab: &'a mut AddTxTab,
        transfer_tab: &'a mut TransferTab,
        chart_tab: &'a mut ChartTab,
        summary_tab: &'a mut SummaryTab,
        home_tab: &'a mut HomeTab,
        add_tx_data: &'a mut TxData,
        transfer_data: &'a mut TxData,
        all_tx_data: &'a mut TransactionData,
        chart_data: &'a mut ChartData,
        summary_data: &'a mut SummaryData,
        table: &'a mut TableData,
        summary_table: &'a mut TableData,
        home_months: &'a mut IndexedData,
        home_years: &'a mut IndexedData,
        chart_months: &'a mut IndexedData,
        chart_years: &'a mut IndexedData,
        chart_modes: &'a mut IndexedData,
        summary_months: &'a mut IndexedData,
        summary_years: &'a mut IndexedData,
        summary_modes: &'a mut IndexedData,
        chart_index: &'a mut Option<usize>,
        chart_hidden_mode: &'a mut bool,
        conn: &'a Connection,
    ) -> InputKeyHandler<'a> {
        let total_tags = summary_data.get_table_data().len();
        InputKeyHandler {
            key,
            page,
            popup,
            tx_tab,
            transfer_tab,
            chart_tab,
            summary_tab,
            home_tab,
            add_tx_data,
            transfer_data,
            all_tx_data,
            chart_data,
            summary_data,
            table,
            summary_table,
            home_months,
            home_years,
            chart_months,
            chart_years,
            chart_modes,
            summary_months,
            summary_years,
            summary_modes,
            total_tags,
            chart_index,
            chart_hidden_mode,
            conn,
        }
    }

    pub fn go_home_reset(&mut self) {
        match self.page {
            CurrentUi::AddTx => {
                *self.add_tx_data = TxData::new();
                *self.tx_tab = AddTxTab::Nothing;
            }
            CurrentUi::Transfer => {
                *self.transfer_tab = TransferTab::Nothing;
                *self.transfer_data = TxData::new();
            }
            _ => {}
        }
        *self.page = CurrentUi::Home;
    }

    pub fn go_home(&mut self) {
        *self.page = CurrentUi::Home;
    }

    pub fn go_add_tx(&mut self) {
        *self.page = CurrentUi::AddTx
    }

    pub fn go_transfer(&mut self) {
        *self.page = CurrentUi::Transfer
    }

    pub fn go_summary(&mut self) {
        *self.page = CurrentUi::Summary;
        self.summary_modes.set_index_zero();
        self.summary_months.set_index_zero();
        self.summary_years.set_index_zero();
        *self.summary_tab = SummaryTab::ModeSelection;
        self.reload_summary();
    }

    pub fn go_chart(&mut self) {
        *self.page = CurrentUi::Chart;
        self.chart_modes.set_index_zero();
        self.chart_years.set_index_zero();
        self.chart_months.set_index_zero();
        *self.chart_tab = ChartTab::ModeSelection;
        *self.chart_hidden_mode = false;
        self.reload_chart();
    }

    pub fn do_help_popup(&mut self) {
        *self.popup = PopupState::Helper
    }

    pub fn do_empty_popup(&mut self) {
        *self.popup = PopupState::Nothing
    }

    pub fn do_hidden_mode(&mut self) {
        *self.chart_hidden_mode = !*self.chart_hidden_mode;
    }

    pub fn handle_update_popup(&mut self) -> Result<(), HandlingOutput> {
        match self.key.code {
            KeyCode::Enter => {
                // If there is a new version, Enter will try to open the default browser with this link
                Ok(
                    open::that("https://github.com/WaffleMixer/Rex/releases/latest")
                        .map_err(|_| HandlingOutput::PrintNewUpdate)?,
                )
            }
            _ => Ok(*self.popup = PopupState::Nothing),
        }
    }

    pub fn add_tx(&mut self) {
        let status = self.add_tx_data.add_tx();
        if status.is_empty() {
            self.go_home_reset();
            // we just added a new tx, select the month tab again + reload the data of balance and table widgets to get updated data
            *self.home_tab = HomeTab::Months;
            self.reload_home_table()
        } else {
            self.add_tx_data.add_tx_status(status);
        }
    }

    pub fn edit_tx(&mut self) {
        if let Some(a) = self.table.state.selected() {
            let target_data = &self.all_tx_data.get_txs()[a];
            let target_id_num = self.all_tx_data.get_id_num(a);
            let tx_type = &target_data[4];

            // based on what kind of transaction is selected, passes the tx data to the struct
            // and changes the current interface
            if tx_type != "Transfer" {
                *self.add_tx_data = TxData::custom(
                    &target_data[0],
                    &target_data[1],
                    &target_data[2],
                    "",
                    &target_data[3],
                    &target_data[4],
                    &target_data[5],
                    target_id_num,
                );
                *self.page = CurrentUi::AddTx;
            } else {
                let splitted_method = target_data[2].split(" to ").collect::<Vec<&str>>();
                let from_method = splitted_method[0];
                let to_method = splitted_method[1];

                *self.transfer_data = TxData::custom(
                    &target_data[0],
                    &target_data[1],
                    from_method,
                    to_method,
                    &target_data[3],
                    "Transfer",
                    &target_data[5],
                    target_id_num,
                );
                *self.page = CurrentUi::Transfer;
            }
        }
    }

    pub fn delete_tx(&mut self) {
        if let Some(index) = self.table.state.selected() {
            let status = self.all_tx_data.del_tx(index);
            match status {
                Ok(_) => {
                    // transaction deleted so reload the data again
                    self.reload_home_table();
                    self.table.state.select(None);
                    *self.home_tab = HomeTab::Months;
                }
                Err(err) => {
                    *self.popup = PopupState::DeleteFailed(err.to_string());
                }
            }
        }
    }

    pub fn add_transfer_tx(&mut self) {
        let status = self.transfer_data.add_tx();
        if status == *"" {
            // reload home page and switch UI
            *self.home_tab = HomeTab::Months;
            self.go_home_reset();
            self.reload_home_table();
        } else {
            self.transfer_data.add_tx_status(status);
        }
    }

    pub fn handle_number_press(&mut self) {
        match self.page {
            CurrentUi::AddTx => match self.key.code {
                KeyCode::Char('1') => *self.tx_tab = AddTxTab::Date,
                KeyCode::Char('2') => *self.tx_tab = AddTxTab::Details,
                KeyCode::Char('3') => *self.tx_tab = AddTxTab::TxMethod,
                KeyCode::Char('4') => *self.tx_tab = AddTxTab::Amount,
                KeyCode::Char('5') => *self.tx_tab = AddTxTab::TxType,
                KeyCode::Char('6') => *self.tx_tab = AddTxTab::Tags,
                _ => {}
            },
            CurrentUi::Transfer => match self.key.code {
                KeyCode::Char('1') => *self.transfer_tab = TransferTab::Date,
                KeyCode::Char('2') => *self.transfer_tab = TransferTab::Details,
                KeyCode::Char('3') => *self.transfer_tab = TransferTab::From,
                KeyCode::Char('4') => *self.transfer_tab = TransferTab::To,
                KeyCode::Char('5') => *self.transfer_tab = TransferTab::Amount,
                KeyCode::Char('6') => *self.transfer_tab = TransferTab::Tags,
                _ => {}
            },
            _ => {}
        }
    }

    pub fn handle_left_arrow(&mut self) {
        match self.page {
            CurrentUi::Home => match self.home_tab {
                HomeTab::Months => {
                    self.home_months.previous();
                    self.reload_home_table();
                }
                HomeTab::Years => {
                    self.home_years.previous();
                    self.home_months.set_index_zero();
                    self.reload_home_table();
                }
                _ => {}
            },
            CurrentUi::AddTx => {}
            CurrentUi::Transfer => {}
            CurrentUi::Chart => {
                if !*self.chart_hidden_mode {
                    match self.chart_tab {
                        ChartTab::ModeSelection => {
                            self.chart_modes.previous();
                            self.reload_chart();
                        }
                        ChartTab::Years => {
                            self.chart_years.previous();
                            self.chart_months.set_index_zero();
                            self.reload_chart();
                        }
                        ChartTab::Months => {
                            self.chart_months.previous();
                            self.reload_chart();
                        }
                    }
                }
            }
            CurrentUi::Summary => match self.summary_tab {
                SummaryTab::ModeSelection => {
                    self.summary_modes.previous();
                    self.reload_summary();
                }
                SummaryTab::Years => {
                    self.summary_months.set_index_zero();
                    self.summary_years.previous();
                    self.reload_summary();
                }
                SummaryTab::Months => {
                    self.summary_months.previous();
                    self.reload_summary();
                }
                _ => {}
            },
            _ => {}
        }
    }

    pub fn handle_right_arrow(&mut self) {
        match self.page {
            CurrentUi::Home => match self.home_tab {
                HomeTab::Months => {
                    self.home_months.next();
                    self.reload_home_table();
                }
                HomeTab::Years => {
                    self.home_years.next();
                    self.home_months.set_index_zero();
                    self.reload_home_table();
                }
                _ => {}
            },
            CurrentUi::AddTx => {}
            CurrentUi::Transfer => {}
            CurrentUi::Chart => {
                if !*self.chart_hidden_mode {
                    match self.chart_tab {
                        ChartTab::ModeSelection => {
                            self.chart_modes.next();
                            self.reload_chart();
                        }
                        ChartTab::Years => {
                            self.chart_years.next();
                            self.chart_months.set_index_zero();
                            self.reload_chart();
                        }
                        ChartTab::Months => {
                            self.chart_months.next();
                            self.reload_chart();
                        }
                    }
                }
            }
            CurrentUi::Summary => match self.summary_tab {
                SummaryTab::ModeSelection => {
                    self.summary_modes.next();
                    self.reload_summary();
                }
                SummaryTab::Years => {
                    self.summary_months.set_index_zero();
                    self.summary_years.next();
                    self.reload_summary();
                }
                SummaryTab::Months => {
                    self.summary_months.next();
                    self.reload_summary();
                }
                _ => {}
            },
            _ => {}
        }
    }

    pub fn handle_up_arrow(&mut self) {
        match self.page {
            CurrentUi::Home => self.do_home_up(),
            CurrentUi::AddTx => {}
            CurrentUi::Transfer => {}
            CurrentUi::Summary => self.do_summary_up(),
            CurrentUi::Chart => self.do_chart_up(),
            _ => {}
        }
    }

    pub fn handle_down_arrow(&mut self) {
        match self.page {
            CurrentUi::Home => self.do_home_down(),
            CurrentUi::AddTx => {}
            CurrentUi::Transfer => {}
            CurrentUi::Summary => self.do_summary_down(),
            CurrentUi::Chart => self.do_chart_down(),
            _ => {}
        }
    }

    pub fn handle_date(&mut self) {
        match self.page {
            CurrentUi::AddTx => self.check_add_tx_date(),
            CurrentUi::Transfer => self.check_transfer_date(),
            _ => {}
        }
    }

    pub fn handle_details(&mut self) {
        match self.page {
            CurrentUi::AddTx => self.check_add_tx_details(),
            CurrentUi::Transfer => self.check_transfer_details(),
            _ => {}
        }
    }

    pub fn handle_tx_method(&mut self) {
        match self.page {
            CurrentUi::AddTx => self.check_add_tx_method(),
            CurrentUi::Transfer => match self.transfer_tab {
                TransferTab::From => self.check_transfer_from(),
                TransferTab::To => self.check_transfer_to(),
                _ => {}
            },
            _ => {}
        }
    }

    pub fn handle_amount(&mut self) {
        match self.page {
            CurrentUi::AddTx => self.check_add_tx_amount(),
            CurrentUi::Transfer => self.check_transfer_amount(),
            _ => {}
        }
    }

    pub fn handle_tx_type(&mut self) {
        match self.page {
            CurrentUi::AddTx => self.check_add_tx_type(),
            _ => {}
        }
    }

    pub fn handle_tags(&mut self) {
        match self.page {
            CurrentUi::AddTx => self.check_add_tx_tags(),
            CurrentUi::Transfer => self.check_transfer_tags(),
            _ => {}
        }
    }
}

impl<'a> InputKeyHandler<'a> {
    fn do_home_up(&mut self) {
        match &self.home_tab {
            HomeTab::Table => {
                // Do not select any table rows in the table section If
                // there is no transaction
                if self.all_tx_data.all_tx.is_empty() {
                    *self.home_tab = self.home_tab.change_tab_up();
                }
                // executes when going from first table row to month widget
                else if self.table.state.selected() == Some(0) {
                    *self.home_tab = HomeTab::Months;
                    self.table.state.select(None);
                } else if !self.all_tx_data.all_tx.is_empty() {
                    self.table.previous();
                }
            }
            HomeTab::Years => {
                // Do not select any table rows in the table section If
                // there is no transaction
                if self.all_tx_data.all_tx.is_empty() {
                    *self.home_tab = self.home_tab.change_tab_up();
                } else {
                    // Move to the selected value on table/Transaction widget
                    // to the last row if pressed up on Year section
                    self.table.state.select(Some(self.table.items.len() - 1));
                    *self.home_tab = self.home_tab.change_tab_up();
                    if self.all_tx_data.all_tx.is_empty() {
                        *self.home_tab = self.home_tab.change_tab_up();
                    }
                }
            }
            _ => *self.home_tab = self.home_tab.change_tab_up(),
        }
    }

    fn do_home_down(&mut self) {
        match &self.home_tab {
            HomeTab::Table => {
                // Do not proceed to the table section If
                // there is no transaction
                if self.all_tx_data.all_tx.is_empty() {
                    *self.home_tab = self.home_tab.change_tab_down();
                }
                // executes when pressed on last row of the table
                // moves to the year widget
                else if self.table.state.selected() == Some(self.table.items.len() - 1) {
                    *self.home_tab = HomeTab::Years;
                    self.table.state.select(None);
                } else if !self.all_tx_data.all_tx.is_empty() {
                    self.table.next();
                }
            }
            HomeTab::Months => {
                if self.all_tx_data.all_tx.is_empty() {
                    *self.home_tab = self.home_tab.change_tab_up();
                } else {
                    *self.home_tab = self.home_tab.change_tab_down();
                    self.table.state.select(Some(0));
                };
            }
            _ => *self.home_tab = self.home_tab.change_tab_down(),
        }
    }

    fn do_summary_up(&mut self) {
        match self.summary_modes.index {
            0 => match self.summary_tab {
                SummaryTab::Table => {
                    if self.summary_table.state.selected() == Some(0) {
                        *self.summary_tab = self.summary_tab.change_tab_up_monthly();
                    } else {
                        self.summary_table.previous()
                    }
                }
                SummaryTab::ModeSelection => {
                    if self.total_tags > 0 {
                        self.summary_table.state.select(Some(self.total_tags - 1));
                        *self.summary_tab = self.summary_tab.change_tab_up_monthly();
                    } else {
                        *self.summary_tab = self.summary_tab.change_tab_up_monthly();
                        *self.summary_tab = self.summary_tab.change_tab_up_monthly();
                        self.summary_table.state.select(None)
                    }
                }
                _ => *self.summary_tab = self.summary_tab.change_tab_up_monthly(),
            },
            1 => match self.summary_tab {
                SummaryTab::Table => {
                    if self.summary_table.state.selected() == Some(0) {
                        *self.summary_tab = self.summary_tab.change_tab_up_yearly();
                    } else {
                        self.summary_table.previous()
                    }
                }
                SummaryTab::ModeSelection => {
                    if self.total_tags > 0 {
                        self.summary_table.state.select(Some(self.total_tags - 1));
                        *self.summary_tab = self.summary_tab.change_tab_up_yearly()
                    } else {
                        *self.summary_tab = self.summary_tab.change_tab_up_yearly();
                        *self.summary_tab = self.summary_tab.change_tab_up_yearly();
                        self.summary_table.state.select(None)
                    }
                }
                _ => *self.summary_tab = self.summary_tab.change_tab_up_yearly(),
            },
            2 => match self.summary_tab {
                SummaryTab::Table => {
                    if self.summary_table.state.selected() == Some(0) {
                        *self.summary_tab = self.summary_tab.change_tab_up_all_time();
                    } else {
                        self.summary_table.previous()
                    }
                }
                SummaryTab::ModeSelection => {
                    if self.total_tags > 0 {
                        self.summary_table.state.select(Some(self.total_tags - 1));
                        *self.summary_tab = self.summary_tab.change_tab_up_all_time();
                    } else {
                        *self.summary_tab = self.summary_tab.change_tab_up_all_time();
                        *self.summary_tab = self.summary_tab.change_tab_up_all_time();
                        self.summary_table.state.select(None)
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn do_summary_down(&mut self) {
        match self.summary_modes.index {
            0 => match self.summary_tab {
                SummaryTab::Table => {
                    if self.summary_table.state.selected() == Some(self.total_tags - 1) {
                        *self.summary_tab = self.summary_tab.change_tab_down_monthly();
                    } else {
                        self.summary_table.next()
                    }
                }
                SummaryTab::Months => {
                    if self.total_tags > 0 {
                        self.summary_table.state.select(Some(0));
                        *self.summary_tab = self.summary_tab.change_tab_down_monthly();
                    } else {
                        *self.summary_tab = self.summary_tab.change_tab_down_monthly();
                        *self.summary_tab = self.summary_tab.change_tab_down_monthly();
                        self.summary_table.state.select(None)
                    }
                }
                _ => *self.summary_tab = self.summary_tab.change_tab_down_monthly(),
            },
            1 => match self.summary_tab {
                SummaryTab::Table => {
                    if self.summary_table.state.selected() == Some(self.total_tags - 1) {
                        *self.summary_tab = self.summary_tab.change_tab_down_yearly();
                    } else {
                        self.summary_table.next()
                    }
                }
                SummaryTab::Years => {
                    if self.total_tags > 0 {
                        self.summary_table.state.select(Some(0));
                        *self.summary_tab = self.summary_tab.change_tab_down_yearly()
                    } else {
                        *self.summary_tab = self.summary_tab.change_tab_down_yearly();
                        *self.summary_tab = self.summary_tab.change_tab_down_yearly();
                        self.summary_table.state.select(None)
                    }
                }
                _ => *self.summary_tab = self.summary_tab.change_tab_down_yearly(),
            },
            2 => match self.summary_tab {
                SummaryTab::Table => {
                    if self.summary_table.state.selected() == Some(self.total_tags - 1) {
                        *self.summary_tab = self.summary_tab.change_tab_down_all_time();
                    } else {
                        self.summary_table.next()
                    }
                }
                SummaryTab::ModeSelection => {
                    if self.total_tags > 0 {
                        self.summary_table.state.select(Some(0));
                        *self.summary_tab = self.summary_tab.change_tab_down_all_time();
                    } else {
                        *self.summary_tab = self.summary_tab.change_tab_down_all_time();
                        *self.summary_tab = self.summary_tab.change_tab_down_all_time();
                        self.summary_table.state.select(None)
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn do_chart_up(&mut self) {
        if !*self.chart_hidden_mode {
            match self.chart_modes.index {
                0 => *self.chart_tab = self.chart_tab.change_tab_up_monthly(),
                1 => *self.chart_tab = self.chart_tab.change_tab_up_yearly(),
                _ => {}
            }
        }
    }

    fn do_chart_down(&mut self) {
        if !*self.chart_hidden_mode {
            match self.chart_modes.index {
                0 => *self.chart_tab = self.chart_tab.change_tab_down_monthly(),
                1 => *self.chart_tab = self.chart_tab.change_tab_down_yearly(),
                _ => {}
            }
        }
    }

    fn check_add_tx_date(&mut self) {
        match self.key.code {
            KeyCode::Enter => {
                let status = self.add_tx_data.check_date();
                self.add_tx_data.add_tx_status(status.to_string());
                match status {
                    VerifyingOutput::Accepted(_) | VerifyingOutput::Nothing(_) => {
                        *self.tx_tab = AddTxTab::Details
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Esc => {
                let status = self.add_tx_data.check_date();
                self.add_tx_data.add_tx_status(status.to_string());
                match status {
                    VerifyingOutput::Accepted(_) | VerifyingOutput::Nothing(_) => {
                        *self.tx_tab = AddTxTab::Nothing
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Backspace => self.add_tx_data.edit_date(None),
            KeyCode::Char(a) => self.add_tx_data.edit_date(Some(a)),
            _ => {}
        }
    }

    fn check_add_tx_method(&mut self) {
        match self.key.code {
            KeyCode::Enter => {
                let status = self.add_tx_data.check_from_method(self.conn);
                self.add_tx_data.add_tx_status(status.to_string());
                match status {
                    VerifyingOutput::Accepted(_) | VerifyingOutput::Nothing(_) => {
                        *self.tx_tab = AddTxTab::Amount
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Esc => {
                let status = self.add_tx_data.check_from_method(self.conn);
                self.add_tx_data.add_tx_status(status.to_string());
                match status {
                    VerifyingOutput::Accepted(_) | VerifyingOutput::Nothing(_) => {
                        *self.tx_tab = AddTxTab::Nothing
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Backspace => self.add_tx_data.edit_from_method(None),
            KeyCode::Char(a) => self.add_tx_data.edit_from_method(Some(a)),
            _ => {}
        }
    }

    fn check_add_tx_amount(&mut self) {
        match self.key.code {
            KeyCode::Enter => {
                let status = self.add_tx_data.check_amount(self.conn);
                self.add_tx_data.add_tx_status(status.to_string());
                match status {
                    VerifyingOutput::Accepted(_) | VerifyingOutput::Nothing(_) => {
                        *self.tx_tab = AddTxTab::TxType
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Esc => {
                let status = self.add_tx_data.check_amount(self.conn);
                self.add_tx_data.add_tx_status(status.to_string());
                match status {
                    VerifyingOutput::Accepted(_) | VerifyingOutput::Nothing(_) => {
                        *self.tx_tab = AddTxTab::Nothing
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Backspace => self.add_tx_data.edit_amount(None),
            KeyCode::Char(a) => self.add_tx_data.edit_amount(Some(a)),
            _ => {}
        }
    }

    fn check_add_tx_type(&mut self) {
        match self.key.code {
            KeyCode::Enter => {
                let status = self.add_tx_data.check_tx_type();
                self.add_tx_data.add_tx_status(status.to_string());
                match status {
                    VerifyingOutput::Accepted(_) | VerifyingOutput::Nothing(_) => {
                        *self.tx_tab = AddTxTab::Tags
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Esc => {
                let status = self.add_tx_data.check_tx_type();
                self.add_tx_data.add_tx_status(status.to_string());
                match status {
                    VerifyingOutput::Accepted(_) | VerifyingOutput::Nothing(_) => {
                        *self.tx_tab = AddTxTab::Nothing
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Backspace => self.add_tx_data.edit_tx_type(None),
            KeyCode::Char(a) => self.add_tx_data.edit_tx_type(Some(a)),
            _ => {}
        }
    }

    fn check_add_tx_details(&mut self) {
        match self.key.code {
            KeyCode::Enter => *self.tx_tab = AddTxTab::TxMethod,
            KeyCode::Esc => *self.tx_tab = AddTxTab::Nothing,
            KeyCode::Backspace => self.add_tx_data.edit_details(None),
            KeyCode::Char(a) => self.add_tx_data.edit_details(Some(a)),
            _ => {}
        }
    }

    fn check_add_tx_tags(&mut self) {
        match self.key.code {
            KeyCode::Enter => *self.tx_tab = AddTxTab::Nothing,
            KeyCode::Esc => *self.tx_tab = AddTxTab::Nothing,
            KeyCode::Backspace => self.add_tx_data.edit_tags(None),
            KeyCode::Char(a) => self.add_tx_data.edit_tags(Some(a)),
            _ => {}
        }
    }

    fn check_transfer_date(&mut self) {
        match self.key.code {
            KeyCode::Enter => {
                let status = self.transfer_data.check_date();
                self.transfer_data.add_tx_status(status.to_string());
                match status {
                    VerifyingOutput::Accepted(_) | VerifyingOutput::Nothing(_) => {
                        *self.transfer_tab = TransferTab::Details
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Esc => {
                let status = self.transfer_data.check_date();
                self.transfer_data.add_tx_status(status.to_string());
                match status {
                    VerifyingOutput::Accepted(_) | VerifyingOutput::Nothing(_) => {
                        *self.transfer_tab = TransferTab::Nothing
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Backspace => self.transfer_data.edit_date(None),
            KeyCode::Char(a) => self.transfer_data.edit_date(Some(a)),
            _ => {}
        }
    }

    fn check_transfer_details(&mut self) {
        match self.key.code {
            KeyCode::Enter => *self.transfer_tab = TransferTab::From,
            KeyCode::Esc => *self.transfer_tab = TransferTab::Nothing,
            KeyCode::Backspace => self.transfer_data.edit_details(None),
            KeyCode::Char(a) => self.transfer_data.edit_details(Some(a)),
            _ => {}
        }
    }

    fn check_transfer_from(&mut self) {
        match self.key.code {
            KeyCode::Enter => {
                let status = self.transfer_data.check_from_method(self.conn);
                self.transfer_data.add_tx_status(status.to_string());
                match status {
                    VerifyingOutput::Accepted(_) | VerifyingOutput::Nothing(_) => {
                        *self.transfer_tab = TransferTab::To
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Esc => {
                let status = self.transfer_data.check_from_method(self.conn);
                self.transfer_data.add_tx_status(status.to_string());
                match status {
                    VerifyingOutput::Accepted(_) | VerifyingOutput::Nothing(_) => {
                        *self.transfer_tab = TransferTab::Nothing
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Backspace => self.transfer_data.edit_from_method(None),
            KeyCode::Char(a) => self.transfer_data.edit_from_method(Some(a)),
            _ => {}
        }
    }

    fn check_transfer_to(&mut self) {
        match self.key.code {
            KeyCode::Enter => {
                let status = self.transfer_data.check_to_method(self.conn);
                self.transfer_data.add_tx_status(status.to_string());
                match status {
                    VerifyingOutput::Accepted(_) | VerifyingOutput::Nothing(_) => {
                        *self.transfer_tab = TransferTab::Amount
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Esc => {
                let status = self.transfer_data.check_to_method(self.conn);
                self.transfer_data.add_tx_status(status.to_string());
                match status {
                    VerifyingOutput::Accepted(_) | VerifyingOutput::Nothing(_) => {
                        *self.transfer_tab = TransferTab::Nothing
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Backspace => self.transfer_data.edit_to_method(None),
            KeyCode::Char(a) => self.transfer_data.edit_to_method(Some(a)),
            _ => {}
        }
    }

    fn check_transfer_amount(&mut self) {
        match self.key.code {
            KeyCode::Enter => {
                let status = self.transfer_data.check_amount(self.conn);
                self.transfer_data.add_tx_status(status.to_string());
                match status {
                    VerifyingOutput::Accepted(_) | VerifyingOutput::Nothing(_) => {
                        *self.transfer_tab = TransferTab::Tags
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Esc => {
                let status = self.transfer_data.check_amount(self.conn);
                self.transfer_data.add_tx_status(status.to_string());
                match status {
                    VerifyingOutput::Accepted(_) | VerifyingOutput::Nothing(_) => {
                        *self.transfer_tab = TransferTab::Nothing
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Backspace => self.transfer_data.edit_amount(None),
            KeyCode::Char(a) => self.transfer_data.edit_amount(Some(a)),
            _ => {}
        }
    }

    fn check_transfer_tags(&mut self) {
        match self.key.code {
            KeyCode::Enter => *self.transfer_tab = TransferTab::Nothing,
            KeyCode::Esc => *self.transfer_tab = TransferTab::Nothing,
            KeyCode::Backspace => self.transfer_data.edit_tags(None),
            KeyCode::Char(a) => self.transfer_data.edit_tags(Some(a)),
            _ => {}
        }
    }

    fn reload_home_table(&mut self) {
        *self.all_tx_data =
            TransactionData::new(self.conn, self.home_months.index, self.home_years.index);
        *self.table = TableData::new(self.all_tx_data.get_txs());
    }

    fn reload_summary(&mut self) {
        *self.summary_data = SummaryData::new(
            self.summary_modes,
            self.summary_months.index,
            self.summary_years.index,
            self.conn,
        );
        *self.summary_table = TableData::new(self.summary_data.get_table_data());
        self.total_tags = self.summary_data.get_table_data().len();
    }

    fn reload_chart(&mut self) {
        *self.chart_data = ChartData::set(
            self.chart_modes,
            self.chart_months.index,
            self.chart_years.index,
        );
        *self.chart_index = Some(0);
    }
}
