use rusqlite::Connection;
use std::collections::HashMap;

use crate::page_handler::ActivityType;
use crate::utility::get_all_activities;

pub struct ActivityDetails {
    created_on: String,
    pub activity_type: ActivityType,
    pub description: String,
    activity_num: i32,
}

impl ActivityDetails {
    pub fn new(
        date: String,
        activity_type: String,
        description: String,
        activity_num: i32,
    ) -> Self {
        ActivityDetails {
            created_on: date,
            activity_type: ActivityType::from_s(&activity_type),
            description,
            activity_num,
        }
    }

    pub fn activity_num(&self) -> i32 {
        self.activity_num
    }

    #[cfg(not(tarpaulin_include))]
    fn to_vec(&self) -> Vec<String> {
        vec![
            self.created_on.clone(),
            self.activity_type.to_str(),
            self.description.clone(),
        ]
    }
}

pub struct ActivityTx {
    pub date: String,
    pub details: String,
    pub tx_method: String,
    pub amount: String,
    pub tx_type: String,
    pub tags: String,
    pub id_num: String,
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

    #[cfg(not(tarpaulin_include))]
    pub fn to_vec(&self, smaller_num: Option<i32>) -> Vec<String> {
        if let Some(id) = smaller_num {
            let is_smaller = id == self.insertion_id;

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
            }
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

pub struct ActivityData {
    pub activities: Vec<ActivityDetails>,
    activity_txs: HashMap<i32, Vec<ActivityTx>>,
}

impl ActivityData {
    #[cfg(not(tarpaulin_include))]
    pub fn new(month: usize, year: usize, conn: &Connection) -> Self {
        let (activities, activity_txs) = get_all_activities(month, year, conn);

        ActivityData {
            activities,
            activity_txs,
        }
    }

    /// Convert all activity to a Vector where each value of the vector is a vector of the activity
    #[cfg(not(tarpaulin_include))]
    pub fn get_txs(&self) -> Vec<Vec<String>> {
        let mut txs = Vec::new();

        for tx in &self.activities {
            txs.push(tx.to_vec());
        }

        txs
    }

    /// Convert all activity txs to a Vector where each value of the vector is a vector of the tx data
    #[cfg(not(tarpaulin_include))]
    pub fn get_activity_txs(&self, index: Option<usize>) -> Vec<Vec<String>> {
        let Some(index) = index else {
            return Vec::new();
        };

        let target_activity_num = self.activities[index].activity_num();

        let target_txs = self.activity_txs.get(&target_activity_num).unwrap();

        let mut txs = Vec::new();

        let mut is_swap = false;

        let mut smaller_num = if target_txs.len() == 2 {
            let first_tx_id = target_txs[0].insertion_id;
            let second_tx_id = target_txs[1].insertion_id;

            if first_tx_id < second_tx_id {
                Some(first_tx_id)
            } else {
                Some(second_tx_id)
            }
        } else {
            None
        };

        if let ActivityType::IDNumSwap(_, _) = self.activities[index].activity_type {
            is_swap = true;
            smaller_num = None;
        }

        for tx in target_txs {
            let mut data = tx.to_vec(smaller_num);
            if is_swap {
                data.push(String::from("New ID"));
            }
            txs.push(data);
        }

        txs
    }

    /// Whether there is any activity data
    #[cfg(not(tarpaulin_include))]
    pub fn is_activity_empty(&self) -> bool {
        self.activities.is_empty()
    }

    /// Whether the activity at this index should have an extra field in the UI
    #[cfg(not(tarpaulin_include))]
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
