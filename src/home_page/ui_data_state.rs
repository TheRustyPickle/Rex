use tui::widgets::TableState;

pub struct TableData {
    pub state: TableState,
    pub items: Vec<Vec<String>>,
}

impl TableData {
    pub fn new(data: Vec<Vec<String>>) -> Self {
        TableData {
            state: TableState::default(),
            items: data,
        }
    }

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

pub struct TimeData<'a> {
    pub titles: Vec<&'a str>,
    pub index: usize,
}

impl<'a> TimeData<'a> {
    pub fn new(values: Vec<&'a str>) -> Self {
        TimeData {
            titles: values,
            index: 0,
        }
    }

    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.titles.len() - 1;
        }
    }
}

pub enum SelectedTab {
    Years,
    Months,
    Table,
}

impl SelectedTab {
    pub fn change_tab_up(self) -> Self {
        let to_return;
        match &self {
            SelectedTab::Years => to_return = SelectedTab::Table,
            SelectedTab::Months => to_return = SelectedTab::Years,
            SelectedTab::Table => to_return = SelectedTab::Months,
        };
        to_return
    }

    pub fn change_tab_down(self) -> Self {
        let to_return;
        match &self {
            SelectedTab::Years => to_return = SelectedTab::Months,
            SelectedTab::Months => to_return = SelectedTab::Table,
            SelectedTab::Table => to_return = SelectedTab::Years,
        };
        to_return
    }
}

pub enum TxTab {
    Date,
    Details,
    TxMethod,
    Amount,
    TxType,
    Nothing,
}

pub enum CurrentUi {
    Home,
    AddTx,
}
