use chrono::Datelike;
use chrono::prelude::Local;
use ratatui::widgets::TableState;
use rusqlite::Connection;
use std::fmt::{self, Display, Result};
use std::path::PathBuf;

use crate::db::{MODES, MONTHS, YEARS};
use crate::utility::{get_all_tx_methods, get_all_tx_methods_cumulative};

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
    /// State is the library default.
    #[must_use]
    pub fn new(data: Vec<Vec<String>>) -> Self {
        TableData {
            state: TableState::default(),
            items: data,
        }
    }

    /// Adds 1 to the current index. If at the final value, goes to 0
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
#[derive(Clone)]
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

    pub fn new_monthly_no_local() -> Self {
        IndexedData {
            titles: MONTHS.into_iter().map(ToString::to_string).collect(),
            index: 0,
        }
    }

    pub fn new_yearly() -> Self {
        let year_index = Local::now().year() as usize - 2022;
        IndexedData {
            titles: YEARS.into_iter().map(ToString::to_string).collect(),
            index: year_index,
        }
    }

    pub fn new_yearly_no_local() -> Self {
        IndexedData {
            titles: YEARS.into_iter().map(ToString::to_string).collect(),
            index: 0,
        }
    }

    pub fn new_modes() -> Self {
        IndexedData {
            titles: MODES.into_iter().map(ToString::to_string).collect(),
            index: 0,
        }
    }

    pub fn new_tx_methods(conn: &Connection) -> Self {
        IndexedData {
            titles: get_all_tx_methods(conn),
            index: 0,
        }
    }

    pub fn new_tx_methods_cumulative(conn: &Connection) -> Self {
        IndexedData {
            titles: get_all_tx_methods_cumulative(conn),
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

    pub fn get_selected_value(&self) -> &str {
        &self.titles[self.index]
    }
}

/// The enum is used to keep track of which tab is currently set at active
/// or being interacted with in the Homepage. There are 3 interact-able widgets
/// in the homepage thus three values. The goal is to keep them cycling through
/// all values.
pub enum HomeTab {
    Years,
    Months,
    Table,
}

impl HomeTab {
    /// Moves the current selected tab to the upper value. If at the 1st value, the final value is selected.
    pub fn change_tab_up(&mut self) -> Self {
        match &self {
            HomeTab::Years => HomeTab::Table,
            HomeTab::Months => HomeTab::Years,
            HomeTab::Table => HomeTab::Months,
        }
    }

    /// Moves the current selected tab to the bottom value. If at the last value, the 1st value is selected.
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
    Activity,
}

/// Indicates which pop up is currently on and is being shown in the screen
pub enum PopupState {
    NewUpdate(Vec<String>),
    HomeHelp,
    AddTxHelp,
    ChartHelp,
    SummaryHelp,
    SearchHelp,
    ActivityHelp,
    DeleteFailed(String),
    TxDeletion,
    ShowDetails(String),
    Nothing,
}

pub enum ChartTab {
    ModeSelection,
    Years,
    Months,
    TxMethods,
}

impl ChartTab {
    /// Moves the current selected tab to the upper value. If at the 1st value, the final value is selected.
    pub fn change_tab_up_monthly(&mut self) -> Self {
        match &self {
            ChartTab::ModeSelection => ChartTab::TxMethods,
            ChartTab::Years => ChartTab::ModeSelection,
            ChartTab::Months => ChartTab::Years,
            ChartTab::TxMethods => ChartTab::Months,
        }
    }

    /// Moves the current selected tab to the bottom value. If at the last value, the 1st value is selected.
    pub fn change_tab_down_monthly(&mut self) -> Self {
        match &self {
            ChartTab::ModeSelection => ChartTab::Years,
            ChartTab::Years => ChartTab::Months,
            ChartTab::Months => ChartTab::TxMethods,
            ChartTab::TxMethods => ChartTab::ModeSelection,
        }
    }

    /// Moves the current selected tab to the upper value. If at the 1st value, the final value is selected.
    pub fn change_tab_up_yearly(&mut self) -> Self {
        match &self {
            ChartTab::ModeSelection => ChartTab::TxMethods,
            ChartTab::Years => ChartTab::ModeSelection,
            ChartTab::TxMethods => ChartTab::Years,
            ChartTab::Months => unreachable!(),
        }
    }

    /// Moves the current selected tab to the bottom value. If at the last value, the 1st value is selected.
    pub fn change_tab_down_yearly(&mut self) -> Self {
        match &self {
            ChartTab::ModeSelection => ChartTab::Years,
            ChartTab::Years => ChartTab::TxMethods,
            ChartTab::TxMethods => ChartTab::ModeSelection,
            ChartTab::Months => unreachable!(),
        }
    }

    /// Moves the current selected tab to the upper value. If at the 1st value, the final value is selected.
    pub fn change_tab_up_all_time(&mut self) -> Self {
        match &self {
            ChartTab::ModeSelection => ChartTab::TxMethods,
            ChartTab::TxMethods => ChartTab::ModeSelection,
            ChartTab::Months | ChartTab::Years => unreachable!(),
        }
    }

    /// Moves the current selected tab to the bottom value. If at the last value, the 1st value is selected.
    pub fn change_tab_down_all_time(&mut self) -> Self {
        match &self {
            ChartTab::ModeSelection => ChartTab::TxMethods,
            ChartTab::TxMethods => ChartTab::ModeSelection,
            ChartTab::Months | ChartTab::Years => unreachable!(),
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
    /// Moves the current selected tab to the upper value. If at the 1st value, the final value is selected.
    pub fn change_tab_up_monthly(&mut self) -> Self {
        match &self {
            SummaryTab::ModeSelection => SummaryTab::Table,
            SummaryTab::Years => SummaryTab::ModeSelection,
            SummaryTab::Months => SummaryTab::Years,
            SummaryTab::Table => SummaryTab::Months,
        }
    }

    /// Moves the current selected tab to the bottom value. If at the last value, the 1st value is selected.
    pub fn change_tab_down_monthly(&mut self) -> Self {
        match &self {
            SummaryTab::ModeSelection => SummaryTab::Years,
            SummaryTab::Years => SummaryTab::Months,
            SummaryTab::Months => SummaryTab::Table,
            SummaryTab::Table => SummaryTab::ModeSelection,
        }
    }

    /// Moves the current selected tab to the upper value. If at the 1st value, the final value is selected.
    pub fn change_tab_up_yearly(&mut self) -> Self {
        match &self {
            SummaryTab::ModeSelection => SummaryTab::Table,
            SummaryTab::Years => SummaryTab::ModeSelection,
            SummaryTab::Table => SummaryTab::Years,
            SummaryTab::Months => SummaryTab::Months,
        }
    }

    /// Moves the current selected tab to the bottom value. If at the last value, the 1st value is selected.
    pub fn change_tab_down_yearly(&mut self) -> Self {
        match &self {
            SummaryTab::ModeSelection => SummaryTab::Years,
            SummaryTab::Years => SummaryTab::Table,
            SummaryTab::Table => SummaryTab::ModeSelection,
            SummaryTab::Months => SummaryTab::Months,
        }
    }

    /// Moves the current selected tab to the upper value. If at the 1st value, the final value is selected.
    pub fn change_tab_up_all_time(&mut self) -> Self {
        match &self {
            SummaryTab::ModeSelection => SummaryTab::Table,
            SummaryTab::Table => SummaryTab::ModeSelection,
            SummaryTab::Years => SummaryTab::Years,
            SummaryTab::Months => SummaryTab::Months,
        }
    }

    /// Moves the current selected tab to the bottom value. If at the last value, the 1st value is selected.
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
    #[must_use]
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
    pub fn get_next(&mut self) -> Self {
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
    pub fn get_next(&mut self) -> Self {
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
    DailyIncome,
    DailyExpense,
    TopRow,
}

impl Display for HomeRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result {
        match self {
            HomeRow::Balance => write!(f, "Balance"),
            HomeRow::Changes => write!(f, "Changes"),
            HomeRow::Income => write!(f, "Income"),
            HomeRow::Expense => write!(f, "Expense"),
            HomeRow::DailyIncome => write!(f, "DailyIncome"),
            HomeRow::DailyExpense => write!(f, "DailyExpense"),
            HomeRow::TopRow => write!(f, "TopRow"),
        }
    }
}

impl HomeRow {
    #[must_use]
    pub fn get_row(data: &[String]) -> Self {
        if data[0] == "Balance" {
            HomeRow::Balance
        } else if data[0] == "Changes" {
            HomeRow::Changes
        } else if data[0] == "Income" {
            HomeRow::Income
        } else if data[0] == "Expense" {
            HomeRow::Expense
        } else if data[0] == "Daily Income" {
            HomeRow::DailyIncome
        } else if data[0] == "Daily Expense" {
            HomeRow::DailyExpense
        } else {
            HomeRow::TopRow
        }
    }
}

pub enum ActivityTab {
    Years,
    Months,
    List,
}

impl ActivityTab {
    pub fn change_tab_up(&mut self) -> Self {
        match &self {
            ActivityTab::Years => ActivityTab::List,
            ActivityTab::Months => ActivityTab::Years,
            ActivityTab::List => ActivityTab::Months,
        }
    }

    pub fn change_tab_down(&mut self) -> Self {
        match &self {
            ActivityTab::List => ActivityTab::Years,
            ActivityTab::Years => ActivityTab::Months,
            ActivityTab::Months => ActivityTab::List,
        }
    }
}

pub enum ActivityType {
    NewTX,
    EditTX(Option<i32>),
    DeleteTX(Option<i32>),
    IDNumSwap(Option<i32>, Option<i32>),
    SearchTX(Option<u8>),
}

impl ActivityType {
    #[must_use]
    pub fn from_s(data: &str) -> Self {
        match data {
            "Add TX" => Self::NewTX,
            "Edit TX" => Self::EditTX(None),
            "Delete TX" => Self::DeleteTX(None),
            "TX Position Swap" => Self::IDNumSwap(None, None),
            "Search TX" => Self::SearchTX(None),
            _ => unreachable!(),
        }
    }

    #[must_use]
    pub fn to_details(&self) -> String {
        match self {
            Self::NewTX => String::from("A new Transaction was added"),
            Self::EditTX(id) => format!("A transaction was edited with ID {}", id.unwrap()),
            Self::DeleteTX(id) => format!("A transaction was deleted with ID {}", id.unwrap()),
            Self::IDNumSwap(id_1, id_2) => format!(
                "Transaction with ID num {} and ID num {} was swapped",
                id_1.unwrap(),
                id_2.unwrap()
            ),
            Self::SearchTX(total) => {
                if total.unwrap() == 1 {
                    String::from("Transactions were searched with one field")
                } else {
                    String::from("Transactions were searched with multiple fields")
                }
            }
        }
    }

    #[must_use]
    pub fn to_str(&self) -> String {
        match self {
            Self::NewTX => String::from("Add TX"),
            Self::EditTX(_) => String::from("Edit TX"),
            Self::DeleteTX(_) => String::from("Delete TX"),
            Self::IDNumSwap(_, _) => String::from("TX Position Swap"),
            Self::SearchTX(_) => String::from("Search TX"),
        }
    }
}
