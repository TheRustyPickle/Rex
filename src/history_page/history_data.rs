use rusqlite::Connection;
use std::collections::HashMap;

use crate::page_handler::ActivityType;
use crate::utility::get_all_activities;

pub struct ActivityData {
    created_on: String,
    activity_type: ActivityType,
    description: String,
    activity_num: i32,
}

impl ActivityData {
    pub fn new(
        date: String,
        activity_type: String,
        description: String,
        activity_num: i32,
    ) -> Self {
        ActivityData {
            created_on: date,
            activity_type: ActivityType::from_str(&activity_type),
            description,
            activity_num,
        }
    }

    pub fn activity_num(&self) -> i32 {
        self.activity_num
    }

    fn to_vec(&self) -> Vec<String> {
        vec![
            self.created_on.clone(),
            self.activity_type.to_str(),
            self.description.clone(),
        ]
    }
}

pub struct ActivityTx {
    date: String,
    details: String,
    tx_method: String,
    amount: String,
    tx_type: String,
    tags: String,
    id_num: String,
    activity_num: i32,
    insertion_id: i32,
}

impl ActivityTx {
    pub fn new(
        date: String,
        details: String,
        tx_method: String,
        amount: String,
        tx_type: String,
        tags: String,
        id_num: String,
        activity_num: i32,
        insertion_id: i32,
    ) -> Self {
        ActivityTx {
            date,
            details,
            tx_method,
            amount,
            tx_type,
            tags,
            id_num,
            activity_num,
            insertion_id,
        }
    }

    pub fn activity_num(&self) -> i32 {
        self.activity_num
    }

    pub fn to_vec(&self, compare_with: Option<i32>) -> Vec<String> {
        if let Some(id) = compare_with {
            let is_smaller = if id == self.insertion_id { true } else { false };

            if is_smaller {
                return vec![
                    self.date.clone(),
                    self.details.clone(),
                    self.tx_method.clone(),
                    self.amount.clone(),
                    self.tx_type.clone(),
                    self.tags.clone(),
                    self.id_num.clone(),
                    String::from("New Tx"),
                ];
            } else {
                return vec![
                    self.date.clone(),
                    self.details.clone(),
                    self.tx_method.clone(),
                    self.amount.clone(),
                    self.tx_type.clone(),
                    self.tags.clone(),
                    self.id_num.clone(),
                    String::from("Old Tx"),
                ];
            }
        }

        vec![
            self.date.clone(),
            self.details.clone(),
            self.tx_method.clone(),
            self.amount.clone(),
            self.tx_type.clone(),
            self.tags.clone(),
            self.id_num.clone(),
        ]
    }
}

pub struct HistoryData {
    pub activities: Vec<ActivityData>,
    activity_txs: HashMap<i32, Vec<ActivityTx>>,
}

impl HistoryData {
    pub fn new(month: usize, year: usize, conn: &Connection) -> Self {
        let (activities, activity_txs) = get_all_activities(month, year, conn);

        HistoryData {
            activities,
            activity_txs,
        }
    }

    pub fn get_txs(&self) -> Vec<Vec<String>> {
        let mut txs = Vec::new();

        for tx in &self.activities {
            txs.push(tx.to_vec())
        }

        txs
    }

    pub fn get_activity_txs(&self, index: Option<usize>) -> Vec<Vec<String>> {
        let Some(index) = index else {
            return Vec::new();
        };

        let target_activity_num = self.activities[index].activity_num();

        let target_txs = self.activity_txs.get(&target_activity_num).unwrap();

        let mut txs = Vec::new();

        let mut is_swap = false;

        let compare_with = if target_txs.len() == 2 {
            if let ActivityType::IDNumSwap(_, _) = self.activities[index].activity_type {
                is_swap = true;
                None
            } else {
                Some(target_txs[0].insertion_id)
            }
        } else {
            None
        };

        for tx in target_txs {
            let mut data = tx.to_vec(compare_with);
            if is_swap {
                data.push(String::from("New ID"));
            }
            txs.push(data)
        }

        txs
    }

    pub fn is_activity_empty(&self) -> bool {
        self.activities.is_empty()
    }

    pub fn add_extra_field(&self, index: usize) -> bool {
        let target_activity = self.activities.get(index).unwrap();

        if let ActivityType::EditTX(_) = target_activity.activity_type {
            return true;
        }

        if let ActivityType::IDNumSwap(_, _) = target_activity.activity_type {
            return true;
        }

        false
    }
}
