use chrono::NaiveDate;
use db::models::FullTx;
use std::collections::{HashMap, HashSet};

use crate::fetcher::{Cent, TxViewGroup};

pub struct ChartView {
    txs: TxViewGroup,
    dates: HashSet<NaiveDate>,
    first_date: NaiveDate,
    last_date: NaiveDate,
}

pub(crate) fn get_chart_view(txs: TxViewGroup) -> ChartView {
    let mut first_date = NaiveDate::default();
    let mut last_date = NaiveDate::default();

    let default_date = NaiveDate::default();

    let unique_dates: HashSet<NaiveDate> = txs
        .0
        .iter()
        .map(|tx| {
            let tx_date = tx.tx.date;

            if first_date == default_date {
                first_date = tx_date;
            }

            if last_date == default_date {
                last_date = tx_date;
            }

            if tx_date < first_date {
                first_date = tx_date;
            }

            if tx_date > last_date {
                last_date = tx_date;
            }

            tx.tx.date
        })
        .collect();

    ChartView {
        txs,
        dates: unique_dates,
        first_date,
        last_date,
    }
}

impl ChartView {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.txs.0.is_empty()
    }

    #[must_use]
    pub fn contains_date(&self, date: &NaiveDate) -> bool {
        self.dates.contains(date)
    }

    #[must_use]
    pub fn start_date(&self) -> NaiveDate {
        self.first_date
    }

    #[must_use]
    pub fn end_date(&self) -> NaiveDate {
        self.last_date
    }

    #[must_use]
    pub fn get_balance(&self, index: usize) -> &HashMap<i32, Cent> {
        self.txs.get_tx_balance(index)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.txs.len()
    }

    #[must_use]
    pub fn get_tx(&self, index: usize) -> &FullTx {
        &self.txs.0[index].tx
    }
}
