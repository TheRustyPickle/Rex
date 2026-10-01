use anyhow::Result;
use chrono::Datelike;
use rex_db::ConnCache;
use rex_db::models::{Balance, Tx};
use std::collections::HashMap;

use crate::modifier::tidy_balances;

/// Do the V1 migration of the balances. Tidies every single balance and the final balance. This
/// removes the defensive balance update in the balance fetching.
pub(crate) fn migrate_to_v1(conn: &mut impl ConnCache) -> Result<()> {
    // If no tx, nothing to tidy
    let Some(oldest_tx_date) = Tx::get_lowest_date(conn)? else {
        return Ok(());
    };

    let oldest_month = oldest_tx_date.date().with_day(1).unwrap();

    tidy_balances(oldest_month, conn)?;

    // Tidied from the oldest tx to the newest balance, so the latest monthly balances are now
    // assumed to be absolutely correct.
    let latest_balances: HashMap<i32, i64> = Balance::get_balance_highest_date(conn)?
        .into_iter()
        .map(|balance| (balance.method_id, balance.balance))
        .collect();

    for (method_id, mut final_balance) in Balance::get_final_balance(conn)? {
        final_balance.balance = latest_balances.get(&method_id).copied().unwrap_or(0);
        final_balance.update_final_balance(conn)?;
    }

    Ok(())
}
