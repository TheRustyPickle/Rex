use chrono::prelude::Local;
use chrono::Datelike;
use ratatui::widgets::TableState;
use std::path::PathBuf;

use crate::db::{MODES, MONTHS, YEARS};

/// The struct stores all transaction data for the Transaction widget
/// and creates an index to keep track of which transactions row is selected
/// if any. Each vec inside the vec of items contains 1 full transaction.
///
/// state : `None` or an index
/// items : `[["2022-05-01", "test", "source_1", "15.50", Expense], ]`
pub struct TableData {
    pub state: TableState,
    pub items: Vec<Vec<String>>,
}

impl TableData {
    /// Creates the default table state and adds the manual transaction data
    /// that was passed to it as an argument to consider them as a value of an index.
    /// state is the library default.
    pub fn new(data: Vec<Vec<String>>) -> Self {
        TableData {
            state: TableState::default(),
            items: data,
        }
    }

    /// Adds 1 to the current index. if at the final value, goes to 0
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    /// Removes 1 from the current index. If index at 0, goes to the final index
    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

/// This struct takes anything inside a vector and adds an index it it.
/// It is used for keeping track of the Months and Years current index.
///
/// titles: `["January", "February",]`

pub struct IndexedData {
    pub titles: Vec<String>,
    pub index: usize,
}

impl IndexedData {
    pub fn new_monthly() -> Self {
        let month_index = Local::now().month() as usize - 1;
        IndexedData {
            titles: MONTHS.into_iter().map(ToString::to_string).collect(),
            index: month_index,
        }
    }

    pub fn new_yearly() -> Self {
        let year_index = Local::now().year() as usize - 2022;
        IndexedData {
            titles: YEARS.into_iter().map(ToString::to_string).collect(),
            index: year_index,
        }
    }

    pub fn new_modes() -> Self {
        IndexedData {
            titles: MODES.into_iter().map(ToString::to_string).collect(),
            index: 0,
        }
    }

    /// Increases the current index by 1 or goes to 0 if at the final value
    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }

    /// Decreases the current index by 1 or goes to final index if at 0
    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.titles.len() - 1;
        }
    }

    pub fn set_index_zero(&mut self) {
        self.index = 0;
    }
}

/// The enum is used to keep track of which tab is currently set at active
/// or being interacted with in the Home page. There are 3 interact-able widgets
/// in the home page thus three values. The goal is to keep them cycling through
/// all values.
pub enum HomeTab {
    Years,
    Months,
    Table,
}

impl HomeTab {
    /// Moves the current selected tab to the upper value. If at the 1st value, the
    /// the final value is selected.
    #[cfg(not(tarpaulin_include))]
    pub fn change_tab_up(&mut self) -> Self {
        match &self {
            HomeTab::Years => HomeTab::Table,
            HomeTab::Months => HomeTab::Years,
            HomeTab::Table => HomeTab::Months,
        }
    }

    /// Moves the current selected tab to the bottom value. If at the last value, the
    /// the 1st value is selected.
    #[cfg(not(tarpaulin_include))]
    pub fn change_tab_down(&mut self) -> Self {
        match &self {
            HomeTab::Years => HomeTab::Months,
            HomeTab::Months => HomeTab::Table,
            HomeTab::Table => HomeTab::Years,
        }
    }
}

/// This enum is used inside the Add Transaction page.
/// This is targeted to be used to keep track which widget of the Add Transaction
/// is currently being interacted with.
pub enum TxTab {
    Date,
    Details,
    FromMethod,
    ToMethod,
    Amount,
    TxType,
    Tags,
    Nothing,
}

/// Shows the currently active page in the terminal. Used to properly
/// direct key presses to the relevant structs and widget selection.
pub enum CurrentUi {
    Initial,
    Home,
    AddTx,
    Chart,
    Summary,
    Search,
    History,
}

/// Indicates which popup is currently on and is being shown in the screen
pub enum PopupState {
    NewUpdate(Vec<String>),
    HomeHelp,
    AddTxHelp,
    ChartHelp,
    SummaryHelp,
    SearchHelp,
    DeleteFailed(String),
    TxDeletion,
    Nothing,
}

pub enum ChartTab {
    ModeSelection,
    Years,
    Months,
}

impl ChartTab {
    /// Moves the current selected tab to the upper value. If at the 1st value, the
    /// the final value is selected.
    #[cfg(not(tarpaulin_include))]
    pub fn change_tab_up_monthly(&mut self) -> Self {
        match &self {
            ChartTab::ModeSelection => ChartTab::Months,
            ChartTab::Years => ChartTab::ModeSelection,
            ChartTab::Months => ChartTab::Years,
        }
    }

    /// Moves the current selected tab to the bottom value. If at the last value, the
    /// the 1st value is selected.
    #[cfg(not(tarpaulin_include))]
    pub fn change_tab_down_monthly(&mut self) -> Self {
        match &self {
            ChartTab::ModeSelection => ChartTab::Years,
            ChartTab::Years => ChartTab::Months,
            ChartTab::Months => ChartTab::ModeSelection,
        }
    }

    /// Moves the current selected tab to the upper value. If at the 1st value, the
    /// the final value is selected.
    #[cfg(not(tarpaulin_include))]
    pub fn change_tab_up_yearly(&mut self) -> Self {
        match &self {
            ChartTab::ModeSelection => ChartTab::Years,
            ChartTab::Years => ChartTab::ModeSelection,
            ChartTab::Months => ChartTab::Months,
        }
    }

    /// Moves the current selected tab to the bottom value. If at the last value, the
    /// the 1st value is selected.
    #[cfg(not(tarpaulin_include))]
    pub fn change_tab_down_yearly(&mut self) -> Self {
        match &self {
            ChartTab::ModeSelection => ChartTab::Years,
            ChartTab::Years => ChartTab::ModeSelection,
            ChartTab::Months => ChartTab::Months,
        }
    }
}

pub enum SummaryTab {
    ModeSelection,
    Years,
    Months,
    Table,
}

impl SummaryTab {
    /// Moves the current selected tab to the upper value. If at the 1st value, the
    /// the final value is selected.
    #[cfg(not(tarpaulin_include))]
    pub fn change_tab_up_monthly(&mut self) -> Self {
        match &self {
            SummaryTab::ModeSelection => SummaryTab::Table,
            SummaryTab::Years => SummaryTab::ModeSelection,
            SummaryTab::Months => SummaryTab::Years,
            SummaryTab::Table => SummaryTab::Months,
        }
    }

    /// Moves the current selected tab to the bottom value. If at the last value, the
    /// the 1st value is selected.
    #[cfg(not(tarpaulin_include))]
    pub fn change_tab_down_monthly(&mut self) -> Self {
        match &self {
            SummaryTab::ModeSelection => SummaryTab::Years,
            SummaryTab::Years => SummaryTab::Months,
            SummaryTab::Months => SummaryTab::Table,
            SummaryTab::Table => SummaryTab::ModeSelection,
        }
    }

    /// Moves the current selected tab to the upper value. If at the 1st value, the
    /// the final value is selected.
    #[cfg(not(tarpaulin_include))]
    pub fn change_tab_up_yearly(&mut self) -> Self {
        match &self {
            SummaryTab::ModeSelection => SummaryTab::Table,
            SummaryTab::Years => SummaryTab::ModeSelection,
            SummaryTab::Table => SummaryTab::Years,
            SummaryTab::Months => SummaryTab::Months,
        }
    }

    /// Moves the current selected tab to the bottom value. If at the last value, the
    /// the 1st value is selected.
    #[cfg(not(tarpaulin_include))]
    pub fn change_tab_down_yearly(&mut self) -> Self {
        match &self {
            SummaryTab::ModeSelection => SummaryTab::Years,
            SummaryTab::Years => SummaryTab::Table,
            SummaryTab::Table => SummaryTab::ModeSelection,
            SummaryTab::Months => SummaryTab::Months,
        }
    }

    /// Moves the current selected tab to the upper value. If at the 1st value, the
    /// the final value is selected.
    #[cfg(not(tarpaulin_include))]
    pub fn change_tab_up_all_time(&mut self) -> Self {
        match &self {
            SummaryTab::ModeSelection => SummaryTab::Table,
            SummaryTab::Table => SummaryTab::ModeSelection,
            SummaryTab::Years => SummaryTab::Years,
            SummaryTab::Months => SummaryTab::Months,
        }
    }

    /// Moves the current selected tab to the bottom value. If at the last value, the
    /// the 1st value is selected.
    #[cfg(not(tarpaulin_include))]
    pub fn change_tab_down_all_time(&mut self) -> Self {
        match &self {
            SummaryTab::ModeSelection => SummaryTab::Table,
            SummaryTab::Table => SummaryTab::ModeSelection,
            SummaryTab::Years => SummaryTab::Years,
            SummaryTab::Months => SummaryTab::Months,
        }
    }
}

pub enum UserInputType {
    AddNewTxMethod(Vec<String>),
    RenameTxMethod(Vec<String>),
    RepositionTxMethod(Vec<String>),
    SetNewLocation(PathBuf),
    CancelledOperation,
    ResetData(ResetType),
    BackupDBPath(Vec<PathBuf>),
    InvalidInput,
}

pub enum ResetType {
    NewLocation,
    BackupDB,
}

impl UserInputType {
    #[cfg(not(tarpaulin_include))]
    pub fn from_string(input: &str) -> Self {
        match input {
            "1" => UserInputType::AddNewTxMethod(Vec::new()),
            "2" => UserInputType::RenameTxMethod(Vec::new()),
            "3" => UserInputType::RepositionTxMethod(Vec::new()),
            "4" => UserInputType::SetNewLocation(PathBuf::new()),
            "5" => UserInputType::BackupDBPath(Vec::new()),
            "cancel" => UserInputType::CancelledOperation,
            _ => UserInputType::InvalidInput,
        }
    }
}

pub enum SortingType {
    ByTags,
    ByIncome,
    ByExpense,
}

impl SortingType {
    #[cfg(not(tarpaulin_include))]
    pub fn next_type(&mut self) -> Self {
        match self {
            SortingType::ByTags => SortingType::ByIncome,
            SortingType::ByIncome => SortingType::ByExpense,
            SortingType::ByExpense => SortingType::ByTags,
        }
    }
}

pub enum DeletionStatus {
    Yes,
    No,
}

impl DeletionStatus {
    #[cfg(not(tarpaulin_include))]
    pub fn next(&mut self) -> Self {
        match self {
            DeletionStatus::Yes => DeletionStatus::No,
            DeletionStatus::No => DeletionStatus::Yes,
        }
    }
}

pub enum DateType {
    Exact,
    Monthly,
    Yearly,
}

impl DateType {
    #[cfg(not(tarpaulin_include))]
    pub fn next(&mut self) -> Self {
        match self {
            DateType::Exact => DateType::Monthly,
            DateType::Monthly => DateType::Yearly,
            DateType::Yearly => DateType::Exact,
        }
    }
}

#[derive(PartialEq)]
pub enum HomeRow {
    Balance,
    Changes,
    Income,
    Expense,
    TopRow,
}

impl HomeRow {
    #[cfg(not(tarpaulin_include))]
    pub fn get_row(data: &[String]) -> Self {
        if data[0] == "Balance" {
            HomeRow::Balance
        } else if data[0] == "Changes" {
            HomeRow::Changes
        } else if data[0] == "Income" {
            HomeRow::Income
        } else if data[0] == "Expense" {
            HomeRow::Expense
        } else {
            HomeRow::TopRow
        }
    }
}

pub enum HistoryTab {
    Years,
    Months,
    List,
}

impl HistoryTab {
    #[cfg(not(tarpaulin_include))]
    pub fn change_tab_up(&mut self) -> Self {
        match &self {
            HistoryTab::Years => HistoryTab::List,
            HistoryTab::Months => HistoryTab::Years,
            HistoryTab::List => HistoryTab::Months,
        }
    }

    #[cfg(not(tarpaulin_include))]
    pub fn change_tab_down(&mut self) -> Self {
        match &self {
            HistoryTab::List => HistoryTab::Years,
            HistoryTab::Years => HistoryTab::Months,
            HistoryTab::Months => HistoryTab::List,
        }
    }
}

pub enum ActivityType {
    NewTX,
    EditTX,
    DeleteTX,
    IDNumSwap,
    SearchTX,
}
