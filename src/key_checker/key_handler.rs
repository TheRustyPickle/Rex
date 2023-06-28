use crate::chart_page::ChartData;
use crate::home_page::TransactionData;
use crate::outputs::TxType;
use crate::outputs::{HandlingOutput, VerifyingOutput};
use crate::page_handler::{
    ChartTab, CurrentUi, HomeTab, IndexedData, PopupState, SortingType, SummaryTab, TableData,
    TxTab,
};
use crate::summary_page::SummaryData;
use crate::tx_handler::TxData;
use crate::utility::sort_table_data;
use crossterm::event::{KeyCode, KeyEvent};
use rusqlite::Connection;

/// Stores all the data that is required to handle
/// every single possible key press event from the
/// entire app
pub struct InputKeyHandler<'a> {
    pub key: KeyEvent,
    pub page: &'a mut CurrentUi,
    pub popup: &'a mut PopupState,
    pub add_tx_tab: &'a mut TxTab,
    chart_tab: &'a mut ChartTab,
    summary_tab: &'a mut SummaryTab,
    home_tab: &'a mut HomeTab,
    add_tx_data: &'a mut TxData,
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
    summary_sort: &'a mut SortingType,
    search_data: &'a mut TxData,
    pub search_tab: &'a mut TxTab,
    search_table: &'a mut TableData,
    search_txs: &'a mut TransactionData,
    total_tags: usize,
    chart_index: &'a mut Option<f64>,
    chart_hidden_mode: &'a mut bool,
    summary_hidden_mode: &'a mut bool,
    conn: &'a mut Connection,
}

impl<'a> InputKeyHandler<'a> {
    #[cfg(not(tarpaulin_include))]
    pub fn new(
        key: KeyEvent,
        page: &'a mut CurrentUi,
        popup: &'a mut PopupState,
        add_tx_tab: &'a mut TxTab,
        chart_tab: &'a mut ChartTab,
        summary_tab: &'a mut SummaryTab,
        home_tab: &'a mut HomeTab,
        add_tx_data: &'a mut TxData,
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
        summary_sort: &'a mut SortingType,
        search_data: &'a mut TxData,
        search_tab: &'a mut TxTab,
        search_table: &'a mut TableData,
        search_txs: &'a mut TransactionData,
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
            chart_tab,
            summary_tab,
            home_tab,
            add_tx_data,
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
            summary_sort,
            search_data,
            search_tab,
            search_table,
            search_txs,
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
    #[cfg(not(tarpaulin_include))]
    pub fn go_home_reset(&mut self) {
        match self.page {
            CurrentUi::AddTx => {
                *self.add_tx_data = TxData::new();
                *self.add_tx_tab = TxTab::Nothing;
            }
            CurrentUi::Search => {
                *self.search_data = TxData::new();
                *self.search_tab = TxTab::Nothing;
            }
            _ => {}
        }
        *self.page = CurrentUi::Home;
    }

    /// Moves the interface to Home page
    #[cfg(not(tarpaulin_include))]
    pub fn go_home(&mut self) {
        *self.page = CurrentUi::Home;
    }

    /// Moves the interface to Add Tx page
    #[cfg(not(tarpaulin_include))]
    pub fn go_add_tx(&mut self) {
        *self.page = CurrentUi::AddTx
    }

    /// Moves the interface to Search page
    #[cfg(not(tarpaulin_include))]
    pub fn go_search(&mut self) {
        *self.page = CurrentUi::Search
    }

    /// Moves the interface to Summary page
    #[cfg(not(tarpaulin_include))]
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
    #[cfg(not(tarpaulin_include))]
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
    #[cfg(not(tarpaulin_include))]
    pub fn do_help_popup(&mut self) {
        match self.page {
            CurrentUi::Home => *self.popup = PopupState::HomeHelp,
            CurrentUi::AddTx => *self.popup = PopupState::AddTxHelp,
            CurrentUi::Chart => *self.popup = PopupState::ChartHelp,
            CurrentUi::Summary => *self.popup = PopupState::SummaryHelp,
            _ => {}
        }
    }

    /// Removes popup status
    #[cfg(not(tarpaulin_include))]
    pub fn do_empty_popup(&mut self) {
        *self.popup = PopupState::Nothing
    }

    /// Hides chart top widgets
    #[cfg(not(tarpaulin_include))]
    pub fn do_chart_hidden_mode(&mut self) {
        *self.chart_hidden_mode = !*self.chart_hidden_mode;
    }

    /// Hides summary top widgets
    #[cfg(not(tarpaulin_include))]
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
    #[cfg(not(tarpaulin_include))]
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

    #[cfg(not(tarpaulin_include))]
    pub fn search_tx(&mut self) {
        if self.search_data.check_all_empty() {
            self.search_data
                .add_tx_status("Search: All fields cannot be empty".to_string())
        } else {
            let search_txs = self.search_data.get_search_tx(self.conn);

            if search_txs.0.is_empty() {
                self.search_data.add_tx_status(
                    "Search: No transactions found with the provided input".to_string(),
                )
            } else {
                *self.search_txs =
                    TransactionData::new_search(search_txs.0.to_owned(), search_txs.1);
                *self.search_table = TableData::new(search_txs.0);
                self.search_table.state.select(Some(0));
            }
        }
    }

    /// Adds new tx and reloads home and chart data
    #[cfg(not(tarpaulin_include))]
    pub fn add_tx(&mut self) {
        let status = self.add_tx_data.add_tx(self.conn);

        match status {
            Ok(_) => {
                self.go_home_reset();
                // we just added a new tx, select the month tab again + reload the data of balance and table widgets to get updated data
                *self.home_tab = HomeTab::Months;
                self.reload_home_table();
                self.reload_chart_data();
                self.reload_summary_data();
                self.reload_search_data();
            }
            Err(e) => self.add_tx_data.add_tx_status(e),
        }
    }

    /// Based on transaction Selected, opens Add Tx page and
    /// allocates the data of the tx to the input boxes
    #[cfg(not(tarpaulin_include))]
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

                *self.add_tx_data = TxData::custom(
                    &target_data[0],
                    &target_data[1],
                    from_method,
                    to_method,
                    &target_data[3],
                    "Transfer",
                    &target_data[5],
                    target_id_num,
                );
                *self.page = CurrentUi::AddTx;
            }
        }
    }

    /// Deletes the selected transaction and reloads home and chart page
    #[cfg(not(tarpaulin_include))]
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
                    self.reload_search_data();
                }
                Err(err) => {
                    *self.popup = PopupState::DeleteFailed(err.to_string());
                }
            }
        }
    }

    /// Handles all number key presses and selects relevant input field
    #[cfg(not(tarpaulin_include))]
    pub fn handle_number_press(&mut self) {
        match self.page {
            CurrentUi::AddTx => match self.add_tx_data.get_tx_type() {
                TxType::IncomeExpense => match self.key.code {
                    KeyCode::Char('1') => *self.add_tx_tab = TxTab::Date,
                    KeyCode::Char('2') => *self.add_tx_tab = TxTab::Details,
                    KeyCode::Char('3') => *self.add_tx_tab = TxTab::TxType,
                    KeyCode::Char('4') => *self.add_tx_tab = TxTab::FromMethod,
                    KeyCode::Char('5') => *self.add_tx_tab = TxTab::Amount,
                    KeyCode::Char('6') => *self.add_tx_tab = TxTab::Tags,
                    _ => {}
                },
                TxType::Transfer => match self.key.code {
                    KeyCode::Char('1') => *self.add_tx_tab = TxTab::Date,
                    KeyCode::Char('2') => *self.add_tx_tab = TxTab::Details,
                    KeyCode::Char('3') => *self.add_tx_tab = TxTab::TxType,
                    KeyCode::Char('4') => *self.add_tx_tab = TxTab::FromMethod,
                    KeyCode::Char('5') => *self.add_tx_tab = TxTab::ToMethod,
                    KeyCode::Char('6') => *self.add_tx_tab = TxTab::Amount,
                    KeyCode::Char('7') => *self.add_tx_tab = TxTab::Tags,
                    _ => {}
                },
            },
            CurrentUi::Search => match self.search_data.get_tx_type() {
                TxType::IncomeExpense => match self.key.code {
                    KeyCode::Char('1') => *self.search_tab = TxTab::Date,
                    KeyCode::Char('2') => *self.search_tab = TxTab::Details,
                    KeyCode::Char('3') => *self.search_tab = TxTab::TxType,
                    KeyCode::Char('4') => *self.search_tab = TxTab::FromMethod,
                    KeyCode::Char('5') => *self.search_tab = TxTab::Amount,
                    KeyCode::Char('6') => *self.search_tab = TxTab::Tags,
                    _ => {}
                },
                TxType::Transfer => match self.key.code {
                    KeyCode::Char('1') => *self.search_tab = TxTab::Date,
                    KeyCode::Char('2') => *self.search_tab = TxTab::Details,
                    KeyCode::Char('3') => *self.search_tab = TxTab::TxType,
                    KeyCode::Char('4') => *self.search_tab = TxTab::FromMethod,
                    KeyCode::Char('5') => *self.search_tab = TxTab::ToMethod,
                    KeyCode::Char('6') => *self.search_tab = TxTab::Amount,
                    KeyCode::Char('7') => *self.search_tab = TxTab::Tags,
                    _ => {}
                },
            },
            _ => {}
        }
        self.go_correct_index();
        self.check_autofill();
    }

    /// Handles left arrow key press for multiple pages
    #[cfg(not(tarpaulin_include))]
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
            CurrentUi::Search => self.search_data.move_index_left(self.search_tab),
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
    #[cfg(not(tarpaulin_include))]
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
            CurrentUi::Search => self.search_data.move_index_right(self.search_tab),
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
    #[cfg(not(tarpaulin_include))]
    pub fn handle_up_arrow(&mut self) {
        match self.page {
            CurrentUi::Home => self.do_home_up(),
            CurrentUi::AddTx => self.do_add_tx_up(),
            CurrentUi::Summary => self.do_summary_up(),
            CurrentUi::Chart => self.do_chart_up(),
            CurrentUi::Search => self.do_search_up(),
            _ => {}
        }
        self.check_autofill();
    }

    /// Handles down arrow key press for multiple pages
    #[cfg(not(tarpaulin_include))]
    pub fn handle_down_arrow(&mut self) {
        match self.page {
            CurrentUi::Home => self.do_home_down(),
            CurrentUi::AddTx => self.do_add_tx_down(),
            CurrentUi::Summary => self.do_summary_down(),
            CurrentUi::Chart => self.do_chart_down(),
            CurrentUi::Search => self.do_search_down(),
            _ => {}
        }
        self.check_autofill();
    }

    /// Checks and verifies date field
    #[cfg(not(tarpaulin_include))]
    pub fn handle_date(&mut self) {
        match self.page {
            CurrentUi::AddTx => self.check_add_tx_date(),
            CurrentUi::Search => self.check_search_date(),
            _ => {}
        }
    }

    /// Checks and verifies details field
    #[cfg(not(tarpaulin_include))]
    pub fn handle_details(&mut self) {
        match self.page {
            CurrentUi::AddTx => self.check_add_tx_details(),
            CurrentUi::Search => self.check_search_details(),
            _ => {}
        }
        self.check_autofill();
    }

    /// Checks and verifies tx method field
    #[cfg(not(tarpaulin_include))]
    pub fn handle_tx_method(&mut self) {
        match self.page {
            CurrentUi::AddTx => match self.add_tx_tab {
                TxTab::FromMethod => self.check_add_tx_from(),
                TxTab::ToMethod => self.check_add_tx_to(),
                _ => {}
            },
            CurrentUi::Search => match self.search_tab {
                TxTab::FromMethod => self.check_search_from(),
                TxTab::ToMethod => self.check_search_to(),
                _ => {}
            },
            _ => {}
        }
        self.check_autofill();
    }

    /// Checks and verifies amount field
    #[cfg(not(tarpaulin_include))]
    pub fn handle_amount(&mut self) {
        match self.page {
            CurrentUi::AddTx => self.check_add_tx_amount(),
            CurrentUi::Search => self.check_search_amount(),
            _ => {}
        }
    }

    // Checks and verifies tx type field
    #[cfg(not(tarpaulin_include))]
    pub fn handle_tx_type(&mut self) {
        match self.page {
            CurrentUi::AddTx => self.check_add_tx_type(),
            CurrentUi::Search => self.check_search_type(),
            _ => {}
        }
    }

    /// Checks and verifies tags field
    #[cfg(not(tarpaulin_include))]
    pub fn handle_tags(&mut self) {
        match self.page {
            CurrentUi::AddTx => self.check_add_tx_tags(),
            CurrentUi::Search => self.check_search_tags(),
            _ => {}
        }
        self.check_autofill();
    }

    /// Resets all input boxes on Add Tx and Transfer page
    #[cfg(not(tarpaulin_include))]
    pub fn clear_input(&mut self) {
        match self.page {
            CurrentUi::AddTx => *self.add_tx_data = TxData::new(),
            CurrentUi::Search => *self.search_data = TxData::new(),
            _ => {}
        }
    }

    /// Takes the autofill value and adds it to the relevant field
    #[cfg(not(tarpaulin_include))]
    pub fn do_autofill(&mut self) {
        match self.page {
            CurrentUi::AddTx => self.add_tx_data.accept_autofill(self.add_tx_tab),
            CurrentUi::Search => self.search_data.accept_autofill(self.search_tab),
            _ => {}
        }
    }

    #[cfg(not(tarpaulin_include))]
    pub fn select_date_field(&mut self) {
        match self.page {
            CurrentUi::AddTx => *self.add_tx_tab = TxTab::Date,
            CurrentUi::Search => *self.search_tab = TxTab::Date,
            _ => {}
        }
        self.go_correct_index();
    }

    #[cfg(not(tarpaulin_include))]
    pub fn change_summary_sort(&mut self) {
        *self.summary_sort = self.summary_sort.next_type();
        let summary_data = self.summary_table.items.to_owned();
        let sorted_data = sort_table_data(summary_data, self.summary_sort);
        *self.summary_table = TableData::new(sorted_data);
    }
}

impl<'a> InputKeyHandler<'a> {
    #[cfg(not(tarpaulin_include))]
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

    #[cfg(not(tarpaulin_include))]
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

    #[cfg(not(tarpaulin_include))]
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

    #[cfg(not(tarpaulin_include))]
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

    #[cfg(not(tarpaulin_include))]
    fn do_chart_up(&mut self) {
        if !*self.chart_hidden_mode {
            match self.chart_modes.index {
                0 => *self.chart_tab = self.chart_tab.change_tab_up_monthly(),
                1 => *self.chart_tab = self.chart_tab.change_tab_up_yearly(),
                _ => {}
            }
        }
    }

    #[cfg(not(tarpaulin_include))]
    fn do_chart_down(&mut self) {
        if !*self.chart_hidden_mode {
            match self.chart_modes.index {
                0 => *self.chart_tab = self.chart_tab.change_tab_down_monthly(),
                1 => *self.chart_tab = self.chart_tab.change_tab_down_yearly(),
                _ => {}
            }
        }
    }

    #[cfg(not(tarpaulin_include))]
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

    #[cfg(not(tarpaulin_include))]
    fn check_add_tx_details(&mut self) {
        match self.key.code {
            KeyCode::Enter => {
                *self.add_tx_tab = TxTab::TxType;
                self.go_correct_index();
            }
            KeyCode::Esc => *self.add_tx_tab = TxTab::Nothing,
            KeyCode::Backspace => self.add_tx_data.edit_details(None),
            KeyCode::Char(a) => self.add_tx_data.edit_details(Some(a)),
            _ => {}
        }
    }

    #[cfg(not(tarpaulin_include))]
    fn check_add_tx_type(&mut self) {
        match self.key.code {
            KeyCode::Enter => {
                let status = self.add_tx_data.check_tx_type();
                self.add_tx_data.add_tx_status(status.to_string());
                match status {
                    VerifyingOutput::Accepted(_) | VerifyingOutput::Nothing(_) => {
                        *self.add_tx_tab = TxTab::FromMethod;
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

    #[cfg(not(tarpaulin_include))]
    fn check_add_tx_from(&mut self) {
        match self.key.code {
            KeyCode::Enter => {
                let status = self.add_tx_data.check_from_method(self.conn);
                self.add_tx_data.add_tx_status(status.to_string());
                match status {
                    VerifyingOutput::Accepted(_) | VerifyingOutput::Nothing(_) => {
                        match self.add_tx_data.get_tx_type() {
                            TxType::IncomeExpense => *self.add_tx_tab = TxTab::Amount,
                            TxType::Transfer => *self.add_tx_tab = TxTab::ToMethod,
                        }
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

    #[cfg(not(tarpaulin_include))]
    fn check_add_tx_to(&mut self) {
        match self.key.code {
            KeyCode::Enter => {
                let status = self.add_tx_data.check_to_method(self.conn);
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
                let status = self.add_tx_data.check_to_method(self.conn);
                self.add_tx_data.add_tx_status(status.to_string());
                match status {
                    VerifyingOutput::Accepted(_) | VerifyingOutput::Nothing(_) => {
                        *self.add_tx_tab = TxTab::Nothing
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Backspace => self.add_tx_data.edit_to_method(None),
            KeyCode::Char(a) => self.add_tx_data.edit_to_method(Some(a)),
            _ => {}
        }
    }

    #[cfg(not(tarpaulin_include))]
    fn check_add_tx_amount(&mut self) {
        match self.key.code {
            KeyCode::Enter => {
                let status = self.add_tx_data.check_amount(false, self.conn);
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
                let status = self.add_tx_data.check_amount(false, self.conn);
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

    #[cfg(not(tarpaulin_include))]
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

    #[cfg(not(tarpaulin_include))]
    fn check_search_date(&mut self) {
        match self.key.code {
            KeyCode::Enter => {
                let status = self.search_data.check_date();
                self.search_data.add_tx_status(status.to_string());
                match status {
                    VerifyingOutput::Accepted(_) | VerifyingOutput::Nothing(_) => {
                        *self.search_tab = TxTab::Details;
                        self.go_correct_index();
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Esc => {
                let status = self.search_data.check_date();
                self.search_data.add_tx_status(status.to_string());
                match status {
                    VerifyingOutput::Accepted(_) | VerifyingOutput::Nothing(_) => {
                        *self.search_tab = TxTab::Nothing
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Backspace => self.search_data.edit_date(None),
            KeyCode::Char(a) => self.search_data.edit_date(Some(a)),
            _ => {}
        }
    }

    #[cfg(not(tarpaulin_include))]
    fn check_search_details(&mut self) {
        match self.key.code {
            KeyCode::Enter => {
                *self.search_tab = TxTab::TxType;
                self.go_correct_index();
            }
            KeyCode::Esc => *self.search_tab = TxTab::Nothing,
            KeyCode::Backspace => self.search_data.edit_details(None),
            KeyCode::Char(a) => self.search_data.edit_details(Some(a)),
            _ => {}
        }
    }

    #[cfg(not(tarpaulin_include))]
    fn check_search_type(&mut self) {
        match self.key.code {
            KeyCode::Enter => {
                let status = self.search_data.check_tx_type();
                self.search_data.add_tx_status(status.to_string());
                match status {
                    VerifyingOutput::Accepted(_) | VerifyingOutput::Nothing(_) => {
                        *self.search_tab = TxTab::FromMethod;
                        self.go_correct_index();
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Esc => {
                let status = self.search_data.check_tx_type();
                self.search_data.add_tx_status(status.to_string());
                match status {
                    VerifyingOutput::Accepted(_) | VerifyingOutput::Nothing(_) => {
                        *self.search_tab = TxTab::Nothing
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Backspace => self.search_data.edit_tx_type(None),
            KeyCode::Char(a) => self.search_data.edit_tx_type(Some(a)),
            _ => {}
        }
    }

    #[cfg(not(tarpaulin_include))]
    fn check_search_from(&mut self) {
        match self.key.code {
            KeyCode::Enter => {
                let status = self.search_data.check_from_method(self.conn);
                self.search_data.add_tx_status(status.to_string());
                match status {
                    VerifyingOutput::Accepted(_) | VerifyingOutput::Nothing(_) => {
                        match self.search_data.get_tx_type() {
                            TxType::IncomeExpense => *self.search_tab = TxTab::Amount,
                            TxType::Transfer => *self.search_tab = TxTab::ToMethod,
                        }
                        self.go_correct_index();
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Esc => {
                let status = self.search_data.check_from_method(self.conn);
                self.search_data.add_tx_status(status.to_string());
                match status {
                    VerifyingOutput::Accepted(_) | VerifyingOutput::Nothing(_) => {
                        *self.search_tab = TxTab::Nothing
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Backspace => self.search_data.edit_from_method(None),
            KeyCode::Char(a) => self.search_data.edit_from_method(Some(a)),
            _ => {}
        }
    }

    #[cfg(not(tarpaulin_include))]
    fn check_search_to(&mut self) {
        match self.key.code {
            KeyCode::Enter => {
                let status = self.search_data.check_to_method(self.conn);
                self.search_data.add_tx_status(status.to_string());
                match status {
                    VerifyingOutput::Accepted(_) | VerifyingOutput::Nothing(_) => {
                        *self.search_tab = TxTab::Amount;
                        self.go_correct_index();
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Esc => {
                let status = self.search_data.check_to_method(self.conn);
                self.search_data.add_tx_status(status.to_string());
                match status {
                    VerifyingOutput::Accepted(_) | VerifyingOutput::Nothing(_) => {
                        *self.search_tab = TxTab::Nothing
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Backspace => self.search_data.edit_to_method(None),
            KeyCode::Char(a) => self.search_data.edit_to_method(Some(a)),
            _ => {}
        }
    }

    #[cfg(not(tarpaulin_include))]
    fn check_search_amount(&mut self) {
        match self.key.code {
            KeyCode::Enter => {
                let status = self.search_data.check_amount(true, self.conn);
                self.search_data.add_tx_status(status.to_string());
                match status {
                    VerifyingOutput::Accepted(_) | VerifyingOutput::Nothing(_) => {
                        *self.search_tab = TxTab::Tags;
                        self.go_correct_index();
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Esc => {
                let status = self.search_data.check_amount(true, self.conn);
                self.search_data.add_tx_status(status.to_string());
                match status {
                    VerifyingOutput::Accepted(_) | VerifyingOutput::Nothing(_) => {
                        *self.search_tab = TxTab::Nothing
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Backspace => self.search_data.edit_amount(None),
            KeyCode::Char(a) => self.search_data.edit_amount(Some(a)),
            _ => {}
        }
    }

    #[cfg(not(tarpaulin_include))]
    fn check_search_tags(&mut self) {
        match self.key.code {
            KeyCode::Enter => {
                *self.search_tab = TxTab::Nothing;
                self.search_data.check_tags()
            }
            KeyCode::Esc => {
                *self.search_tab = TxTab::Nothing;
                self.search_data.check_tags()
            }
            KeyCode::Backspace => self.search_data.edit_tags(None),
            KeyCode::Char(a) => self.search_data.edit_tags(Some(a)),
            _ => {}
        }
    }

    #[cfg(not(tarpaulin_include))]
    fn reload_home_table(&mut self) {
        *self.all_tx_data =
            TransactionData::new(self.home_months.index, self.home_years.index, self.conn);
        *self.table = TableData::new(self.all_tx_data.get_txs());
    }

    #[cfg(not(tarpaulin_include))]
    fn reload_summary(&mut self) {
        let summary_table = self.summary_data.get_table_data(
            self.summary_modes,
            self.summary_months.index,
            self.summary_years.index,
        );
        self.total_tags = summary_table.len();
        *self.summary_table = TableData::new(summary_table);
        *self.summary_sort = SortingType::ByTags;
    }

    #[cfg(not(tarpaulin_include))]
    fn reload_summary_data(&mut self) {
        *self.summary_data = SummaryData::new(self.conn);
    }

    #[cfg(not(tarpaulin_include))]
    fn reload_chart_data(&mut self) {
        *self.chart_data = ChartData::new(self.conn);
    }

    #[cfg(not(tarpaulin_include))]
    fn reload_chart(&mut self) {
        *self.chart_index = Some(0.0);
    }

    #[cfg(not(tarpaulin_include))]
    fn reload_search_data(&mut self) {
        *self.search_table = TableData::new(Vec::new());
        *self.search_txs = TransactionData::new_search(Vec::new(), Vec::new())
    }

    #[cfg(not(tarpaulin_include))]
    fn go_correct_index(&mut self) {
        match self.page {
            CurrentUi::AddTx => self.add_tx_data.go_current_index(self.add_tx_tab),
            CurrentUi::Search => self.search_data.go_current_index(self.search_tab),
            _ => {}
        }
    }

    #[cfg(not(tarpaulin_include))]
    fn do_add_tx_up(&mut self) {
        let status = match self.add_tx_tab {
            TxTab::Date => self.add_tx_data.do_date_up(),
            TxTab::FromMethod => self.add_tx_data.do_from_method_up(self.conn),
            TxTab::ToMethod => self.add_tx_data.do_to_method_up(self.conn),
            TxTab::Amount => self.add_tx_data.do_amount_up(false, self.conn),
            TxTab::TxType => self.add_tx_data.do_tx_type_up(),
            TxTab::Tags => self.add_tx_data.do_tags_up(self.conn),
            _ => Ok(()),
        };

        if let Err(e) = status {
            self.add_tx_data.add_tx_status(e.to_string())
        }
    }

    #[cfg(not(tarpaulin_include))]
    fn do_add_tx_down(&mut self) {
        let status = match self.add_tx_tab {
            TxTab::Date => self.add_tx_data.do_date_down(),
            TxTab::FromMethod => self.add_tx_data.do_from_method_down(self.conn),
            TxTab::ToMethod => self.add_tx_data.do_to_method_down(self.conn),
            TxTab::Amount => self.add_tx_data.do_amount_down(false, self.conn),
            TxTab::TxType => self.add_tx_data.do_tx_type_down(),
            TxTab::Tags => self.add_tx_data.do_tags_down(self.conn),
            _ => Ok(()),
        };

        if let Err(e) = status {
            self.add_tx_data.add_tx_status(e.to_string())
        }
    }

    #[cfg(not(tarpaulin_include))]
    fn do_search_up(&mut self) {
        let status = match self.search_tab {
            TxTab::Date => self.search_data.do_date_up(),
            TxTab::FromMethod => self.search_data.do_from_method_up(self.conn),
            TxTab::ToMethod => self.search_data.do_to_method_up(self.conn),
            TxTab::Amount => self.search_data.do_amount_up(true, self.conn),
            TxTab::TxType => self.search_data.do_tx_type_up(),
            TxTab::Tags => self.search_data.do_tags_up(self.conn),
            TxTab::Nothing => {
                if self.search_table.state.selected() == Some(0) {
                    self.search_table
                        .state
                        .select(Some(self.search_table.items.len() - 1));
                } else if !self.search_txs.all_tx.is_empty() {
                    self.search_table.previous();
                }
                Ok(())
            }
            _ => Ok(()),
        };

        if let Err(e) = status {
            self.search_data.add_tx_status(e.to_string())
        }
    }

    #[cfg(not(tarpaulin_include))]
    fn do_search_down(&mut self) {
        let status = match self.search_tab {
            TxTab::Date => self.search_data.do_date_down(),
            TxTab::FromMethod => self.search_data.do_from_method_down(self.conn),
            TxTab::ToMethod => self.search_data.do_to_method_down(self.conn),
            TxTab::Amount => self.search_data.do_amount_down(true, self.conn),
            TxTab::TxType => self.search_data.do_tx_type_down(),
            TxTab::Tags => self.search_data.do_tags_down(self.conn),
            TxTab::Nothing => {
                if self.search_table.state.selected() == Some(self.search_table.items.len() - 1) {
                    self.search_table.state.select(Some(0));
                } else if !self.search_txs.all_tx.is_empty() {
                    self.search_table.next();
                }
                Ok(())
            }
            _ => Ok(()),
        };

        if let Err(e) = status {
            self.search_data.add_tx_status(e.to_string())
        }
    }

    #[cfg(not(tarpaulin_include))]
    fn check_autofill(&mut self) {
        match self.page {
            CurrentUi::AddTx => self.add_tx_data.check_autofill(self.add_tx_tab, self.conn),
            CurrentUi::Search => self.search_data.check_autofill(self.search_tab, self.conn),
            _ => {}
        }
    }
}
