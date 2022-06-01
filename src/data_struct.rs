use tui::widgets::TableState;
use crate::sub_func::*;
use rusqlite::Connection;

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

impl <'a> TimeData<'a> {
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
        }
        else {
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
            SelectedTab::Table => to_return = SelectedTab::Months
        };
        to_return
    }

    pub fn change_tab_down(self) -> Self {
        let to_return;
        match &self {
            SelectedTab::Years => to_return = SelectedTab::Months,
            SelectedTab::Months => to_return = SelectedTab::Table,
            SelectedTab::Table => to_return = SelectedTab::Years
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
    AddTx
}

pub struct TransactionData {
    pub all_tx: Vec<Vec<String>>,
    all_balance: Vec<Vec<String>>,
    all_changes: Vec<Vec<String>>,
}

impl TransactionData {
    pub fn new(conn: &Connection, month: usize, year: usize) -> Self {
        let all_tx = get_all_txs(conn, month, year);
        let all_balance = get_all_balance(conn);
        let all_changes = get_all_changes(conn);

        TransactionData {
            all_tx,
            all_balance,
            all_changes
        }
    }

    /*pub fn get_txs(&self) -> Vec<Vec<String>> {
        let mut table_data = Vec::new();
        for (i, x) in self.all_tx.iter() {
            table_data.push(x.clone())
        }
        table_data
    }*/

    pub fn get_txs(&self) -> Vec<Vec<String>> {
        let mut table_data = Vec::new();
        for i in self.all_tx.iter() {
            table_data.push(i.clone());
        }
        table_data
    }

    pub fn get_balance(&self, index: usize) -> Vec<String> {
        let mut balance_data = vec!["Balance".to_string()];
        for i in  self.all_balance[index].iter() {
            balance_data.push(i.to_string());
        }
        balance_data
    }

    pub fn get_last_balance(&self) -> Vec<String> {
        let mut balance_data = vec!["Balance".to_string()];
        let last_index = self.all_balance.len() - 1;
        for i in self.all_balance[last_index].iter() {
            balance_data.push(i.to_string());
        }
        balance_data
    }
    pub fn get_changes(&self, index: usize) -> Vec<String> {
        let mut changes_data = vec!["Changes".to_string()];
        for i in self.all_changes[index].iter() {
            changes_data.push(i.to_string());
        }
        changes_data
    }
}

pub struct AddTxData {
    date: String,
    details: String,
    tx_method: String,
    amount: String,
    tx_type: String,
}

impl AddTxData {
    pub fn new() -> Self {
        AddTxData {
            date: "".to_string(),
            details: "".to_string(),
            tx_method: "".to_string(),
            amount: "".to_string(),
            tx_type: "".to_string(),
        }
    }

    pub fn get_all_texts(&self) -> Vec<&str> {
        vec![&self.date, &self.details, &self.tx_method, &self.amount, &self.tx_type]
    }

    //TODO emit some kind of status to place on placement field ex check date format, amount
    pub fn edit_date(&mut self, text: char, pop_last: bool){
        match pop_last {
            true => {
                if self.date.len() > 0 {
                    self.date.pop().unwrap();
                }
            },
            false => self.date = format!("{}{text}", self.date),
        }
    }

    pub fn edit_details(&mut self, text: char, pop_last: bool) {
        match pop_last {
            true => {
                if self.details.len() > 0 {
                    self.details.pop().unwrap();
                }
            },
            false => self.details = format!("{}{text}", self.details),
        }
    }

    pub fn edit_tx_method(&mut self, text: char, pop_last: bool) {
        match pop_last {
            true => {
                if self.tx_method.len() > 0 {
                    self.tx_method.pop().unwrap();
                }
            },
            false => self.tx_method = format!("{}{text}", self.tx_method),
        }
    }

    pub fn edit_amount(&mut self, text: char, pop_last: bool) {
        match pop_last {
            true => {
                if self.amount.len() > 0 {
                    self.amount.pop().unwrap();
                }
            },
            false => self.amount = format!("{}{text}", self.amount),
        }
    }

    pub fn edit_tx_type(&mut self, text: char, pop_last: bool) {
        match pop_last {
            true => {
                if self.tx_type.len() > 0 {
                    self.tx_type.pop().unwrap();
                }
            },
            false => self.tx_type = format!("{}{text}", self.tx_type),
        }
    }

    pub fn add_tx(&mut self, conn: &Connection) -> String {
        let status = add_new_tx(conn, &self.date, &self.details, &self.tx_method, &self.amount, &self.tx_type);
        match status {
            Ok(_) => "Transaction Added Successfully".to_string(),
            Err(e) => format!("Error Adding Transaction. Error: {}", e),
        }
    }
}