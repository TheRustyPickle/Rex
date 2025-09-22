use anyhow::Result;
use chrono::NaiveDate;
use db::ConnCache;
pub use db::models::FullActivityTx;
use db::models::{Activity, ActivityNature, ActivityWithTxs};

pub struct ActivityView(Vec<ActivityWithTxs>);

pub(crate) fn get_activity_view(
    date: NaiveDate,
    conn: &mut impl ConnCache,
) -> Result<ActivityView> {
    let activities = Activity::get_activities(date, conn)?;

    Ok(ActivityView(activities))
}

impl ActivityView {
    #[must_use]
    pub fn total_activity(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn get_activity_table(&self) -> Vec<Vec<String>> {
        self.0.iter().map(|a| a.activity.to_array()).collect()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[must_use]
    pub fn get_activity_txs_table(&self, index: Option<usize>) -> Vec<Vec<String>> {
        let Some(index) = index else {
            return Vec::new();
        };

        let target_activity = self.0.get(index).unwrap();

        target_activity.to_array()
    }

    #[must_use]
    pub fn add_extra_field(&self, index: usize) -> bool {
        let target_activity = self.0.get(index).unwrap();

        matches!(
            target_activity.activity.activity_type.as_str().into(),
            ActivityNature::EditTx | ActivityNature::PositionSwap
        )
    }

    pub fn get_activity_txs(&self, index: usize) -> Vec<&FullActivityTx> {
        self.0.get(index).unwrap().txs.iter().collect()
    }
}
