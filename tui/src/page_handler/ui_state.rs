use chrono::Datelike;
use chrono::prelude::Local;
use ratatui::widgets::TableState;
use rex_app::conn::DbConn;
use strum_macros::Display;

pub const MONTHS: [&str; 12] = [
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
];

pub const YEARS: [&str; 16] = [
    "2022", "2023", "2024", "2025", "2026", "2027", "2028", "2029", "2030", "2031", "2032", "2033",
    "2034", "2035", "2036", "2037",
];

pub const MODES: [&str; 3] = ["Monthly", "Yearly", "All Time"];

/// The rows of a transaction table, each row holding the already formatted
/// columns of one transaction, plus the state ratatui needs to know which row
/// is selected and where the viewport starts.
///
/// state: `None` when nothing is selected, otherwise the selected row index
/// items: `[["2022-05-01", "test", "source_1", "15.50", "Expense"], ]`
pub struct TableData {
    pub state: TableState,
    pub items: Vec<Vec<String>>,
}

impl TableData {
    /// Wraps the given rows in a table whose selection state is ratatui's
    /// default, meaning nothing is selected yet
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

/// Wraps a list of values and keeps track of which one is currently selected.
/// Used for the month and year lists of the selectors.
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

    pub fn new_tx_methods_cumulative(conn: &mut DbConn) -> Self {
        IndexedData {
            titles: conn.get_tx_methods_cumulative(),
            index: 0,
        }
    }

    /// Increases the current index by 1 or goes to 0 if at the final value
    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }

    pub fn next_yearly(&mut self) {
        if self.index == 8 {
            self.titles.remove(0);
            let mut final_year: u64 = self.titles[self.titles.len() - 1].parse().unwrap();
            final_year += 1;
            self.titles.push(final_year.to_string());
        } else {
            self.next();
        }
    }

    /// Decreases the current index by 1 or goes to final index if at 0
    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.titles.len() - 1;
        }
    }

    pub fn previous_yearly(&mut self) {
        if self.index == 8 {
            let first_year: i64 = self.titles[0].parse().unwrap();

            if first_year != 2022 {
                self.titles.pop();
                self.titles.insert(0, (first_year - 1).to_string());
            } else if self.index > 0 {
                self.index -= 1;
            }
        } else if self.index > 0 {
            self.index -= 1;
        }
    }

    pub fn set_index_zero(&mut self) {
        self.index = 0;
    }

    #[must_use]
    pub fn get_selected_value(&self) -> &str {
        &self.titles[self.index]
    }
}

/// Which of the homepage's 3 selectable widgets is currently focused. The focus
/// cycles through all of them, so it never gets stuck at an end.
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

/// Which field of a transaction form is currently being edited. The Add
/// Transaction, Search and Recurring pages each keep their own value of this.
/// The last five values only ever appear on the Recurring page.
#[derive(Debug, PartialEq)]
pub enum TxTab {
    Date,
    Details,
    FromMethod,
    ToMethod,
    Amount,
    TxType,
    Tags,
    Frequency,
    RecurInterval,
    RecurValue,
    RecurMonth,
    EndDate,
    Nothing,
}

/// Which page is currently shown, used to send keypresses to the right handler
pub enum CurrentUi {
    Initial,
    Home,
    AddTx,
    Chart,
    Summary,
    Search,
    Activity,
    Recurring,
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

pub enum SortingType {
    Tags,
    Income,
    Expense,
}

impl SortingType {
    pub fn next_type(&mut self) -> Self {
        match self {
            SortingType::Tags => SortingType::Income,
            SortingType::Income => SortingType::Expense,
            SortingType::Expense => SortingType::Tags,
        }
    }
}

#[derive(PartialEq, Display)]
pub enum HomeRow {
    #[strum(to_string = "Balance")]
    Balance,
    #[strum(to_string = "Changes")]
    Changes,
    #[strum(to_string = "Income")]
    Income,
    #[strum(to_string = "Expense")]
    Expense,
    #[strum(to_string = "DailyIncome")]
    DailyIncome,
    #[strum(to_string = "DailyExpense")]
    DailyExpense,
    #[strum(to_string = "TopRow")]
    TopRow,
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

pub enum LogType {
    Info,
    Error,
}

pub struct LogData {
    pub text: String,
    pub log_type: LogType,
}

impl LogData {
    pub fn new(text: String, log_type: LogType) -> Self {
        LogData { text, log_type }
    }
}
