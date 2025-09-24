use chrono::{Datelike, Days, Local, Months, NaiveDate, NaiveDateTime, NaiveTime};
use diesel::prelude::*;
use diesel::result::Error;

use crate::ConnCache;
use crate::models::{ActivityNature, ActivityTx, FullActivityTx};
use crate::schema::activities;

#[derive(Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = activities)]
pub struct NewActivity {
    date: NaiveDateTime,
    activity_type: String,
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = activities)]
pub struct Activity {
    pub id: i32,
    date: NaiveDateTime,
    pub activity_type: String,
}

pub struct ActivityWithTxs {
    pub activity: Activity,
    pub txs: Vec<FullActivityTx>,
}

impl NewActivity {
    #[must_use]
    pub fn new(activity_type: ActivityNature) -> Self {
        let now = Local::now().naive_local();

        Self {
            date: now,
            activity_type: activity_type.into(),
        }
    }

    pub fn insert(self, db_conn: &mut impl ConnCache) -> Result<Activity, Error> {
        use crate::schema::activities::dsl::activities;

        diesel::insert_into(activities)
            .values(self)
            .returning(Activity::as_returning())
            .get_result(db_conn.conn())
    }
}

impl Activity {
    #[must_use]
    pub fn new(date: NaiveDateTime, activity_type: ActivityNature, id: i32) -> Self {
        Self {
            id,
            date,
            activity_type: activity_type.into(),
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

        let start_date = start_date.and_time(NaiveTime::MIN);
        let end_date = end_date.and_time(NaiveTime::MIN);

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

    #[must_use]
    pub fn to_array(&self) -> Vec<String> {
        let activity_type: ActivityNature = self.activity_type.as_str().into();
        vec![
            self.date.format("%a %d %I:%M %p").to_string(),
            activity_type.to_string(),
        ]
    }
}

impl ActivityWithTxs {
    #[must_use]
    pub fn to_array(&self) -> Vec<Vec<String>> {
        let first_tx = self.txs.first().unwrap();

        let last_tx = self.txs.last().unwrap();

        match self.activity.activity_type.as_str().into() {
            ActivityNature::PositionSwap | ActivityNature::EditTx => {
                assert!(
                    (first_tx.id != last_tx.id),
                    "Both activity tx id should not have matched"
                );

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

                        vec![lower_id_tx_array, higher_id_tx_array]
                    }
                    ActivityNature::EditTx => {
                        lower_id_tx_array.push("New Tx".to_string());

                        higher_id_tx_array.push("Old Tx".to_string());

                        vec![lower_id_tx_array, higher_id_tx_array]
                    }
                    _ => unreachable!(),
                }
            }
            _ => {
                vec![first_tx.to_array()]
            }
        }
    }
}
