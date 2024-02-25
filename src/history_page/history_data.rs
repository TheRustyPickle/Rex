use rusqlite::Connection;
use std::collections::HashMap;

use crate::page_handler::ActivityType;

pub struct ActivityData {
    created_on: String,
    activity_type: ActivityType,
    description: String,
    activity_num: usize,
}

pub struct ActivityTx {
    date: String,
    details: String,
    tx_method: String,
    amount: f64,
    tx_type: String,
    id_num: usize,
    tags: String,
    activity_num: usize,
}

pub struct HistoryData {
    // Each to be turned into the a table row
    activities: Vec<ActivityData>,
    // activity num: activity txs
    // a single activity can affect multiple transactions so a vector for the txs
    activity_txs: HashMap<usize, Vec<ActivityTx>>,
}

impl HistoryData {
    pub fn new(month: usize, year: usize, conn: &Connection) -> Self {
        // Fetch these data from the database
        // TODO: Two new tables. 1 would contain the activity details. The other table would contain txs and a link to the activity number
        // In edit and swap mode save both delete and new tx activity, this way we can keep track of both old and the new values
        todo!()
    }
}
