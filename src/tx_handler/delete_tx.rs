use crate::utility::{get_all_tx_methods, get_last_balance_id, get_last_balances};
use rusqlite::{Connection, Result as sqlResult};

/// Updates the absolute final balance, balance data and deletes the selected transaction.
/// Foreign key cascade takes care of the Changes data in the database.
pub fn delete_tx(id_num: usize, conn: &mut Connection) -> sqlResult<()> {
    let sp = conn.savepoint()?;

    let tx_methods = get_all_tx_methods(&sp);

    // contains the data of the final row data before the tx gets deleted
    let last_balance = get_last_balances(&sp);
    let last_balance_id = get_last_balance_id(&sp)?;

    // will contain data of the updated final balance row data
    let mut final_last_balance = Vec::new();

    // get the deletion tx data
    let query = format!("SELECT * FROM tx_all Where id_num = {}", id_num);
    let data = sp.query_row(&query, [], |row| {
        let final_data: Vec<String> = vec![row.get(0)?, row.get(2)?, row.get(3)?, row.get(4)?];
        Ok(final_data)
    })?;

    // 2025-05-10
    // take 2025 and subtract 2022 = 3, means the year number 3
    // take 05 -> 5 -> 5th month. 5 + (3 * 12) =  the row of this month's balance on balance_all table
    // we are not subtracting 1 from month because balance_all table starts at 1
    let splitted = data[0].split('-').collect::<Vec<&str>>();
    let (year, month) = (
        splitted[0].parse::<i32>().unwrap() - 2022,
        splitted[1].parse::<i32>().unwrap(),
    );

    let mut target_id_num = month + (year * 12);

    let mut from_method = "";
    let mut to_method = "";

    // the tx_method of the tx we will delete
    let source = &data[1];

    // execute this block to get block tx method if the tx type is a Transfer
    if source.contains(" to ") {
        let from_to = data[1].split(" to ").collect::<Vec<&str>>();

        from_method = from_to[0];
        to_method = from_to[1];
    }

    let amount = &data[2].parse::<f64>().unwrap();
    let tx_type: &str = &data[3];

    // loop through all rows in the balance_all table from the deletion point and update balance
    // basically there are 193 rows(at the time of writing) on balance_all table. each row = 1 month. if month 4 had balance of 100,
    // month 5 will also have the balance of 100 if no tx was added on month 5.
    // if the tx deletion happens on row/month 5, that means month 4 balance is correct but from the month from
    // 5 to the final row needs to be deduct that amount.

    // there are 3 steps in deleting a tx
    // Update balance_all table with the amount
    // Delete the tx itself from tx_all table
    // Update balance_all table final balance row data which holds the absolute final amount after all tx
    loop {
        let query = format!(
            "SELECT {} FROM balance_all WHERE id_num = {}",
            tx_methods
                .iter()
                .map(|s| format!(r#""{}""#, s))
                .collect::<Vec<_>>()
                .join(", "),
            target_id_num
        );

        let current_month_balance = sp.query_row(&query, [], |row| {
            let mut final_data: Vec<String> = Vec::new();
            for i in 0..tx_methods.len() {
                let row_data: f64 = row.get(i)?;
                final_data.push(row_data.to_string())
            }
            Ok(final_data)
        })?;

        let mut untouched = true;

        for x in current_month_balance.iter() {
            if x != "0" {
                untouched = false;
                break;
            }
        }

        if untouched {
            target_id_num += 1;
            if target_id_num == 193 {
                break;
            }
            continue;
        }

        let mut updated_month_balance = vec![];

        // add or subtract based on the tx type to the relevant method

        // check the month balance as not zero because if it is 0, there was never any transaction
        // done on that month
        for i in 0..tx_methods.len() {
            if &tx_methods[i] == source && current_month_balance[i] != "0.00" {
                let mut current_amount = current_month_balance[i].parse::<f64>().unwrap();
                if tx_type == "Expense" {
                    current_amount += amount;
                } else if tx_type == "Income" {
                    current_amount -= amount;
                }
                updated_month_balance.push(format!("{:.2}", current_amount));
            } else if tx_methods[i] == from_method && current_month_balance[i] != "0.00" {
                let mut current_amount = current_month_balance[i].parse::<f64>().unwrap();
                current_amount += amount;
                updated_month_balance.push(format!("{:.2}", current_amount));
            } else if tx_methods[i] == to_method && current_month_balance[i] != "0.00" {
                let mut current_amount = current_month_balance[i].parse::<f64>().unwrap();
                current_amount -= amount;
                updated_month_balance.push(format!("{:.2}", current_amount));
            } else {
                updated_month_balance.push(format!(
                    "{:.2}",
                    current_month_balance[i].parse::<f64>().unwrap()
                ));
            }
        }

        let set_values = tx_methods
            .iter()
            .zip(updated_month_balance.iter())
            .map(|(method, value)| format!(r#""{}" = "{}""#, method, value))
            .collect::<Vec<_>>()
            .join(", ");

        let balance_query = format!(
            "UPDATE balance_all SET {} WHERE id_num = {}",
            set_values, target_id_num
        );

        sp.execute(&balance_query, [])?;

        // 193 is the absolute final balance which we don't need to modify
        target_id_num += 1;
        if target_id_num == 193 {
            break;
        }
    }

    // we need to update the 193 row with the latest balance.
    // Based on the tx_type and method, edit the amount from the 193 row's balance
    // we fetched earlier
    for i in 0..tx_methods.len() {
        let mut current_balance = last_balance[i].parse::<f64>().unwrap();
        if &tx_methods[i] == source && tx_type != "Transfer" {
            match tx_type {
                "Expense" => current_balance += amount,
                "Income" => current_balance -= amount,
                _ => {}
            }
        } else if tx_methods[i] == from_method && tx_type == "Transfer" {
            current_balance += amount;
        } else if tx_methods[i] == to_method && tx_type == "Transfer" {
            current_balance -= amount;
        }
        final_last_balance.push(format!("{:.2}", current_balance));
    }

    let del_query = format!("DELETE FROM tx_all WHERE id_num = {id_num}");

    let last_balance_query = format!(
        "UPDATE balance_all SET {} WHERE id_num = {}",
        tx_methods
            .iter()
            .zip(final_last_balance.iter())
            .map(|(method, balance)| format!(r#""{}" = "{}""#, method, balance))
            .collect::<Vec<_>>()
            .join(", "),
        last_balance_id
    );

    sp.execute(&last_balance_query, [])?;
    sp.execute(&del_query, [])?;

    sp.commit()?;
    Ok(())
}
