use chrono::NaiveDate;
use rex_shared::models::{Cent, Dollar};
use std::fmt;

#[derive(Default, Clone)]
pub(crate) struct LargestMomvement {
    pub(crate) date: NaiveDate,
    pub(crate) method: String,
    pub(crate) amount: Cent,
}

#[derive(Clone, Default, Copy)]
pub(crate) struct PeakMonthlyMovement {
    pub(crate) date: NaiveDate,
    pub(crate) amount: Cent,
}

impl PeakMonthlyMovement {
    pub(crate) fn new(date: NaiveDate) -> Self {
        Self {
            date,
            amount: Cent::new(0),
        }
    }
}

#[derive(Debug)]
pub(crate) struct SummaryNet {
    pub(crate) total_income: Dollar,
    pub(crate) total_expense: Dollar,
    average_income: Option<Dollar>,
    average_expense: Option<Dollar>,
    income_percentage: f64,
    expense_percentage: f64,
    mom_yoy_earning: Option<String>,
    mom_yoy_expense: Option<String>,
}

impl SummaryNet {
    #[must_use]
    pub(crate) fn new(
        total_income: Dollar,
        total_expense: Dollar,
        average_income: Option<Dollar>,
        average_expense: Option<Dollar>,
        income_percentage: f64,
        expense_percentage: f64,
        mom_yoy_earning: Option<String>,
        mom_yoy_expense: Option<String>,
    ) -> Self {
        Self {
            total_income,
            total_expense,
            average_income,
            average_expense,
            income_percentage,
            expense_percentage,
            mom_yoy_earning,
            mom_yoy_expense,
        }
    }

    #[must_use]
    pub(crate) fn array(&self) -> Vec<Vec<String>> {
        let mut to_return = if let Some(average_income) = self.average_income
            && let Some(average_expense) = self.average_expense
        {
            vec![vec![
                "Net".to_string(),
                format!("{:.2}", self.total_income.value()),
                format!("{:.2}", self.total_expense.value()),
                format!("{:.2}", average_income.value()),
                format!("{:.2}", average_expense.value()),
                format!("{:.2}", self.income_percentage),
                format!("{:.2}", self.expense_percentage),
            ]]
        } else {
            vec![vec![
                "Net".to_string(),
                format!("{:.2}", self.total_income.value()),
                format!("{:.2}", self.total_expense.value()),
                format!("{:.2}", self.income_percentage),
                format!("{:.2}", self.expense_percentage),
            ]]
        };

        if let Some(mom_yoy_earning) = &self.mom_yoy_earning {
            to_return[0].push(mom_yoy_earning.clone());
        }

        if let Some(mom_yoy_expense) = &self.mom_yoy_expense {
            to_return[0].push(mom_yoy_expense.clone());
        }

        to_return
    }
}

#[derive(Debug)]
pub(crate) enum LargestType {
    Earning,
    Expense,
}

#[derive(Debug)]
pub(crate) enum PeakType {
    Earning,
    Expense,
}

impl fmt::Display for LargestType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LargestType::Earning => write!(f, "Largest Earning"),
            LargestType::Expense => write!(f, "Largest Expense"),
        }
    }
}

impl fmt::Display for PeakType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PeakType::Earning => write!(f, "Peak Earning"),
            PeakType::Expense => write!(f, "Peak Expense"),
        }
    }
}

#[derive(Debug)]
pub(crate) struct SummaryLargest {
    largest_type: LargestType,
    method: String,
    amount: Dollar,
    date: NaiveDate,
}

impl SummaryLargest {
    #[must_use]
    pub(crate) fn new(
        largest_type: LargestType,
        method: String,
        amount: Dollar,
        date: NaiveDate,
    ) -> Self {
        Self {
            largest_type,
            method,
            amount,
            date,
        }
    }

    #[must_use]
    pub(crate) fn array(&self) -> Vec<String> {
        let default_date = NaiveDate::default();
        if self.date == default_date {
            vec![
                self.largest_type.to_string(),
                String::from("-"),
                format!("{:.2}", self.amount.value()),
                self.method.clone(),
            ]
        } else {
            vec![
                self.largest_type.to_string(),
                self.date.format("%d-%m-%Y").to_string(),
                format!("{:.2}", self.amount.value()),
                self.method.clone(),
            ]
        }
    }
}

#[derive(Debug)]
pub(crate) struct SummaryPeak {
    peak_type: PeakType,
    amount: Dollar,
    date: NaiveDate,
}

impl SummaryPeak {
    #[must_use]
    pub(crate) fn new(peak_type: PeakType, amount: Dollar, date: NaiveDate) -> Self {
        Self {
            peak_type,
            amount,
            date,
        }
    }

    #[must_use]
    pub(crate) fn array(&self) -> Vec<String> {
        let default_date = NaiveDate::default();
        if self.date == default_date {
            vec![
                self.peak_type.to_string(),
                String::from("-"),
                format!("{:.2}", self.amount.value()),
            ]
        } else {
            vec![
                self.peak_type.to_string(),
                self.date.format("%m-%Y").to_string(),
                format!("{:.2}", self.amount.value()),
            ]
        }
    }
}

#[derive(Debug)]
pub(crate) struct SummaryMethods {
    method: String,
    pub(crate) total_earning: Dollar,
    pub(crate) total_expense: Dollar,
    percentage_earning: f64,
    percentage_expense: f64,
    average_earning: Option<Dollar>,
    average_expense: Option<Dollar>,
    mom_yoy_earning: Option<String>,
    mom_yoy_expense: Option<String>,
}

impl SummaryMethods {
    #[must_use]
    pub(crate) fn new(
        method: String,
        total_earning: Dollar,
        total_expense: Dollar,
        percentage_earning: f64,
        percentage_expense: f64,
        average_earning: Option<Dollar>,
        average_expense: Option<Dollar>,
        mom_yoy_earning: Option<String>,
        mom_yoy_expense: Option<String>,
    ) -> Self {
        Self {
            method,
            total_earning,
            total_expense,
            percentage_earning,
            percentage_expense,
            average_earning,
            average_expense,
            mom_yoy_earning,
            mom_yoy_expense,
        }
    }

    #[must_use]
    pub(crate) fn array(&self) -> Vec<String> {
        let mut to_return = if let Some(average_income) = self.average_earning
            && let Some(average_expense) = self.average_expense
        {
            vec![
                self.method.clone(),
                format!("{:.2}", self.total_earning.value()),
                format!("{:.2}", self.total_expense.value()),
                format!("{:.2}", average_income.value()),
                format!("{:.2}", average_expense.value()),
                format!("{:.2}", self.percentage_earning),
                format!("{:.2}", self.percentage_expense),
            ]
        } else {
            vec![
                self.method.clone(),
                format!("{:.2}", self.total_earning.value()),
                format!("{:.2}", self.total_expense.value()),
                format!("{:.2}", self.percentage_earning),
                format!("{:.2}", self.percentage_expense),
            ]
        };

        if let Some(mom_yoy_earning) = &self.mom_yoy_earning {
            to_return.push(mom_yoy_earning.clone());
        }
        if let Some(mom_yoy_expense) = &self.mom_yoy_expense {
            to_return.push(mom_yoy_expense.clone());
        }

        to_return
    }
}

#[derive(Debug)]
pub(crate) struct SummaryLendBorrows {
    pub(crate) borrows: Dollar,
    pub(crate) lends: Dollar,
    mom_yoy_borrows: Option<String>,
    mom_yoy_lends: Option<String>,
}

impl SummaryLendBorrows {
    pub(crate) fn new(
        borrows: Dollar,
        lends: Dollar,
        mom_yoy_borrows: Option<String>,
        mom_yoy_lends: Option<String>,
    ) -> Self {
        Self {
            borrows,
            lends,
            mom_yoy_borrows,
            mom_yoy_lends,
        }
    }

    pub(crate) fn array(&self) -> Vec<String> {
        let mut to_return = vec![
            format!("{:.2}", self.borrows.value()),
            format!("{:.2}", self.lends.value()),
        ];

        if let Some(mom_yoy_borrows) = &self.mom_yoy_borrows {
            to_return.push(mom_yoy_borrows.clone());
        }
        if let Some(mom_yoy_lends) = &self.mom_yoy_lends {
            to_return.push(mom_yoy_lends.clone());
        }

        to_return
    }
}
