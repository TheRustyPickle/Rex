use tui::widgets::TableState;
use std::collections::HashMap;
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

pub struct TransactionData {
    all_tx: Vec<Vec<String>>,
    all_balance: HashMap<i32, Vec<String>>,
    all_changes: HashMap<i32, Vec<String>>,
}

impl TransactionData {
    pub fn new(conn: &Connection, month: usize, year: usize) -> Self {
        let all_tx = test_get_all_txs(conn, month, year);
        let all_balance = get_all_balance(conn, month, year);
        let all_changes = get_all_changes(conn, month, year);

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

    pub fn get_balance(&self, index: i32) -> Vec<String> {
        let mut balance_data = vec!["Balance".to_string()];
        for i in self.all_balance[&index].iter() {
            balance_data.push(i.to_string());
        }
        balance_data
    }

    pub fn get_last_balance(&self) -> Vec<String> {
        let mut balance_data = vec!["Balance".to_string()];
        let last_index = self.all_balance.len() as i32 - 1;
        for i in self.all_balance[&last_index].iter() {
            balance_data.push(i.to_string());
        }
        println!("{:?}", balance_data);
        balance_data
    }
    pub fn get_changes(&self, index: i32) -> Vec<String> {
        let mut changes_data = vec!["Changes".to_string()];
        for i in self.all_changes[&index].iter() {
            changes_data.push(i.to_string());
        }
        changes_data
    }
}