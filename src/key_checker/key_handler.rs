use crate::chart_page::ChartData;
use crate::home_page::TransactionData;
use crate::outputs::{HandlingOutput, VerifyingOutput};
use crate::page_handler::{
    ChartTab, CurrentUi, HomeTab, IndexedData, PopupState, SummaryTab, TableData, TxTab,
};
use crate::summary_page::SummaryData;
use crate::tx_handler::TxData;
use crossterm::event::{KeyCode, KeyEvent};
use rusqlite::Connection;

/// Stores all the data that is required to handle
/// every single possible key press event from the
/// entire app
#[cfg(not(tarpaulin_include))]
pub struct InputKeyHandler<'a> {
    pub key: KeyEvent,
    pub page: &'a mut CurrentUi,
    pub popup: &'a mut PopupState,
    pub add_tx_tab: &'a mut TxTab,
    pub transfer_tab: &'a mut TxTab,
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
    chart_index: &'a mut Option<f64>,
    chart_hidden_mode: &'a mut bool,
    summary_hidden_mode: &'a mut bool,
    conn: &'a mut Connection,
}

impl<'a> InputKeyHandler<'a> {
    pub fn new(
        key: KeyEvent,
        page: &'a mut CurrentUi,
        popup: &'a mut PopupState,
        add_tx_tab: &'a mut TxTab,
        transfer_tab: &'a mut TxTab,
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
        chart_index: &'a mut Option<f64>,
        chart_hidden_mode: &'a mut bool,
        summary_hidden_mode: &'a mut bool,
        conn: &'a mut Connection,
    ) -> InputKeyHandler<'a> {
        let total_tags = summary_data
            .get_table_data(summary_modes, summary_months.index, summary_years.index)
            .len();
        InputKeyHandler {
            key,
            page,
            popup,
            add_tx_tab,
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
            summary_hidden_mode,
            chart_hidden_mode,
            conn,
        }
    }

    /// Moves the interface to Home page and
    /// resets any selected widget/data from Add Tx or Transfer
    /// page to Nothing
    pub fn go_home_reset(&mut self) {
        match self.page {
            CurrentUi::AddTx => {
                *self.add_tx_data = TxData::new();
                *self.add_tx_tab = TxTab::Nothing;
            }
            CurrentUi::Transfer => {
                *self.transfer_tab = TxTab::Nothing;
                *self.transfer_data = TxData::new_transfer();
            }
            _ => {}
        }
        *self.page = CurrentUi::Home;
    }

    /// Moves the interface to Home page
    pub fn go_home(&mut self) {
        *self.page = CurrentUi::Home;
    }

    /// Moves the interface to Add Tx page
    pub fn go_add_tx(&mut self) {
        *self.page = CurrentUi::AddTx
    }

    /// Moves the interface to Transfer page
    pub fn go_transfer(&mut self) {
        *self.page = CurrentUi::Transfer
    }

    /// Moves the interface to Summary page
    pub fn go_summary(&mut self) {
        *self.page = CurrentUi::Summary;
        self.summary_modes.set_index_zero();
        self.summary_months.set_index_zero();
        self.summary_years.set_index_zero();
        *self.summary_tab = SummaryTab::ModeSelection;
        *self.summary_hidden_mode = false;
        self.reload_summary();
    }

    /// Moves the interface to Chart page
    pub fn go_chart(&mut self) {
        *self.page = CurrentUi::Chart;
        self.chart_modes.set_index_zero();
        self.chart_years.set_index_zero();
        self.chart_months.set_index_zero();
        *self.chart_tab = ChartTab::ModeSelection;
        *self.chart_hidden_mode = false;
        self.reload_chart();
    }

    /// Turns on help popup
    pub fn do_help_popup(&mut self) {
        match self.page {
            CurrentUi::Home => *self.popup = PopupState::HomeHelp,
            CurrentUi::AddTx => *self.popup = PopupState::AddTxHelp,
            CurrentUi::Transfer => *self.popup = PopupState::TransferHelp,
            CurrentUi::Chart => *self.popup = PopupState::ChartHelp,
            CurrentUi::Summary => *self.popup = PopupState::SummaryHelp,
            _ => {}
        }
    }

    /// Removes popup status
    pub fn do_empty_popup(&mut self) {
        *self.popup = PopupState::Nothing
    }

    /// Hides chart top widgets
    pub fn do_chart_hidden_mode(&mut self) {
        *self.chart_hidden_mode = !*self.chart_hidden_mode;
    }

    pub fn do_summary_hidden_mode(&mut self) {
        *self.summary_hidden_mode = !*self.summary_hidden_mode;

        if *self.summary_hidden_mode {
            *self.summary_tab = SummaryTab::Table;
            if self.total_tags > 0 {
                self.summary_table.state.select(Some(0));
            }
        } else {
            *self.summary_tab = SummaryTab::ModeSelection
        }
    }

    /// Handles Enter key press if there is a new update and the update popup is on
    pub fn handle_update_popup(&mut self) -> Result<(), HandlingOutput> {
        match self.key.code {
            KeyCode::Enter => {
                // If there is a new version, Enter will try to open the default browser with this link
                open::that("https://github.com/WaffleMixer/Rex/releases/latest")
                    .map_err(|_| HandlingOutput::PrintNewUpdate)?;
                *self.popup = PopupState::Nothing;
                Ok(())
            }
            _ => {
                *self.popup = PopupState::Nothing;
                Ok(())
            }
        }
    }

    /// Adds new tx and reloads home and chart data
    pub fn add_tx(&mut self) {
        let status = self.add_tx_data.add_tx(self.conn);
        if status.is_empty() {
            self.go_home_reset();
            // we just added a new tx, select the month tab again + reload the data of balance and table widgets to get updated data
            *self.home_tab = HomeTab::Months;
            self.reload_home_table();
            self.reload_chart_data();
            self.reload_summary_data();
        } else {
            self.add_tx_data.add_tx_status(status);
        }
    }

    /// Based on transaction Selected, opens Add Tx or Transfer Page and
    /// allocates the data of the tx to the input boxes
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

    /// Deletes the selected transaction and reloads home and chart page
    pub fn delete_tx(&mut self) {
        if let Some(index) = self.table.state.selected() {
            let status = self.all_tx_data.del_tx(index, self.conn);
            match status {
                Ok(_) => {
                    // transaction deleted so reload the data again
                    self.reload_home_table();
                    self.table.state.select(None);
                    *self.home_tab = HomeTab::Months;
                    self.reload_chart_data();
                    self.reload_summary_data();
                }
                Err(err) => {
                    *self.popup = PopupState::DeleteFailed(err.to_string());
                }
            }
        }
    }

    /// Adds a transfer transaction and reloads home and chart page
    pub fn add_transfer_tx(&mut self) {
        let status = self.transfer_data.add_tx(self.conn);
        if status == *"" {
            // reload home page and switch UI
            *self.home_tab = HomeTab::Months;
            self.go_home_reset();
            self.reload_home_table();
            self.reload_chart_data();
            self.reload_summary_data();
        } else {
            self.transfer_data.add_tx_status(status);
        }
    }

    /// Handles all number key presses and selects relevant input field
    pub fn handle_number_press(&mut self) {
        match self.page {
            CurrentUi::AddTx => {
                match self.key.code {
                    KeyCode::Char('1') => *self.add_tx_tab = TxTab::Date,
                    KeyCode::Char('2') => *self.add_tx_tab = TxTab::Details,
                    KeyCode::Char('3') => *self.add_tx_tab = TxTab::FromMethod,
                    KeyCode::Char('4') => *self.add_tx_tab = TxTab::Amount,
                    KeyCode::Char('5') => *self.add_tx_tab = TxTab::TxType,
                    KeyCode::Char('6') => *self.add_tx_tab = TxTab::Tags,
                    _ => {}
                }
                self.go_correct_index();
            }
            CurrentUi::Transfer => {
                match self.key.code {
                    KeyCode::Char('1') => *self.transfer_tab = TxTab::Date,
                    KeyCode::Char('2') => *self.transfer_tab = TxTab::Details,
                    KeyCode::Char('3') => *self.transfer_tab = TxTab::FromMethod,
                    KeyCode::Char('4') => *self.transfer_tab = TxTab::ToMethod,
                    KeyCode::Char('5') => *self.transfer_tab = TxTab::Amount,
                    KeyCode::Char('6') => *self.transfer_tab = TxTab::Tags,
                    _ => {}
                }
                self.go_correct_index();
            }
            _ => {}
        }
    }

    /// Handles left arrow key press for multiple pages
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
            CurrentUi::AddTx => self.add_tx_data.move_index_left(self.add_tx_tab),
            CurrentUi::Transfer => self.transfer_data.move_index_left(self.transfer_tab),
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
            CurrentUi::Summary => {
                if !*self.summary_hidden_mode {
                    match self.summary_tab {
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
                    }
                }
            }
            _ => {}
        }
    }

    /// Handles right arrow key press for multiple pages
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
            CurrentUi::AddTx => self.add_tx_data.move_index_right(self.add_tx_tab),
            CurrentUi::Transfer => self.transfer_data.move_index_right(self.transfer_tab),
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

    /// Handles up arrow key press for multiple pages
    pub fn handle_up_arrow(&mut self) {
        match self.page {
            CurrentUi::Home => self.do_home_up(),
            CurrentUi::AddTx => self.do_add_tx_up(),
            CurrentUi::Transfer => self.do_transfer_up(),
            CurrentUi::Summary => self.do_summary_up(),
            CurrentUi::Chart => self.do_chart_up(),
            _ => {}
        }
    }

    /// Handles down arrow key press for multiple pages
    pub fn handle_down_arrow(&mut self) {
        match self.page {
            CurrentUi::Home => self.do_home_down(),
            CurrentUi::AddTx => self.do_add_tx_down(),
            CurrentUi::Transfer => self.do_transfer_down(),
            CurrentUi::Summary => self.do_summary_down(),
            CurrentUi::Chart => self.do_chart_down(),
            _ => {}
        }
    }

    /// Checks and verifies date for Add Tx and Transfer page
    pub fn handle_date(&mut self) {
        match self.page {
            CurrentUi::AddTx => self.check_add_tx_date(),
            CurrentUi::Transfer => self.check_transfer_date(),
            _ => {}
        }
    }

    /// Checks and verifies details for Add Tx and Transfer page
    pub fn handle_details(&mut self) {
        match self.page {
            CurrentUi::AddTx => self.check_add_tx_details(),
            CurrentUi::Transfer => self.check_transfer_details(),
            _ => {}
        }
    }

    /// Checks and verifies tx method for Add Tx and Transfer page
    pub fn handle_tx_method(&mut self) {
        match self.page {
            CurrentUi::AddTx => self.check_add_tx_method(),
            CurrentUi::Transfer => match self.transfer_tab {
                TxTab::FromMethod => self.check_transfer_from(),
                TxTab::ToMethod => self.check_transfer_to(),
                _ => {}
            },
            _ => {}
        }
    }

    /// Checks and verifies amount for Add Tx and Transfer page
    pub fn handle_amount(&mut self) {
        match self.page {
            CurrentUi::AddTx => self.check_add_tx_amount(),
            CurrentUi::Transfer => self.check_transfer_amount(),
            _ => {}
        }
    }

    // Checks and verifies tx type for Add Tx page
    pub fn handle_tx_type(&mut self) {
        if let CurrentUi::AddTx = self.page {
            self.check_add_tx_type()
        }
    }

    /// Checks and verifies tags for Add Tx and Transfer page
    pub fn handle_tags(&mut self) {
        match self.page {
            CurrentUi::AddTx => self.check_add_tx_tags(),
            CurrentUi::Transfer => self.check_transfer_tags(),
            _ => {}
        }
    }

    /// Resets all input boxes on Add Tx and Transfer page
    pub fn clear_input(&mut self) {
        match self.page {
            CurrentUi::AddTx => *self.add_tx_data = TxData::new(),
            CurrentUi::Transfer => *self.transfer_data = TxData::new_transfer(),
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
                // if arrow key up is pressed and table index is 0, select the Month widget
                // else just select the upper index of the table
                if self.all_tx_data.all_tx.is_empty() {
                    *self.home_tab = self.home_tab.change_tab_up();
                } else if self.table.state.selected() == Some(0) {
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
                    // Move to the selected value on table widget
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
                // if arrow key down is pressed and table index is final, select the year widget
                // else just select the next index of the table
                if self.all_tx_data.all_tx.is_empty() {
                    *self.home_tab = self.home_tab.change_tab_down();
                } else if self.table.state.selected() == Some(self.table.items.len() - 1) {
                    *self.home_tab = HomeTab::Years;
                    self.table.state.select(None);
                } else if !self.all_tx_data.all_tx.is_empty() {
                    self.table.next();
                }
            }
            HomeTab::Months => {
                // Do not select any table rows in the table section If
                // there is no transaction
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
        if !*self.summary_hidden_mode {
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
        } else if self.total_tags > 0 {
            if self.summary_table.state.selected() == Some(0) {
                self.summary_table.state.select(Some(self.total_tags - 1));
            } else {
                self.summary_table.previous()
            }
        }
    }

    fn do_summary_down(&mut self) {
        if !*self.summary_hidden_mode {
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
        } else if self.total_tags > 0 {
            if self.summary_table.state.selected() == Some(self.total_tags - 1) {
                self.summary_table.state.select(Some(0));
            } else {
                self.summary_table.next()
            }
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
                        *self.add_tx_tab = TxTab::Details;
                        self.go_correct_index();
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Esc => {
                let status = self.add_tx_data.check_date();
                self.add_tx_data.add_tx_status(status.to_string());
                match status {
                    VerifyingOutput::Accepted(_) | VerifyingOutput::Nothing(_) => {
                        *self.add_tx_tab = TxTab::Nothing
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
                        *self.add_tx_tab = TxTab::Amount;
                        self.go_correct_index();
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Esc => {
                let status = self.add_tx_data.check_from_method(self.conn);
                self.add_tx_data.add_tx_status(status.to_string());
                match status {
                    VerifyingOutput::Accepted(_) | VerifyingOutput::Nothing(_) => {
                        *self.add_tx_tab = TxTab::Nothing
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
                        *self.add_tx_tab = TxTab::TxType;
                        self.go_correct_index();
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Esc => {
                let status = self.add_tx_data.check_amount(self.conn);
                self.add_tx_data.add_tx_status(status.to_string());
                match status {
                    VerifyingOutput::Accepted(_) | VerifyingOutput::Nothing(_) => {
                        *self.add_tx_tab = TxTab::Nothing;
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
                        *self.add_tx_tab = TxTab::Tags;
                        self.go_correct_index();
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Esc => {
                let status = self.add_tx_data.check_tx_type();
                self.add_tx_data.add_tx_status(status.to_string());
                match status {
                    VerifyingOutput::Accepted(_) | VerifyingOutput::Nothing(_) => {
                        *self.add_tx_tab = TxTab::Nothing
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
            KeyCode::Enter => {
                *self.add_tx_tab = TxTab::FromMethod;
                self.go_correct_index();
            }
            KeyCode::Esc => *self.add_tx_tab = TxTab::Nothing,
            KeyCode::Backspace => self.add_tx_data.edit_details(None),
            KeyCode::Char(a) => self.add_tx_data.edit_details(Some(a)),
            _ => {}
        }
    }

    fn check_add_tx_tags(&mut self) {
        match self.key.code {
            KeyCode::Enter => {
                *self.add_tx_tab = TxTab::Nothing;
                self.add_tx_data.check_tags();
            }
            KeyCode::Esc => {
                *self.add_tx_tab = TxTab::Nothing;
                self.add_tx_data.check_tags();
            }
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
                        *self.transfer_tab = TxTab::Details;
                        self.go_correct_index();
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Esc => {
                let status = self.transfer_data.check_date();
                self.transfer_data.add_tx_status(status.to_string());
                match status {
                    VerifyingOutput::Accepted(_) | VerifyingOutput::Nothing(_) => {
                        *self.transfer_tab = TxTab::Nothing
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
            KeyCode::Enter => {
                *self.transfer_tab = TxTab::FromMethod;
                self.go_correct_index();
            }
            KeyCode::Esc => *self.transfer_tab = TxTab::Nothing,
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
                        *self.transfer_tab = TxTab::ToMethod;
                        self.go_correct_index();
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Esc => {
                let status = self.transfer_data.check_from_method(self.conn);
                self.transfer_data.add_tx_status(status.to_string());
                match status {
                    VerifyingOutput::Accepted(_) | VerifyingOutput::Nothing(_) => {
                        *self.transfer_tab = TxTab::Nothing
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
                        *self.transfer_tab = TxTab::Amount;
                        self.go_correct_index();
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Esc => {
                let status = self.transfer_data.check_to_method(self.conn);
                self.transfer_data.add_tx_status(status.to_string());
                match status {
                    VerifyingOutput::Accepted(_) | VerifyingOutput::Nothing(_) => {
                        *self.transfer_tab = TxTab::Nothing
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
                        *self.transfer_tab = TxTab::Tags;
                        self.go_correct_index();
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Esc => {
                let status = self.transfer_data.check_amount(self.conn);
                self.transfer_data.add_tx_status(status.to_string());
                match status {
                    VerifyingOutput::Accepted(_) | VerifyingOutput::Nothing(_) => {
                        *self.transfer_tab = TxTab::Nothing
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
            KeyCode::Enter => {
                *self.transfer_tab = TxTab::Nothing;
                self.transfer_data.check_tags()
            }
            KeyCode::Esc => {
                *self.transfer_tab = TxTab::Nothing;
                self.transfer_data.check_tags()
            }
            KeyCode::Backspace => self.transfer_data.edit_tags(None),
            KeyCode::Char(a) => self.transfer_data.edit_tags(Some(a)),
            _ => {}
        }
    }

    fn reload_home_table(&mut self) {
        *self.all_tx_data =
            TransactionData::new(self.home_months.index, self.home_years.index, self.conn);
        *self.table = TableData::new(self.all_tx_data.get_txs());
    }

    fn reload_summary(&mut self) {
        let summary_table = self.summary_data.get_table_data(
            self.summary_modes,
            self.summary_months.index,
            self.summary_years.index,
        );
        self.total_tags = summary_table.len();
        *self.summary_table = TableData::new(summary_table);
    }

    fn reload_summary_data(&mut self) {
        *self.summary_data = SummaryData::new(self.conn);
    }

    fn reload_chart_data(&mut self) {
        *self.chart_data = ChartData::new(self.conn);
    }

    fn reload_chart(&mut self) {
        *self.chart_index = Some(0.0);
    }

    fn go_correct_index(&mut self) {
        match self.page {
            CurrentUi::AddTx => self.add_tx_data.go_current_index(self.add_tx_tab),
            CurrentUi::Transfer => self.transfer_data.go_current_index(self.transfer_tab),
            _ => {}
        }
    }

    fn do_add_tx_up(&mut self) {
        let status = match self.add_tx_tab {
            TxTab::Date => self.add_tx_data.do_date_up(),
            TxTab::FromMethod => self.add_tx_data.do_from_method_up(self.conn),
            TxTab::Amount => self.add_tx_data.do_amount_up(self.conn),
            TxTab::TxType => self.add_tx_data.do_tx_type_up(),
            TxTab::Tags => self.add_tx_data.do_tags_up(self.conn),
            _ => Ok(()),
        };

        if let Err(e) = status {
            self.add_tx_data.add_tx_status(e.to_string())
        }
    }

    fn do_add_tx_down(&mut self) {
        let status = match self.add_tx_tab {
            TxTab::Date => self.add_tx_data.do_date_down(),
            TxTab::FromMethod => self.add_tx_data.do_from_method_down(self.conn),
            TxTab::Amount => self.add_tx_data.do_amount_down(self.conn),
            TxTab::TxType => self.add_tx_data.do_tx_type_down(),
            TxTab::Tags => self.add_tx_data.do_tags_down(self.conn),
            _ => Ok(()),
        };

        if let Err(e) = status {
            self.add_tx_data.add_tx_status(e.to_string())
        }
    }

    fn do_transfer_up(&mut self) {
        let status = match self.transfer_tab {
            TxTab::Date => self.transfer_data.do_date_up(),
            TxTab::FromMethod => self.transfer_data.do_from_method_up(self.conn),
            TxTab::ToMethod => self.transfer_data.do_to_method_up(self.conn),
            TxTab::Amount => self.transfer_data.do_amount_up(self.conn),
            TxTab::Tags => self.transfer_data.do_tags_up(self.conn),
            _ => Ok(()),
        };

        if let Err(e) = status {
            self.transfer_data.add_tx_status(e.to_string())
        }
    }

    fn do_transfer_down(&mut self) {
        let status = match self.transfer_tab {
            TxTab::Date => self.transfer_data.do_date_down(),
            TxTab::FromMethod => self.transfer_data.do_from_method_down(self.conn),
            TxTab::ToMethod => self.transfer_data.do_to_method_down(self.conn),
            TxTab::Amount => self.transfer_data.do_amount_down(self.conn),
            TxTab::Tags => self.transfer_data.do_tags_down(self.conn),
            _ => Ok(()),
        };

        if let Err(e) = status {
            self.transfer_data.add_tx_status(e.to_string())
        }
    }
}
