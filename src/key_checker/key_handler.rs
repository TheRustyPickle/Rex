use crossterm::event::{KeyCode, KeyEvent};
use rusqlite::Connection;
use std::collections::HashMap;

use crate::activity_page::ActivityData;
use crate::chart_page::ChartData;
use crate::home_page::TransactionData;
use crate::outputs::TxType;
use crate::outputs::{HandlingOutput, TxUpdateError, VerifyingOutput};
use crate::page_handler::{
    ActivityTab, ActivityType, ChartTab, CurrentUi, DateType, DeletionStatus, HomeTab, IndexedData,
    PopupState, SortingType, SummaryTab, TableData, TxTab,
};
use crate::summary_page::SummaryData;
use crate::tx_handler::TxData;
use crate::utility::{
    add_new_activity, add_new_activity_tx, get_all_tx_methods, get_all_tx_methods_cumulative,
    get_empty_changes, sort_table_data, switch_tx_index, LerpState,
};

/// Stores all the data that is required to handle
/// every single possible key press event from the
/// entire app
pub struct InputKeyHandler<'a> {
    pub key: KeyEvent,
    pub page: &'a mut CurrentUi,
    balance_data: &'a mut Vec<Vec<String>>,
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
    chart_tx_methods: &'a mut IndexedData,
    summary_months: &'a mut IndexedData,
    summary_years: &'a mut IndexedData,
    summary_modes: &'a mut IndexedData,
    summary_sort: &'a mut SortingType,
    search_data: &'a mut TxData,
    search_date_type: &'a mut DateType,
    pub search_tab: &'a mut TxTab,
    search_table: &'a mut TableData,
    search_txs: &'a mut TransactionData,
    activity_months: &'a mut IndexedData,
    activity_years: &'a mut IndexedData,
    activity_tab: &'a mut ActivityTab,
    activity_data: &'a mut ActivityData,
    activity_table: &'a mut TableData,
    total_tags: usize,
    chart_hidden_mode: &'a mut bool,
    summary_hidden_mode: &'a mut bool,
    deletion_status: &'a mut DeletionStatus,
    chart_activated_methods: &'a mut HashMap<String, bool>,
    popup_scroll_position: &'a mut usize,
    max_popup_scroll: &'a mut usize,
    lerp_state: &'a mut LerpState,
    conn: &'a mut Connection,
}

impl<'a> InputKeyHandler<'a> {
    #[cfg(not(tarpaulin_include))]
    pub fn new(
        key: KeyEvent,
        page: &'a mut CurrentUi,
        balance_data: &'a mut Vec<Vec<String>>,
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
        chart_tx_methods: &'a mut IndexedData,
        summary_months: &'a mut IndexedData,
        summary_years: &'a mut IndexedData,
        summary_modes: &'a mut IndexedData,
        summary_sort: &'a mut SortingType,
        search_data: &'a mut TxData,
        search_date_type: &'a mut DateType,
        search_tab: &'a mut TxTab,
        search_table: &'a mut TableData,
        search_txs: &'a mut TransactionData,
        activity_months: &'a mut IndexedData,
        activity_years: &'a mut IndexedData,
        activity_tab: &'a mut ActivityTab,
        activity_data: &'a mut ActivityData,
        activity_table: &'a mut TableData,
        chart_hidden_mode: &'a mut bool,
        summary_hidden_mode: &'a mut bool,
        deletion_status: &'a mut DeletionStatus,
        chart_activated_methods: &'a mut HashMap<String, bool>,
        popup_scroll_position: &'a mut usize,
        max_popup_scroll: &'a mut usize,
        lerp_state: &'a mut LerpState,
        conn: &'a mut Connection,
    ) -> InputKeyHandler<'a> {
        let total_tags = summary_data
            .get_table_data(summary_modes, summary_months.index, summary_years.index)
            .len();
        InputKeyHandler {
            key,
            page,
            balance_data,
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
            chart_tx_methods,
            summary_months,
            summary_years,
            summary_modes,
            summary_sort,
            search_data,
            search_date_type,
            search_tab,
            search_table,
            search_txs,
            activity_months,
            activity_years,
            activity_tab,
            activity_data,
            activity_table,
            total_tags,
            chart_hidden_mode,
            summary_hidden_mode,
            deletion_status,
            chart_activated_methods,
            popup_scroll_position,
            max_popup_scroll,
            lerp_state,
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
                *self.search_data = TxData::new_empty();
                *self.search_tab = TxTab::Nothing;
            }
            _ => {}
        }
        self.go_home();
    }

    /// Moves the interface to Home page
    #[cfg(not(tarpaulin_include))]
    pub fn go_home(&mut self) {
        *self.page = CurrentUi::Home;
        self.reload_home_balance_data();
        self.lerp_state.clear();
    }

    /// Moves the interface to Add Tx page
    #[cfg(not(tarpaulin_include))]
    pub fn go_add_tx(&mut self) {
        *self.page = CurrentUi::AddTx;
        self.add_tx_data
            .add_tx_status("Info: Entering Normal Transaction mode.".to_string());
        self.reload_add_tx_balance_data();
        self.lerp_state.clear();
    }

    /// Moves the interface to Search page
    #[cfg(not(tarpaulin_include))]
    pub fn go_search(&mut self) {
        *self.page = CurrentUi::Search;
        self.lerp_state.clear();
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
        self.lerp_state.clear();
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
        self.lerp_state.clear();
    }

    #[cfg(not(tarpaulin_include))]
    pub fn go_activity(&mut self) {
        *self.page = CurrentUi::Activity;
        self.lerp_state.clear();
    }

    /// Turns on help popup
    #[cfg(not(tarpaulin_include))]
    pub fn do_help_popup(&mut self) {
        match self.page {
            CurrentUi::Home => *self.popup = PopupState::HomeHelp,
            CurrentUi::AddTx => *self.popup = PopupState::AddTxHelp,
            CurrentUi::Chart => *self.popup = PopupState::ChartHelp,
            CurrentUi::Summary => *self.popup = PopupState::SummaryHelp,
            CurrentUi::Search => *self.popup = PopupState::SearchHelp,
            CurrentUi::Activity => *self.popup = PopupState::ActivityHelp,
            CurrentUi::Initial => {}
        }
    }

    /// Turns on deletion confirmation popup
    #[cfg(not(tarpaulin_include))]
    pub fn do_deletion_popup(&mut self) {
        match self.page {
            CurrentUi::Home => {
                if self.table.state.selected().is_some() {
                    *self.popup = PopupState::TxDeletion;
                }
            }
            CurrentUi::Search => {
                if self.search_table.state.selected().is_some() {
                    *self.popup = PopupState::TxDeletion;
                }
            }
            _ => {}
        }
    }

    /// Removes pop up status
    #[cfg(not(tarpaulin_include))]
    pub fn do_empty_popup(&mut self) {
        *self.popup = PopupState::Nothing;
        self.reload_popup_scroll_position();
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
            *self.summary_tab = SummaryTab::ModeSelection;
        }
    }

    /// Handles Enter key press if there is a new update and the update popup is on
    #[cfg(not(tarpaulin_include))]
    pub fn handle_update_popup(&mut self) -> Result<(), HandlingOutput> {
        if self.key.code == KeyCode::Enter {
            // If there is a new version, Enter will try to open the default browser with this link
            open::that("https://github.com/TheRustyPickle/Rex/releases/latest")
                .map_err(|_| HandlingOutput::PrintNewUpdate)?;
            *self.popup = PopupState::Nothing;
            Ok(())
        } else {
            *self.popup = PopupState::Nothing;
            Ok(())
        }
    }

    #[cfg(not(tarpaulin_include))]
    pub fn search_tx(&mut self) {
        if self.search_data.check_all_empty() {
            self.search_data
                .add_tx_status("Search: All fields cannot be empty".to_string());
        } else {
            let search_txs = self
                .search_data
                .get_search_tx(self.search_date_type, self.conn);

            if search_txs.0.is_empty() {
                self.search_data.add_tx_status(
                    "Search: No transactions found with the provided input".to_string(),
                );
            } else {
                *self.search_txs = TransactionData::new_search(search_txs.0.clone(), search_txs.1);
                *self.search_table = TableData::new(search_txs.0);
                self.search_table.state.select(Some(0));
                self.search_data.add_tx_status(format!(
                    "Search: Found {} Transactions",
                    self.search_table.items.len()
                ));
            }
            self.reload_activity_table();
        }
    }

    /// Adds new tx and reloads home and chart data
    #[cfg(not(tarpaulin_include))]
    pub fn add_tx(&mut self) {
        let status = self.add_tx_data.add_tx(self.conn);

        match status {
            Ok(()) => {
                self.go_home_reset();
                // We just added a new tx, select the month tab again + reload the data of balance and table widgets to get updated data
                *self.home_tab = HomeTab::Months;
                self.reload_home_table();
                self.reload_chart_data();
                self.reload_summary_data();
                self.reset_search_data();
                self.reload_activity_table();
            }
            Err(e) => self.add_tx_data.add_tx_status(e),
        }
    }

    /// Based on transaction Selected, opens Add Tx page and
    /// allocates the data of the tx to the input boxes
    #[cfg(not(tarpaulin_include))]
    pub fn home_edit_tx(&mut self) {
        if let Some(a) = self.table.state.selected() {
            let target_data = self.all_tx_data.get_tx(a);
            let target_id_num = self.all_tx_data.get_id_num(a);
            let tx_type = &target_data[4];

            // Based on what kind of transaction is selected, passes the tx data to the struct
            // and change the current interface
            if tx_type == "Transfer" {
                let split_method = target_data[2].split(" to ").collect::<Vec<&str>>();
                let from_method = split_method[0];
                let to_method = split_method[1];

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
            } else {
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
            }
            self.add_tx_data.add_tx_status(
                "Info: Entering Transaction edit mode. Press C to reset.".to_string(),
            );
            self.reload_add_tx_balance_data();
            self.lerp_state.clear();
        }
    }

    /// Deletes the selected transaction and reloads pages
    #[cfg(not(tarpaulin_include))]
    pub fn home_delete_tx(&mut self) {
        if let Some(index) = self.table.state.selected() {
            let mut tx_data = self.all_tx_data.get_tx(index).to_owned();
            let id_num = self.all_tx_data.get_id_num(index);
            tx_data.push(id_num.to_string());

            let status = self.all_tx_data.del_tx(index, self.conn);
            match status {
                Ok(()) => {
                    // Transaction deleted so reload the data again
                    self.reload_home_table();
                    self.reload_chart_data();
                    self.reload_summary_data();
                    self.reset_search_data();
                    self.reload_activity_table();

                    if index == 0 {
                        self.table.state.select(None);
                        *self.home_tab = HomeTab::Months;
                    } else {
                        self.table.state.select(Some(index - 1));
                    }

                    let activity_num =
                        add_new_activity(ActivityType::DeleteTX(Some(id_num)), self.conn);
                    add_new_activity_tx(&tx_data, activity_num, self.conn);
                }
                Err(err) => {
                    *self.popup =
                        PopupState::DeleteFailed(TxUpdateError::FailedDeleteTx(err).to_string());
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
                HomeTab::Table => {}
            },
            CurrentUi::AddTx => self.add_tx_data.move_index_left(self.add_tx_tab),
            CurrentUi::Search => self.search_data.move_index_left(self.search_tab),
            CurrentUi::Chart => {
                if !*self.chart_hidden_mode {
                    match self.chart_tab {
                        ChartTab::ModeSelection => {
                            self.chart_modes.previous();
                            self.lerp_state.clear();
                        }
                        ChartTab::Years => {
                            self.chart_years.previous();
                            self.lerp_state.clear();
                            self.chart_months.set_index_zero();
                        }
                        ChartTab::Months => {
                            self.chart_months.previous();
                            self.lerp_state.clear();
                        }
                        ChartTab::TxMethods => {
                            self.chart_tx_methods.previous();
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
                        SummaryTab::Table => {}
                    }
                }
            }
            CurrentUi::Activity => match self.activity_tab {
                ActivityTab::Years => {
                    self.activity_months.set_index_zero();
                    self.activity_years.previous();
                    self.reload_activity_table();
                }
                ActivityTab::Months => {
                    self.activity_months.previous();
                    self.reload_activity_table();
                }
                ActivityTab::List => {}
            },
            CurrentUi::Initial => {}
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
                HomeTab::Table => {}
            },
            CurrentUi::AddTx => self.add_tx_data.move_index_right(self.add_tx_tab),
            CurrentUi::Search => self.search_data.move_index_right(self.search_tab),
            CurrentUi::Chart => {
                if !*self.chart_hidden_mode {
                    match self.chart_tab {
                        ChartTab::ModeSelection => {
                            self.lerp_state.clear();
                            self.chart_modes.next();
                        }
                        ChartTab::Years => {
                            self.lerp_state.clear();
                            self.chart_years.next();
                            self.chart_months.set_index_zero();
                        }
                        ChartTab::Months => {
                            self.lerp_state.clear();
                            self.chart_months.next();
                        }
                        ChartTab::TxMethods => {
                            self.chart_tx_methods.next();
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
                SummaryTab::Table => {}
            },
            CurrentUi::Activity => match self.activity_tab {
                ActivityTab::Years => {
                    self.activity_months.set_index_zero();
                    self.activity_years.next();
                    self.reload_activity_table();
                }
                ActivityTab::Months => {
                    self.activity_months.next();
                    self.reload_activity_table();
                }
                ActivityTab::List => {}
            },
            CurrentUi::Initial => {}
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
            CurrentUi::Activity => self.do_activity_up(),
            CurrentUi::Initial => {}
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
            CurrentUi::Activity => self.do_activity_down(),
            CurrentUi::Initial => {}
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
            CurrentUi::AddTx => {
                *self.add_tx_data = TxData::new();
                self.reload_add_tx_balance_data();
            }
            CurrentUi::Search => {
                *self.search_data = TxData::new_empty();
                self.reset_search_data();
            }
            _ => {}
        }
    }

    /// Takes the auto fill value and adds it to the relevant field
    #[cfg(not(tarpaulin_include))]
    pub fn do_autofill(&mut self) {
        match self.page {
            CurrentUi::AddTx => self.add_tx_data.accept_autofill(self.add_tx_tab),
            CurrentUi::Search => self.search_data.accept_autofill(self.search_tab),
            _ => {}
        }
    }

    /// No field selected on add tx or search but enter is pressed then
    /// select the Date field
    #[cfg(not(tarpaulin_include))]
    pub fn select_date_field(&mut self) {
        match self.page {
            CurrentUi::AddTx => *self.add_tx_tab = TxTab::Date,
            CurrentUi::Search => *self.search_tab = TxTab::Date,
            _ => {}
        }
        self.go_correct_index();
    }

    /// Cycles through tag, income, expense table sorting on summary page
    #[cfg(not(tarpaulin_include))]
    pub fn change_summary_sort(&mut self) {
        *self.summary_sort = self.summary_sort.next_type();
        let summary_data = self.summary_table.items.clone();
        let sorted_data = sort_table_data(summary_data, self.summary_sort);
        let selection_status = self.summary_table.state.selected();
        *self.summary_table = TableData::new(sorted_data);
        self.summary_table.state.select(selection_status);
    }

    /// If Enter is pressed on Summary page while a tag is selected
    /// go to search page and search for it
    #[cfg(not(tarpaulin_include))]
    pub fn search_tag(&mut self) {
        if let SummaryTab::Table = self.summary_tab {
            if let Some(index) = self.summary_table.state.selected() {
                let tag_name = &self.summary_table.items[index][0];
                let search_param = TxData::custom("", "", "", "", "", "", tag_name, 0);
                *self.search_data = search_param;
                self.go_search();
                self.search_tx();
            }
        }
    }

    /// Handle key press when deletion popup is turned on
    #[cfg(not(tarpaulin_include))]
    pub fn handle_deletion_popup(&mut self) {
        match self.key.code {
            KeyCode::Left | KeyCode::Right => {
                *self.deletion_status = self.deletion_status.get_next();
            }
            KeyCode::Enter => match self.deletion_status {
                DeletionStatus::Yes => match self.page {
                    CurrentUi::Home => {
                        self.home_delete_tx();
                        *self.popup = PopupState::Nothing;
                    }
                    CurrentUi::Search => {
                        self.search_delete_tx();
                        *self.popup = PopupState::Nothing;
                    }
                    _ => {}
                },
                DeletionStatus::No => *self.popup = PopupState::Nothing,
            },
            _ => {}
        }
    }

    /// Cycles through available date types
    #[cfg(not(tarpaulin_include))]
    pub fn change_search_date_type(&mut self) {
        *self.search_date_type = self.search_date_type.get_next();
        self.search_data.clear_date();
    }

    /// Start editing tx from a search result
    #[cfg(not(tarpaulin_include))]
    pub fn search_edit_tx(&mut self) {
        if let Some(a) = self.search_table.state.selected() {
            let target_data = &self.search_txs.get_tx(a);
            let target_id_num = self.search_txs.get_id_num(a);
            let tx_type = &target_data[4];

            // Based on what kind of transaction is selected, passes the tx data to the struct
            // and changes the current interface
            if tx_type == "Transfer" {
                let split_method = target_data[2].split(" to ").collect::<Vec<&str>>();
                let from_method = split_method[0];
                let to_method = split_method[1];

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
            } else {
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
            }
            self.reload_add_tx_balance_data();
        }
    }

    /// Delete a transaction from search page
    #[cfg(not(tarpaulin_include))]
    pub fn search_delete_tx(&mut self) {
        if let Some(index) = self.search_table.state.selected() {
            let status = self.search_txs.del_tx(index, self.conn);
            match status {
                Ok(()) => {
                    // Transaction deleted so reload the data again
                    self.reload_home_table();
                    self.reload_chart_data();
                    self.reload_summary_data();
                    self.reset_search_data();
                }
                Err(err) => {
                    *self.popup =
                        PopupState::DeleteFailed(TxUpdateError::FailedDeleteTx(err).to_string());
                }
            }
        }
    }

    #[cfg(not(tarpaulin_include))]
    pub fn switch_tx_index_up(&mut self) {
        if let Some(index) = self.table.state.selected() {
            // Don't do anything if there is 1 or less items or is selecting the first index which can't be moved up
            if self.table.items.len() <= 1 || index == 0 {
                return;
            }

            let selected_tx = self.all_tx_data.get_tx(index);
            let previous_tx = self.all_tx_data.get_tx(index - 1);

            if selected_tx[0] != previous_tx[0] {
                // If both are not in the same date, no switching can happen
                return;
            }

            let selected_tx_id = self.all_tx_data.get_id_num(index);
            let previous_tx_id = self.all_tx_data.get_id_num(index - 1);

            switch_tx_index(
                selected_tx_id,
                previous_tx_id,
                selected_tx,
                previous_tx,
                self.conn,
            );

            self.reload_home_table();
            self.reload_activity_table();
            self.table.state.select(Some(index - 1));
        }
    }

    #[cfg(not(tarpaulin_include))]
    pub fn switch_tx_index_down(&mut self) {
        if let Some(index) = self.table.state.selected() {
            // Don't do anything if there is 1 or less items or is selecting the last index which can't be moved up
            if self.table.items.len() <= 1 || index == self.table.items.len() - 1 {
                return;
            }

            let selected_tx = self.all_tx_data.get_tx(index);
            let next_tx = self.all_tx_data.get_tx(index + 1);

            if selected_tx[0] != next_tx[0] {
                // If both are not in the same date, no switching can happen
                return;
            }

            let selected_tx_id = self.all_tx_data.get_id_num(index);
            let next_tx_id = self.all_tx_data.get_id_num(index + 1);

            switch_tx_index(selected_tx_id, next_tx_id, selected_tx, next_tx, self.conn);

            self.reload_home_table();
            self.reload_activity_table();
            self.table.state.select(Some(index + 1));
        }
    }

    #[cfg(not(tarpaulin_include))]
    /// Opens a popup that shows the details of the selected transaction on the Homepage
    pub fn show_home_tx_details(&mut self) {
        if let Some(index) = self.table.state.selected() {
            let selected_tx = self.all_tx_data.get_tx(index);
            let tx_details = &selected_tx[1];

            *self.popup = PopupState::ShowDetails(tx_details.to_string());
        }
    }

    #[cfg(not(tarpaulin_include))]
    /// Opens a popup that shows the details of the selected activity tx details on the Activity page
    pub fn show_activity_tx_details(&mut self) {
        if let Some(index) = self.activity_table.state.selected() {
            let activity_txs = self.activity_data.get_activity_txs(Some(index));

            let mut popup_text = String::new();

            if activity_txs.len() == 2 {
                for (index, tx) in activity_txs.iter().enumerate() {
                    let tx_details = &tx[1];
                    popup_text += &format!("Transaction {}: {tx_details}", index + 1);
                    if index == 0 {
                        popup_text += "\n\n";
                    }
                }
            } else {
                let tx_details = &activity_txs[0][1];
                popup_text += &format!("Transaction 1: {tx_details}");
            }
            *self.popup = PopupState::ShowDetails(popup_text);
        }
    }

    #[cfg(not(tarpaulin_include))]
    pub fn switch_chart_tx_method_activation(&mut self) {
        if !*self.chart_hidden_mode {
            if let ChartTab::TxMethods = self.chart_tab {
                let selected_index = self.chart_tx_methods.index;
                let all_tx_methods = get_all_tx_methods_cumulative(self.conn);

                let selected_method = &all_tx_methods[selected_index];
                let activation_status = self
                    .chart_activated_methods
                    .get_mut(selected_method)
                    .unwrap();
                *activation_status = !*activation_status;
                self.lerp_state.clear();
            }
        }
    }

    #[cfg(not(tarpaulin_include))]
    pub fn popup_scroll_up(&mut self) {
        if *self.popup_scroll_position != 0 {
            *self.popup_scroll_position -= 1;
        } else {
            *self.popup_scroll_position = *self.max_popup_scroll;
        }
    }

    #[cfg(not(tarpaulin_include))]
    pub fn popup_scroll_down(&mut self) {
        if self.popup_scroll_position < self.max_popup_scroll {
            *self.popup_scroll_position += 1;
        } else {
            *self.popup_scroll_position = 0;
        }
    }
}

impl InputKeyHandler<'_> {
    /// Handle Arrow Up key press on the Homepage
    #[cfg(not(tarpaulin_include))]
    fn do_home_up(&mut self) {
        match &self.home_tab {
            HomeTab::Table => {
                // Do not select any table rows in the table section If
                // there is no transaction
                // if arrow key up is pressed and table index is 0, select the Month widget
                // else just select the upper index of the table
                if self.all_tx_data.is_tx_empty() {
                    *self.home_tab = self.home_tab.change_tab_up();
                } else if self.table.state.selected() == Some(0) {
                    *self.home_tab = HomeTab::Months;
                    self.table.state.select(None);
                } else if !self.all_tx_data.is_tx_empty() {
                    self.table.previous();
                }
            }
            HomeTab::Years => {
                // Do not select any table rows in the table section If
                // there is no transaction
                if self.all_tx_data.is_tx_empty() {
                    *self.home_tab = self.home_tab.change_tab_down();
                } else {
                    // Move to the selected value on table widget
                    // to the last row if pressed up on Year section
                    self.table.state.select(Some(self.table.items.len() - 1));
                    *self.home_tab = self.home_tab.change_tab_up();
                }
            }
            HomeTab::Months => *self.home_tab = self.home_tab.change_tab_up(),
        }
        self.reload_home_balance_data();
    }

    /// Handle Arrow Down key press on the Homepage
    #[cfg(not(tarpaulin_include))]
    fn do_home_down(&mut self) {
        match &self.home_tab {
            HomeTab::Table => {
                // Do not proceed to the table section If
                // there is no transaction
                // if arrow key down is pressed and table index is final, select the year widget
                // else just select the next index of the table
                if self.all_tx_data.is_tx_empty() {
                    *self.home_tab = self.home_tab.change_tab_down();
                } else if self.table.state.selected() == Some(self.table.items.len() - 1) {
                    *self.home_tab = HomeTab::Years;
                    self.table.state.select(None);
                } else if !self.all_tx_data.is_tx_empty() {
                    self.table.next();
                }
            }
            HomeTab::Months => {
                // Do not select any table rows in the table section If
                // there is no transaction
                if self.all_tx_data.is_tx_empty() {
                    *self.home_tab = self.home_tab.change_tab_up();
                } else {
                    *self.home_tab = self.home_tab.change_tab_down();
                    self.table.state.select(Some(0));
                }
            }
            HomeTab::Years => *self.home_tab = self.home_tab.change_tab_down(),
        }
        self.reload_home_balance_data();
    }

    /// Handle Arrow Up key press on the Summary page
    #[cfg(not(tarpaulin_include))]
    fn do_summary_up(&mut self) {
        if !*self.summary_hidden_mode {
            match self.summary_modes.index {
                0 => match self.summary_tab {
                    SummaryTab::Table => {
                        if self.summary_table.state.selected() == Some(0) {
                            *self.summary_tab = self.summary_tab.change_tab_up_monthly();
                        } else {
                            self.summary_table.previous();
                        }
                    }
                    SummaryTab::ModeSelection => {
                        if self.total_tags > 0 {
                            self.summary_table.state.select(Some(self.total_tags - 1));
                            *self.summary_tab = self.summary_tab.change_tab_up_monthly();
                        } else {
                            *self.summary_tab = self.summary_tab.change_tab_up_monthly();
                            *self.summary_tab = self.summary_tab.change_tab_up_monthly();
                            self.summary_table.state.select(None);
                        }
                    }
                    _ => *self.summary_tab = self.summary_tab.change_tab_up_monthly(),
                },
                1 => match self.summary_tab {
                    SummaryTab::Table => {
                        if self.summary_table.state.selected() == Some(0) {
                            *self.summary_tab = self.summary_tab.change_tab_up_yearly();
                        } else {
                            self.summary_table.previous();
                        }
                    }
                    SummaryTab::ModeSelection => {
                        if self.total_tags > 0 {
                            self.summary_table.state.select(Some(self.total_tags - 1));
                            *self.summary_tab = self.summary_tab.change_tab_up_yearly();
                        } else {
                            *self.summary_tab = self.summary_tab.change_tab_up_yearly();
                            *self.summary_tab = self.summary_tab.change_tab_up_yearly();
                            self.summary_table.state.select(None);
                        }
                    }
                    _ => *self.summary_tab = self.summary_tab.change_tab_up_yearly(),
                },
                2 => match self.summary_tab {
                    SummaryTab::Table => {
                        if self.summary_table.state.selected() == Some(0) {
                            *self.summary_tab = self.summary_tab.change_tab_up_all_time();
                        } else {
                            self.summary_table.previous();
                        }
                    }
                    SummaryTab::ModeSelection => {
                        if self.total_tags > 0 {
                            self.summary_table.state.select(Some(self.total_tags - 1));
                            *self.summary_tab = self.summary_tab.change_tab_up_all_time();
                        } else {
                            *self.summary_tab = self.summary_tab.change_tab_up_all_time();
                            *self.summary_tab = self.summary_tab.change_tab_up_all_time();
                            self.summary_table.state.select(None);
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
                self.summary_table.previous();
            }
        }
    }

    /// Handle Arrow Down key press on the Summary page
    #[cfg(not(tarpaulin_include))]
    fn do_summary_down(&mut self) {
        if !*self.summary_hidden_mode {
            match self.summary_modes.index {
                0 => match self.summary_tab {
                    SummaryTab::Table => {
                        if self.summary_table.state.selected() == Some(self.total_tags - 1) {
                            *self.summary_tab = self.summary_tab.change_tab_down_monthly();
                        } else {
                            self.summary_table.next();
                        }
                    }
                    SummaryTab::Months => {
                        if self.total_tags > 0 {
                            self.summary_table.state.select(Some(0));
                            *self.summary_tab = self.summary_tab.change_tab_down_monthly();
                        } else {
                            *self.summary_tab = self.summary_tab.change_tab_down_monthly();
                            *self.summary_tab = self.summary_tab.change_tab_down_monthly();
                            self.summary_table.state.select(None);
                        }
                    }
                    _ => *self.summary_tab = self.summary_tab.change_tab_down_monthly(),
                },
                1 => match self.summary_tab {
                    SummaryTab::Table => {
                        if self.summary_table.state.selected() == Some(self.total_tags - 1) {
                            *self.summary_tab = self.summary_tab.change_tab_down_yearly();
                        } else {
                            self.summary_table.next();
                        }
                    }
                    SummaryTab::Years => {
                        if self.total_tags > 0 {
                            self.summary_table.state.select(Some(0));
                            *self.summary_tab = self.summary_tab.change_tab_down_yearly();
                        } else {
                            *self.summary_tab = self.summary_tab.change_tab_down_yearly();
                            *self.summary_tab = self.summary_tab.change_tab_down_yearly();
                            self.summary_table.state.select(None);
                        }
                    }
                    _ => *self.summary_tab = self.summary_tab.change_tab_down_yearly(),
                },
                2 => match self.summary_tab {
                    SummaryTab::Table => {
                        if self.summary_table.state.selected() == Some(self.total_tags - 1) {
                            *self.summary_tab = self.summary_tab.change_tab_down_all_time();
                        } else {
                            self.summary_table.next();
                        }
                    }
                    SummaryTab::ModeSelection => {
                        if self.total_tags > 0 {
                            self.summary_table.state.select(Some(0));
                            *self.summary_tab = self.summary_tab.change_tab_down_all_time();
                        } else {
                            *self.summary_tab = self.summary_tab.change_tab_down_all_time();
                            *self.summary_tab = self.summary_tab.change_tab_down_all_time();
                            self.summary_table.state.select(None);
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
                self.summary_table.next();
            }
        }
    }

    /// Handle Arrow Up key press on the chart page
    #[cfg(not(tarpaulin_include))]
    fn do_chart_up(&mut self) {
        if !*self.chart_hidden_mode {
            match self.chart_modes.index {
                0 => *self.chart_tab = self.chart_tab.change_tab_up_monthly(),
                1 => *self.chart_tab = self.chart_tab.change_tab_up_yearly(),
                2 => *self.chart_tab = self.chart_tab.change_tab_up_all_time(),
                _ => {}
            }
        }
    }

    /// Handle Arrow Down key press on the chart page
    #[cfg(not(tarpaulin_include))]
    fn do_chart_down(&mut self) {
        if !*self.chart_hidden_mode {
            match self.chart_modes.index {
                0 => *self.chart_tab = self.chart_tab.change_tab_down_monthly(),
                1 => *self.chart_tab = self.chart_tab.change_tab_down_yearly(),
                2 => *self.chart_tab = self.chart_tab.change_tab_down_all_time(),
                _ => {}
            }
        }
    }

    /// Handle key inputs for the Details field on the Add Tx page
    #[cfg(not(tarpaulin_include))]
    fn check_add_tx_date(&mut self) {
        match self.key.code {
            KeyCode::Enter => {
                let status = self.add_tx_data.check_date(&DateType::Exact);
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
                let status = self.add_tx_data.check_date(&DateType::Exact);
                self.add_tx_data.add_tx_status(status.to_string());
                match status {
                    VerifyingOutput::Accepted(_) | VerifyingOutput::Nothing(_) => {
                        *self.add_tx_tab = TxTab::Nothing;
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Backspace => self.add_tx_data.edit_date(None),
            KeyCode::Char(a) => self.add_tx_data.edit_date(Some(a)),
            _ => {}
        }
    }

    /// Handle key inputs for the Details field on the Add Tx page
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

    /// Handle key inputs for the Tx Type field on the Add Tx page
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
                        self.reload_add_tx_balance_data();
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Esc => {
                let status = self.add_tx_data.check_tx_type();
                self.add_tx_data.add_tx_status(status.to_string());
                match status {
                    VerifyingOutput::Accepted(_) | VerifyingOutput::Nothing(_) => {
                        *self.add_tx_tab = TxTab::Nothing;
                        self.reload_add_tx_balance_data();
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Backspace => self.add_tx_data.edit_tx_type(None),
            KeyCode::Char(a) => self.add_tx_data.edit_tx_type(Some(a)),
            _ => {}
        }
    }

    /// Handle key inputs for the From Method field on the Add Tx page
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
                        *self.add_tx_tab = TxTab::Nothing;
                        self.reload_add_tx_balance_data();
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Backspace => self.add_tx_data.edit_from_method(None),
            KeyCode::Char(a) => self.add_tx_data.edit_from_method(Some(a)),
            _ => {}
        }
    }

    /// Handle key inputs for the To Method field on the Add Tx page
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
                        self.reload_add_tx_balance_data();
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Esc => {
                let status = self.add_tx_data.check_to_method(self.conn);
                self.add_tx_data.add_tx_status(status.to_string());
                match status {
                    VerifyingOutput::Accepted(_) | VerifyingOutput::Nothing(_) => {
                        *self.add_tx_tab = TxTab::Nothing;
                        self.reload_add_tx_balance_data();
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Backspace => self.add_tx_data.edit_to_method(None),
            KeyCode::Char(a) => self.add_tx_data.edit_to_method(Some(a)),
            _ => {}
        }
    }
    /// Handle key inputs for the Amount field on the Add Tx page
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
                        self.reload_add_tx_balance_data();
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
                        self.reload_add_tx_balance_data();
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Backspace => self.add_tx_data.edit_amount(None),
            KeyCode::Char(a) => self.add_tx_data.edit_amount(Some(a)),
            _ => {}
        }
    }

    /// Handle key inputs for the Tag field on the Add Tx page
    #[cfg(not(tarpaulin_include))]
    fn check_add_tx_tags(&mut self) {
        match self.key.code {
            KeyCode::Enter | KeyCode::Esc => {
                *self.add_tx_tab = TxTab::Nothing;
                self.add_tx_data.check_tags();
            }
            KeyCode::Backspace => self.add_tx_data.edit_tags(None),
            KeyCode::Char(a) => self.add_tx_data.edit_tags(Some(a)),
            _ => {}
        }
    }

    /// Handle key inputs for the Date field on the Search page
    #[cfg(not(tarpaulin_include))]
    fn check_search_date(&mut self) {
        match self.key.code {
            KeyCode::Enter => {
                let status = self.search_data.check_date(self.search_date_type);
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
                let status = self.search_data.check_date(self.search_date_type);
                self.search_data.add_tx_status(status.to_string());
                match status {
                    VerifyingOutput::Accepted(_) | VerifyingOutput::Nothing(_) => {
                        *self.search_tab = TxTab::Nothing;
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Backspace => self.search_data.edit_date(None),
            KeyCode::Char(a) => self.search_data.edit_date(Some(a)),
            _ => {}
        }
    }

    /// Handle key inputs for the Details field on the Search page
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

    /// Handle key inputs for the Tx Type field on the Search page
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
                        *self.search_tab = TxTab::Nothing;
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Backspace => self.search_data.edit_tx_type(None),
            KeyCode::Char(a) => self.search_data.edit_tx_type(Some(a)),
            _ => {}
        }
    }

    /// Handle key inputs for the From Method field on the Search page
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
                        *self.search_tab = TxTab::Nothing;
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Backspace => self.search_data.edit_from_method(None),
            KeyCode::Char(a) => self.search_data.edit_from_method(Some(a)),
            _ => {}
        }
    }

    /// Handle key inputs for the To Method field on the Search page
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
                        *self.search_tab = TxTab::Nothing;
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Backspace => self.search_data.edit_to_method(None),
            KeyCode::Char(a) => self.search_data.edit_to_method(Some(a)),
            _ => {}
        }
    }

    /// Handle key inputs for the amount field on the Search page
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
                        *self.search_tab = TxTab::Nothing;
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Backspace => self.search_data.edit_amount(None),
            KeyCode::Char(a) => self.search_data.edit_amount(Some(a)),
            _ => {}
        }
    }

    /// Handle key inputs for the Tag field on the Search page
    #[cfg(not(tarpaulin_include))]
    fn check_search_tags(&mut self) {
        match self.key.code {
            KeyCode::Enter | KeyCode::Esc => {
                let status = self.search_data.check_tags_forced(self.conn);
                self.search_data.add_tx_status(status.to_string());
                match status {
                    VerifyingOutput::Accepted(_) | VerifyingOutput::Nothing(_) => {
                        *self.search_tab = TxTab::Nothing;
                    }
                    VerifyingOutput::NotAccepted(_) => {}
                }
            }
            KeyCode::Backspace => self.search_data.edit_tags(None),
            KeyCode::Char(a) => self.search_data.edit_tags(Some(a)),
            _ => {}
        }
    }

    /// Reload Home page's table data by fetching from the DB
    #[cfg(not(tarpaulin_include))]
    fn reload_home_table(&mut self) {
        *self.all_tx_data =
            TransactionData::new(self.home_months.index, self.home_years.index, self.conn);
        *self.table = TableData::new(self.all_tx_data.get_txs());
        self.reload_home_balance_data();
    }

    /// Reset summary table data by recreating it from gathered Summary Data
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

    /// Reload summary data by fetching from the DB
    #[cfg(not(tarpaulin_include))]
    fn reload_summary_data(&mut self) {
        *self.summary_data = SummaryData::new(self.conn);
    }

    /// Reload chart data by fetching from the DB
    #[cfg(not(tarpaulin_include))]
    fn reload_chart_data(&mut self) {
        *self.chart_data = ChartData::new(self.conn);
    }

    /// Reset all currently shown search related data to nothing
    #[cfg(not(tarpaulin_include))]
    fn reset_search_data(&mut self) {
        *self.search_table = TableData::new(Vec::new());
        *self.search_txs = TransactionData::new_search(Vec::new(), Vec::new());
    }

    /// Reload activity data by fetching from the DB
    #[cfg(not(tarpaulin_include))]
    fn reload_activity_table(&mut self) {
        *self.activity_data = ActivityData::new(
            self.activity_months.index,
            self.activity_years.index,
            self.conn,
        );
        *self.activity_table = TableData::new(self.activity_data.get_txs());
    }

    /// Move the cursor for text fields to the correct position, if it's misplaced
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
            TxTab::Date => self.add_tx_data.do_date_up(&DateType::Exact),
            TxTab::FromMethod => self.add_tx_data.do_from_method_up(self.conn),
            TxTab::ToMethod => self.add_tx_data.do_to_method_up(self.conn),
            TxTab::Amount => self.add_tx_data.do_amount_up(false, self.conn),
            TxTab::TxType => self.add_tx_data.do_tx_type_up(),
            TxTab::Tags => self.add_tx_data.do_tags_up(self.conn),
            _ => Ok(()),
        };

        if let Err(e) = status {
            self.add_tx_data.add_tx_status(e.to_string());
        }
    }

    #[cfg(not(tarpaulin_include))]
    fn do_add_tx_down(&mut self) {
        let status = match self.add_tx_tab {
            TxTab::Date => self.add_tx_data.do_date_down(&DateType::Exact),
            TxTab::FromMethod => self.add_tx_data.do_from_method_down(self.conn),
            TxTab::ToMethod => self.add_tx_data.do_to_method_down(self.conn),
            TxTab::Amount => self.add_tx_data.do_amount_down(false, self.conn),
            TxTab::TxType => self.add_tx_data.do_tx_type_down(),
            TxTab::Tags => self.add_tx_data.do_tags_down(self.conn),
            _ => Ok(()),
        };

        if let Err(e) = status {
            self.add_tx_data.add_tx_status(e.to_string());
        }
    }

    #[cfg(not(tarpaulin_include))]
    fn do_search_up(&mut self) {
        let status = match self.search_tab {
            TxTab::Date => self.search_data.do_date_up(self.search_date_type),
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
                } else if !self.search_txs.is_tx_empty() {
                    self.search_table.previous();
                }
                Ok(())
            }
            TxTab::Details => Ok(()),
        };

        if let Err(e) = status {
            self.search_data.add_tx_status(e.to_string());
        }
    }

    #[cfg(not(tarpaulin_include))]
    fn do_search_down(&mut self) {
        let status = match self.search_tab {
            TxTab::Date => self.search_data.do_date_down(self.search_date_type),
            TxTab::FromMethod => self.search_data.do_from_method_down(self.conn),
            TxTab::ToMethod => self.search_data.do_to_method_down(self.conn),
            TxTab::Amount => self.search_data.do_amount_down(true, self.conn),
            TxTab::TxType => self.search_data.do_tx_type_down(),
            TxTab::Tags => self.search_data.do_tags_down(self.conn),
            TxTab::Nothing => {
                if self.search_table.state.selected() == Some(self.search_table.items.len() - 1) {
                    self.search_table.state.select(Some(0));
                } else if !self.search_txs.is_tx_empty() {
                    self.search_table.next();
                }
                Ok(())
            }
            TxTab::Details => Ok(()),
        };

        if let Err(e) = status {
            self.search_data.add_tx_status(e.to_string());
        }
    }

    #[cfg(not(tarpaulin_include))]
    fn do_activity_up(&mut self) {
        match self.activity_tab {
            ActivityTab::Years => {
                if self.activity_data.is_activity_empty() {
                    *self.activity_tab = self.activity_tab.change_tab_down();
                } else {
                    *self.activity_tab = self.activity_tab.change_tab_up();
                    self.activity_table
                        .state
                        .select(Some(self.activity_table.items.len() - 1));
                }
            }
            ActivityTab::Months => {
                *self.activity_tab = self.activity_tab.change_tab_up();
            }
            ActivityTab::List => {
                if self.activity_table.state.selected() == Some(0) {
                    self.activity_table.state.select(None);
                    *self.activity_tab = self.activity_tab.change_tab_up();
                } else {
                    self.activity_table.previous();
                }
            }
        }
    }

    #[cfg(not(tarpaulin_include))]
    fn do_activity_down(&mut self) {
        match self.activity_tab {
            ActivityTab::Years => {
                *self.activity_tab = self.activity_tab.change_tab_down();
            }
            ActivityTab::Months => {
                if self.activity_data.is_activity_empty() {
                    *self.activity_tab = self.activity_tab.change_tab_up();
                } else {
                    *self.activity_tab = self.activity_tab.change_tab_down();
                    self.activity_table.state.select(Some(0));
                }
            }
            ActivityTab::List => {
                if self.activity_table.state.selected() == Some(self.activity_table.items.len() - 1)
                {
                    *self.activity_tab = self.activity_tab.change_tab_down();
                    self.activity_table.state.select(None);
                } else {
                    self.activity_table.next();
                }
            }
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

    /// Reset scroll position to 0
    #[cfg(not(tarpaulin_include))]
    fn reload_popup_scroll_position(&mut self) {
        *self.popup_scroll_position = 0;
        *self.max_popup_scroll = 0;
    }

    /// Update add home page balance section data that is being shown on the UI
    #[cfg(not(tarpaulin_include))]
    fn reload_home_balance_data(&mut self) {
        let mut balance_data = vec![vec![String::new()]];
        balance_data[0].extend(get_all_tx_methods(self.conn));
        balance_data[0].extend(vec!["Total".to_string()]);

        let current_table_index = self.table.state.selected();

        match current_table_index {
            // Pass out the current index to get the necessary balance & changes data
            Some(a) => {
                balance_data.push(self.all_tx_data.get_balance(a));
                balance_data.push(self.all_tx_data.get_changes(a));
            }
            // If none selected, get empty changes + the absolute final balance
            None => {
                balance_data.push(self.all_tx_data.get_last_balance(self.conn));
                balance_data.push(get_empty_changes(self.conn));
            }
        }

        // Total income, total expense, daily income, daily expense data based on the selected index.
        balance_data.push(
            self.all_tx_data
                .get_total_income(current_table_index, self.conn),
        );
        balance_data.push(
            self.all_tx_data
                .get_total_expense(current_table_index, self.conn),
        );
        balance_data.push(
            self.all_tx_data
                .get_daily_income(current_table_index, self.conn),
        );
        balance_data.push(
            self.all_tx_data
                .get_daily_expense(current_table_index, self.conn),
        );

        *self.balance_data = balance_data;
    }

    /// Update add tx page balance section data that is being shown on the UI
    #[cfg(not(tarpaulin_include))]
    fn reload_add_tx_balance_data(&mut self) {
        let mut balance_data = vec![vec![String::new()]];
        balance_data[0].extend(get_all_tx_methods(self.conn));
        balance_data[0].extend(vec!["Total".to_string()]);

        let current_table_index = self.table.state.selected();

        let (current_balance, current_changes) = match current_table_index {
            // Pass out the current index to get the necessary balance & changes data
            Some(a) => (
                self.all_tx_data.get_balance(a),
                self.all_tx_data.get_changes(a),
            ),
            // If none selected, get empty changes + the absolute final balance
            None => (
                self.all_tx_data.get_last_balance(self.conn),
                get_empty_changes(self.conn),
            ),
        };

        balance_data.push(self.add_tx_data.generate_balance_section(
            self.conn,
            current_balance,
            current_changes,
        ));
        balance_data.push(self.add_tx_data.generate_changes_section(self.conn));

        *self.balance_data = balance_data;
    }
}
