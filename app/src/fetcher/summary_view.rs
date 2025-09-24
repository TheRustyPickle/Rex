use anyhow::Result;
use chrono::{Datelike, NaiveDate};
use db::ConnCache;
use db::models::{FetchNature, FullTx, TxType};
use shared::models::{Cent, Dollar};
use std::collections::HashMap;

use crate::fetcher::{
    LargestMomvement, LargestType, PeakMonthlyMovement, PeakType, SummaryLargest, SummaryMethods,
    SummaryNet, SummaryPeak,
};
use crate::utils::{get_percentages, month_year_to_unique};

/// Contains `FullTx` to generate summary data. Will always contain the exact number of txs from
/// the month and year (or all txs) the summary was generated with
pub struct SummaryView {
    txs: Vec<FullTx>,
    nature: FetchNature,
}

pub struct FullSummary {
    methods: Vec<SummaryMethods>,
    largest: Vec<SummaryLargest>,
    peak: Vec<SummaryPeak>,
    net: SummaryNet,
}

impl FullSummary {
    #[must_use]
    pub fn net_array(&self) -> Vec<Vec<String>> {
        self.net.array()
    }

    pub fn peak_array(&self) -> Vec<Vec<String>> {
        self.peak.iter().map(SummaryPeak::array).collect()
    }

    pub fn method_array(&self) -> Vec<Vec<String>> {
        self.methods.iter().map(SummaryMethods::array).collect()
    }

    pub fn largest_array(&self) -> Vec<Vec<String>> {
        self.largest.iter().map(SummaryLargest::array).collect()
    }
}

type CacheTxs = HashMap<i32, Vec<FullTx>>;

pub(crate) fn get_summary(
    date: NaiveDate,
    nature: FetchNature,
    conn: &mut impl ConnCache,
) -> Result<(SummaryView, Option<CacheTxs>)> {
    let txs = FullTx::get_txs(date, nature, conn)?;

    let mut create_map = false;
    if let FetchNature::All = nature {
        create_map = true;
    }

    if create_map {
        let mut map = HashMap::with_capacity(txs.len());

        for tx in &txs {
            let unique_value = month_year_to_unique(date.month() as i32, date.year());

            map.entry(unique_value)
                .or_insert_with(Vec::new)
                .push(tx.clone());
        }

        let summary_view = SummaryView { txs, nature };

        return Ok((summary_view, Some(map)));
    }

    let summary_view = SummaryView { txs, nature };

    Ok((summary_view, None))
}

impl SummaryView {
    pub fn tags_array(
        &self,
        compare: Option<&SummaryView>,
        conn: &impl ConnCache,
    ) -> Vec<Vec<String>> {
        let mut income_tags = HashMap::new();
        let mut expense_tags = HashMap::new();

        let mut total_income = Cent::new(0);
        let mut total_expense = Cent::new(0);

        let mut no_mom_yoy = false;

        if let FetchNature::All = self.nature {
            no_mom_yoy = true;
        }

        let mut compare_income_tags = HashMap::new();
        let mut compare_expense_tags = HashMap::new();

        if !no_mom_yoy && let Some(compare) = compare {
            let (income_map, expense_map) = compare.get_tags_movement_map();
            compare_income_tags = income_map;
            compare_expense_tags = expense_map;
        }

        for tx in &self.txs {
            for tag in &tx.tags {
                match tx.tx_type {
                    TxType::Income => {
                        total_income += tx.amount;

                        let value = income_tags.entry(tag.name.clone()).or_insert(Cent::new(0));
                        *value += tx.amount;
                    }
                    TxType::Expense => {
                        total_expense += tx.amount;

                        let value = expense_tags.entry(tag.name.clone()).or_insert(Cent::new(0));
                        *value += tx.amount;
                    }
                    TxType::Transfer => {}
                }
            }
        }

        let mut to_return = Vec::new();

        for tag in conn.cache().tags.values() {
            let mut no_push = true;

            let mut to_push = vec![tag.name.clone()];

            let mut income_percentage = 0.0;
            let mut expense_percentage = 0.0;

            let mut income_amount = Dollar::new(0.0);
            let mut expense_amount = Dollar::new(0.0);

            if let Some(income) = income_tags.get(&tag.name) {
                income_percentage = (income.value() as f64 / total_income.value() as f64) * 100.0;
                income_amount = income.dollar();

                no_push = false;
            }

            if let Some(expense) = expense_tags.get(&tag.name) {
                expense_percentage =
                    (expense.value() as f64 / total_expense.value() as f64) * 100.0;
                expense_amount = expense.dollar();

                no_push = false;
            }

            if no_push {
                continue;
            }

            to_push.push(format!("{income_amount:.2}"));
            to_push.push(format!("{expense_amount:.2}"));

            to_push.push(format!("{income_percentage:.2}"));
            to_push.push(format!("{expense_percentage:.2}"));

            if !no_mom_yoy && compare.is_some() {
                let compare_income = compare_income_tags.get(&tag.name);

                let compare_expense = compare_expense_tags.get(&tag.name);

                if let Some(last_income) = compare_income {
                    let last_earning = last_income.dollar();

                    let earning_increased_percentage =
                        ((income_amount - last_earning) / last_earning) * 100.0;

                    if last_earning == 0.0 {
                        to_push.push("∞".to_string());
                    } else if earning_increased_percentage < 0.0 {
                        to_push.push(format!("↓{:.2}", earning_increased_percentage.abs()));
                    } else {
                        to_push.push(format!("↑{earning_increased_percentage:.2}"));
                    }
                } else {
                    to_push.push("∞".to_string());
                }

                if let Some(last_expense) = compare_expense {
                    let last_expense = last_expense.dollar();
                    let expense_increased_percentage =
                        ((expense_amount - last_expense) / last_expense) * 100.0;

                    if last_expense == 0.0 {
                        to_push.push("∞".to_string());
                    } else if expense_increased_percentage < 0.0 {
                        to_push.push(format!("↓{:.2}", expense_increased_percentage.abs()));
                    } else {
                        to_push.push(format!("↑{expense_increased_percentage:.2}"));
                    }
                } else {
                    to_push.push("∞".to_string());
                }
            }

            to_return.push(to_push);
        }

        to_return
    }

    fn get_tags_movement_map(&self) -> (HashMap<String, Cent>, HashMap<String, Cent>) {
        let mut income_tags = HashMap::new();
        let mut expense_tags = HashMap::new();

        for tx in &self.txs {
            for tag in &tx.tags {
                match tx.tx_type {
                    TxType::Income => {
                        let value = income_tags.entry(tag.name.clone()).or_insert(Cent::new(0));
                        *value += tx.amount;
                    }
                    TxType::Expense => {
                        let value = expense_tags.entry(tag.name.clone()).or_insert(Cent::new(0));
                        *value += tx.amount;
                    }
                    TxType::Transfer => {}
                }
            }
        }

        (income_tags, expense_tags)
    }

    #[allow(private_interfaces)]
    pub fn generate_summary(
        &self,
        last_summary: Option<&FullSummary>,
        conn: &impl ConnCache,
    ) -> FullSummary {
        let mut no_mom_yoy = false;

        if let FetchNature::All = self.nature {
            no_mom_yoy = true;
        }

        let mut total_income = Cent::new(0);
        let mut total_expense = Cent::new(0);

        let mut total_month_checked = 0;

        let mut biggest_earning = LargestMomvement::default();
        let mut biggest_expense = LargestMomvement::default();

        let mut method_earning = HashMap::new();
        let mut method_expense = HashMap::new();

        for method in conn.cache().get_methods() {
            method_earning.insert(method.name.to_string(), Cent::new(0));
            method_expense.insert(method.name.to_string(), Cent::new(0));
        }

        let mut ongoing_month = 0;

        let mut ongoing_date = NaiveDate::default();

        let mut peak_earning = PeakMonthlyMovement::default();
        let mut peak_expense = PeakMonthlyMovement::default();

        let mut last_peak_earning = PeakMonthlyMovement::default();
        let mut last_peak_expense = PeakMonthlyMovement::default();

        for tx in &self.txs {
            ongoing_date = tx.date.date();

            let time_unique = month_year_to_unique(tx.date.month() as i32, tx.date.year());

            if ongoing_month == 0 {
                ongoing_month = time_unique;
            }

            if time_unique != ongoing_month {
                ongoing_month = time_unique;
                total_month_checked += 1;

                if last_peak_earning.amount > peak_earning.amount {
                    peak_earning = last_peak_earning;
                    last_peak_earning = PeakMonthlyMovement::new(tx.date.date());
                }

                if last_peak_expense.amount > peak_expense.amount {
                    peak_expense = last_peak_expense;
                    last_peak_expense = PeakMonthlyMovement::new(tx.date.date());
                }
            }

            match tx.tx_type {
                TxType::Income => {
                    total_income += tx.amount;
                    let amount = method_earning
                        .entry(tx.from_method.name.clone())
                        .or_insert(Cent::new(0));

                    *amount += tx.amount;

                    if biggest_earning.amount < tx.amount {
                        biggest_earning.amount = tx.amount;
                        biggest_earning.date = tx.date.date();
                        biggest_earning.method = tx.from_method.name.clone();
                    }

                    last_peak_earning.amount += tx.amount;
                }
                TxType::Expense => {
                    total_expense += tx.amount;
                    let amount = method_expense
                        .entry(tx.from_method.name.clone())
                        .or_insert(Cent::new(0));

                    *amount += tx.amount;

                    if biggest_expense.amount < tx.amount {
                        biggest_expense.amount = tx.amount;
                        biggest_expense.date = tx.date.date();
                        biggest_expense.method = tx.from_method.name.clone();
                    }

                    last_peak_expense.amount += tx.amount;
                }
                TxType::Transfer => {}
            }
        }

        total_month_checked += 1;

        if last_peak_earning.amount > peak_earning.amount {
            peak_earning = last_peak_earning;
            peak_earning.date = ongoing_date;
        }

        if last_peak_expense.amount > peak_expense.amount {
            peak_expense = last_peak_expense;
            peak_expense.date = ongoing_date;
        }

        let (income_percentage, expense_percentage) =
            get_percentages(total_income.value() as f64, total_expense.value() as f64);

        let mut average_income = if total_income == 0 {
            Some(Dollar::new(0.0))
        } else {
            Some(total_income.dollar() / f64::from(total_month_checked))
        };

        let mut average_expense = if total_income == 0 {
            Some(Dollar::new(0.0))
        } else {
            Some(total_expense.dollar() / f64::from(total_month_checked))
        };

        let mut method_data = Vec::new();

        for (index, method) in conn.cache().get_methods().iter().enumerate() {
            // For % calculations, it's safe to directly cast to f64 before calculations

            let earning_percentage = if method_earning[&method.name].value() == 0 {
                0.0
            } else {
                (method_earning[&method.name].value() as f64 / total_income.value() as f64) * 100.0
            };

            let expense_percentage = if method_expense[&method.name] == 0 {
                0.0
            } else {
                (method_expense[&method.name].value() as f64 / total_expense.value() as f64) * 100.0
            };

            let mut average_earning = if method_earning[&method.name] == 0 {
                Some(Dollar::new(0.0))
            } else {
                Some(method_earning[&method.name].dollar() / f64::from(total_month_checked))
            };

            let mut average_expense = if method_expense[&method.name] == 0 {
                Some(Dollar::new(0.0))
            } else {
                Some(method_expense[&method.name].dollar() / f64::from(total_month_checked))
            };

            let mut mom_yoy_earning = None;
            let mut mom_yoy_expense = None;

            if let FetchNature::Monthly = self.nature {
                average_earning = None;
                average_expense = None;
            }

            if let Some(last_summary) = last_summary
                && !no_mom_yoy
            {
                let comparison = &last_summary.methods;

                let last_earning = comparison[index].total_earning;
                let last_expense = comparison[index].total_expense;

                let current_earning = method_earning[&method.name].dollar();
                let current_expense = method_expense[&method.name].dollar();

                let earning_increased_percentage =
                    ((current_earning - last_earning) / last_earning) * 100.0;

                if last_earning == 0.0 {
                    mom_yoy_earning = Some("∞".to_string());
                } else if earning_increased_percentage < 0.0 {
                    mom_yoy_earning = Some(format!("↓{:.2}", earning_increased_percentage.abs()));
                } else {
                    mom_yoy_earning = Some(format!("↑{earning_increased_percentage:.2}"));
                }

                let expense_increased_percentage =
                    ((current_expense - last_expense) / last_expense) * 100.0;

                if last_expense == 0.0 {
                    mom_yoy_expense = Some("∞".to_string());
                } else if expense_increased_percentage < 0.0 {
                    mom_yoy_expense = Some(format!("↓{:.2}", expense_increased_percentage.abs()));
                } else {
                    mom_yoy_expense = Some(format!("↑{expense_increased_percentage:.2}"));
                }
            }

            if !no_mom_yoy && mom_yoy_expense.is_none() && mom_yoy_earning.is_none() {
                mom_yoy_earning = Some("∞".to_string());
                mom_yoy_expense = Some("∞".to_string());
            }

            let method_summary = SummaryMethods::new(
                method.name.to_string(),
                method_earning[&method.name].dollar(),
                method_expense[&method.name].dollar(),
                earning_percentage,
                expense_percentage,
                average_earning,
                average_expense,
                mom_yoy_earning,
                mom_yoy_expense,
            );

            method_data.push(method_summary);
        }

        if let FetchNature::Monthly = self.nature {
            average_income = None;
            average_expense = None;
        }

        let mut net_mom_yoy_earning = None;
        let mut net_mom_yoy_expense = None;

        if let Some(last_summary) = last_summary
            && !no_mom_yoy
        {
            let comparison = &last_summary.net;

            let last_earning = comparison.total_income;
            let last_expense = comparison.total_expense;

            let earning_increased_percentage =
                ((total_income.dollar() - last_earning) / last_earning) * 100.0;

            if last_earning == 0.0 {
                net_mom_yoy_earning = Some("∞".to_string());
            } else if earning_increased_percentage < 0.0 {
                net_mom_yoy_earning = Some(format!("↓{:.2}", earning_increased_percentage.abs()));
            } else {
                net_mom_yoy_earning = Some(format!("↑{earning_increased_percentage:.2}"));
            }

            let expense_increased_percentage =
                ((total_expense.dollar() - last_expense) / last_expense) * 100.0;

            if last_expense == 0.0 {
                net_mom_yoy_expense = Some("∞".to_string());
            } else if expense_increased_percentage < 0.0 {
                net_mom_yoy_expense = Some(format!("↓{:.2}", expense_increased_percentage.abs()));
            } else {
                net_mom_yoy_expense = Some(format!("↑{expense_increased_percentage:.2}"));
            }
        }

        if !no_mom_yoy && net_mom_yoy_expense.is_none() && net_mom_yoy_earning.is_none() {
            net_mom_yoy_earning = Some("∞".to_string());
            net_mom_yoy_expense = Some("∞".to_string());
        }

        let summary_net = SummaryNet::new(
            total_income.dollar(),
            total_expense.dollar(),
            average_income,
            average_expense,
            income_percentage,
            expense_percentage,
            net_mom_yoy_earning,
            net_mom_yoy_expense,
        );

        let summary_largest = vec![
            SummaryLargest::new(
                LargestType::Earning,
                biggest_earning.method,
                biggest_earning.amount.dollar(),
                biggest_earning.date,
            ),
            SummaryLargest::new(
                LargestType::Expense,
                biggest_expense.method,
                biggest_expense.amount.dollar(),
                biggest_expense.date,
            ),
        ];

        let summary_peak = vec![
            SummaryPeak::new(
                PeakType::Earning,
                peak_earning.amount.dollar(),
                peak_earning.date,
            ),
            SummaryPeak::new(
                PeakType::Expense,
                peak_expense.amount.dollar(),
                peak_expense.date,
            ),
        ];

        FullSummary {
            methods: method_data,
            net: summary_net,
            largest: summary_largest,
            peak: summary_peak,
        }
    }
}
