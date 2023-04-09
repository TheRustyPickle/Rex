use chrono::{prelude::Local, Datelike};
use tui::widgets::TableState;

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
    pub fn new(titles: Vec<String>) -> Self {
        IndexedData { titles, index: 0 }
    }

    pub fn new_monthly() -> Self {
        let month_index = Local::now().month() as usize - 1;
        IndexedData {
            titles: vec![
                "January",
                "February",
                "March",
                "April",
                "May",
                "June",
                "July",
                "August",
                "September",
                "October",
                "November",
                "December",
            ]
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
            index: month_index,
        }
    }

    pub fn new_yearly() -> Self {
        let year_index = Local::now().year() as usize - 2022;
        IndexedData {
            titles: vec!["2022", "2023", "2024", "2025"]
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
            index: year_index,
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
    pub fn change_tab_up(&mut self) -> Self {
        match &self {
            HomeTab::Years => HomeTab::Table,
            HomeTab::Months => HomeTab::Years,
            HomeTab::Table => HomeTab::Months,
        }
    }

    /// Moves the current selected tab to the bottom value. If at the last value, the
    /// the 1st value is selected.
    pub fn change_tab_down(&mut self) -> Self {
        match &self {
            HomeTab::Years => HomeTab::Months,
            HomeTab::Months => HomeTab::Table,
            HomeTab::Table => HomeTab::Years,
        }
    }
}

/// This enum is used inside the Add Transaction page.
/// This is targeted to be used to keep track which field of the Add Transaction widgets
/// is currently being interacted with. Based on which one is selected, each keypress is
/// recorded and added to the relevant struct.
pub enum AddTxTab {
    Date,
    Details,
    TxMethod,
    Amount,
    TxType,
    Tags,
    Nothing,
}

/// This enum is used inside the Transfer page.
/// This is targeted to be used to keep track which field of the Transfer widgets
/// is currently being interacted with. Based on which one is selected, each keypress is
/// recorded and added to the relevant struct.
pub enum TransferTab {
    Date,
    Details,
    From,
    To,
    Amount,
    Tags,
    Nothing,
}

/// Shows the currently active page in the terminal. Used to properly
/// direct key presses to the relevant structs and widget selection.
pub enum CurrentUi {
    Initial,
    Home,
    AddTx,
    Transfer,
    Chart,
    Summary,
}

/// Indicates which popup is currently on and is being shown in the screen
pub enum PopupState {
    NewUpdate,
    Helper,
    DeleteFailed(String),
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
    pub fn change_tab_up(&mut self) -> Self {
        match &self {
            ChartTab::ModeSelection => ChartTab::Months,
            ChartTab::Years => ChartTab::ModeSelection,
            ChartTab::Months => ChartTab::Years,
        }
    }

    /// Moves the current selected tab to the bottom value. If at the last value, the
    /// the 1st value is selected.
    pub fn change_tab_down(&mut self) -> Self {
        match &self {
            ChartTab::ModeSelection => ChartTab::Years,
            ChartTab::Years => ChartTab::Months,
            ChartTab::Months => ChartTab::ModeSelection,
        }
    }
}
