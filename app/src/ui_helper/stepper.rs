use db::ConnCache;
use db::models::TxType;

use crate::conn::MutDbConn;
use crate::ui_helper::{DateType, StepType, get_best_match};

pub struct Stepper<'a> {
    conn: MutDbConn<'a>,
}

impl<'a> Stepper<'a> {
    pub(crate) fn new(conn: MutDbConn<'a>) -> Self {
        Self { conn }
    }

    pub fn date(user_input: &mut String, date_type: &DateType, step_type: StepType) {

    }
}
