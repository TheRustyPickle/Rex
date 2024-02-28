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
}

pub struct HistoryData {
    activities: Vec<ActivityData>,
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
}
