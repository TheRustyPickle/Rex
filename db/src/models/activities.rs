use chrono::{Datelike, Days, Months, NaiveDate};
use diesel::prelude::*;
use diesel::result::Error;
use std::fmt::{Display, Formatter};

use crate::ConnCache;
use crate::models::{ActivityTx, FullActivityTx};
use crate::schema::activities;

#[derive(Clone, Debug, Copy)]
pub enum ActivityNature {
    AddTx,
    EditTx,
    DeleteTx,
    SearchTx,
    PositionSwap,
}

impl From<&str> for ActivityNature {
    fn from(s: &str) -> Self {
        match s {
            "add_tx" => ActivityNature::AddTx,
            "edit_tx" => ActivityNature::EditTx,
            "delete_tx" => ActivityNature::DeleteTx,
            "search_tx" => ActivityNature::SearchTx,
            "position_swap" => ActivityNature::PositionSwap,
            other => panic!("Invalid TxType string: {other}"),
        }
    }
}

impl From<ActivityNature> for String {
    fn from(a: ActivityNature) -> Self {
        match a {
            ActivityNature::AddTx => "add_tx".to_string(),
            ActivityNature::EditTx => "edit_tx".to_string(),
            ActivityNature::DeleteTx => "delete_tx".to_string(),
            ActivityNature::SearchTx => "search_tx".to_string(),
            ActivityNature::PositionSwap => "position_swap".to_string(),
        }
    }
}

impl Display for ActivityNature {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ActivityNature::AddTx => write!(f, "add_tx"),
            ActivityNature::EditTx => write!(f, "edit_tx"),
            ActivityNature::DeleteTx => write!(f, "delete_tx"),
            ActivityNature::SearchTx => write!(f, "search_tx"),
            ActivityNature::PositionSwap => write!(f, "position_swap"),
        }
    }
}

#[derive(Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = activities)]
pub struct NewActivity {
    date: NaiveDate,
    activity_type: String,
    description: String,
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = activities)]
pub struct Activity {
    id: i32,
    date: NaiveDate,
    pub activity_type: String,
    description: String,
}

pub struct ActivityWithTxs {
    pub activity: Activity,
    pub txs: Vec<FullActivityTx>,
}

impl NewActivity {
    pub fn new(date: NaiveDate, activity_type: &str, description: &str) -> Self {
        Self {
            date,
            activity_type: activity_type.to_string(),
            description: description.to_string(),
        }
    }

    pub fn insert(self, db_conn: &mut impl ConnCache) -> Result<Self, Error> {
        use crate::schema::activities::dsl::activities;

        diesel::insert_into(activities)
            .values(self)
            .returning(Self::as_returning())
            .get_result(db_conn.conn())
    }
}

impl Activity {
    pub fn new(date: NaiveDate, activity_type: ActivityNature, description: &str, id: i32) -> Self {
        Self {
            id,
            date,
            activity_type: activity_type.into(),
            description: description.to_string(),
        }
    }

    pub fn insert(self, db_conn: &mut impl ConnCache) -> Result<Self, Error> {
        use crate::schema::activities::dsl::activities;

        diesel::insert_into(activities)
            .values(self)
            .returning(Self::as_returning())
            .get_result(db_conn.conn())
    }

    pub fn get_activities(
        d: NaiveDate,
        db_conn: &mut impl ConnCache,
    ) -> Result<Vec<ActivityWithTxs>, Error> {
        use crate::schema::activities::dsl as act;
        use crate::schema::activity_txs::dsl as tx;

        let start_date = NaiveDate::from_ymd_opt(d.year(), d.month(), 1).unwrap();
        let end_date = start_date + Months::new(1) - Days::new(1);

        let results: Vec<(Activity, ActivityTx)> = act::activities
            .inner_join(tx::activity_txs.on(tx::activity_num.eq(act::id)))
            .filter(act::date.ge(start_date))
            .filter(act::date.le(end_date))
            .order((act::date.asc(), act::id.asc()))
            .select((Activity::as_select(), ActivityTx::as_select()))
            .load(db_conn.conn())?;

        let mut grouped: Vec<ActivityWithTxs> = Vec::new();

        let activity_txs: Vec<&ActivityTx> = results.iter().map(|(_, tx)| tx).collect();

        let full_activity_txs = ActivityTx::convert_to_full_tx(activity_txs, db_conn)?;

        for ((activity, _), tx) in results.into_iter().zip(full_activity_txs) {
            if let Some(last) = grouped.last_mut()
                && last.activity.id == activity.id
            {
                last.txs.push(tx);
                continue;
            }

            grouped.push(ActivityWithTxs {
                activity,
                txs: vec![tx],
            });
        }

        Ok(grouped)
    }

    pub fn to_array(&self) -> Vec<String> {
        vec![
            self.date.format("%d-%m-%Y").to_string(),
            self.activity_type.clone(),
            self.description.clone(),
        ]
    }
}

impl ActivityWithTxs {
    pub fn to_array(&self) -> Vec<Vec<String>> {
        let first_tx = self.txs.first().unwrap();

        let last_tx = self.txs.last().unwrap();

        match self.activity.activity_type.as_str().into() {
            ActivityNature::PositionSwap | ActivityNature::EditTx => {
                if first_tx.id == last_tx.id {
                    panic!("Both activity tx id should not have matched")
                }

                let lower_id_tx = if first_tx.id < last_tx.id {
                    first_tx
                } else {
                    last_tx
                };

                let higher_id_tx = if first_tx.id > last_tx.id {
                    first_tx
                } else {
                    last_tx
                };

                let mut lower_id_tx_array = lower_id_tx.to_array();
                let mut higher_id_tx_array = higher_id_tx.to_array();

                match self.activity.activity_type.as_str().into() {
                    ActivityNature::PositionSwap => {
                        let higher_display_order = higher_id_tx
                            .display_order
                            .expect("Display order should not be none for this type of activity");
                        let lower_display_order = lower_id_tx
                            .display_order
                            .expect("Display order should not be none for this type of activity");

                        lower_id_tx_array
                            .push(format!("{higher_display_order} → {lower_display_order}"));

                        higher_id_tx_array
                            .push(format!("{lower_display_order} → {higher_display_order}"));

                        return vec![lower_id_tx_array, higher_id_tx_array];
                    }
                    ActivityNature::EditTx => {
                        lower_id_tx_array.push("New Tx".to_string());

                        higher_id_tx_array.push("Old Tx".to_string());

                        return vec![lower_id_tx_array, higher_id_tx_array];
                    }
                    _ => {}
                }
            }
            _ => {}
        };

        todo!()
    }
}
