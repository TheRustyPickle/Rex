use chrono::NaiveDate;
use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, AddAssign, Div, Mul, Sub};

impl Add<i64> for Cent {
    type Output = Cent;

    fn add(self, rhs: i64) -> Self::Output {
        Cent(self.0 + rhs)
    }
}

impl Add<f64> for Dollar {
    type Output = Dollar;

    fn add(self, rhs: f64) -> Self::Output {
        Dollar(self.0 + rhs)
    }
}

impl Sub<i64> for Cent {
    type Output = Cent;

    fn sub(self, rhs: i64) -> Self::Output {
        Cent(self.0 - rhs)
    }
}

impl Sub<f64> for Dollar {
    type Output = Dollar;

    fn sub(self, rhs: f64) -> Self::Output {
        Dollar(self.0 - rhs)
    }
}

impl Sub for Dollar {
    type Output = Dollar;

    fn sub(self, rhs: Dollar) -> Self::Output {
        Dollar(self.0 - rhs.0)
    }
}

impl Div for Dollar {
    type Output = f64;

    fn div(self, rhs: Dollar) -> Self::Output {
        self.0 / rhs.0
    }
}

impl Mul<i64> for Cent {
    type Output = Cent;

    fn mul(self, rhs: i64) -> Self::Output {
        Cent(self.0 * rhs)
    }
}

impl Mul<f64> for Dollar {
    type Output = Dollar;

    fn mul(self, rhs: f64) -> Self::Output {
        Dollar(self.0 * rhs)
    }
}

impl AddAssign<f64> for Dollar {
    fn add_assign(&mut self, rhs: f64) {
        self.0 += rhs;
    }
}

impl AddAssign<i64> for Cent {
    fn add_assign(&mut self, rhs: i64) {
        self.0 += rhs;
    }
}

impl Div<i64> for Cent {
    type Output = Cent;

    fn div(self, rhs: i64) -> Self::Output {
        Cent(self.0 / rhs)
    }
}

impl Div<f64> for Dollar {
    type Output = Dollar;

    fn div(self, rhs: f64) -> Self::Output {
        Dollar(self.0 / rhs)
    }
}

impl PartialEq<i64> for Cent {
    fn eq(&self, other: &i64) -> bool {
        self.0 == *other
    }
}

impl PartialEq<f64> for Dollar {
    fn eq(&self, other: &f64) -> bool {
        self.0 == *other
    }
}

impl PartialEq for Cent {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialEq for Dollar {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialOrd for Cent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl PartialOrd for Dollar {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl PartialOrd<i64> for Cent {
    fn partial_cmp(&self, other: &i64) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}

impl PartialOrd<f64> for Dollar {
    fn partial_cmp(&self, other: &f64) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Cent(i64);

#[derive(Debug, Clone, Copy, Default)]
pub struct Dollar(f64);

impl Cent {
    pub(crate) fn new(value: i64) -> Self {
        Self(value)
    }

    pub(crate) fn dollar(&self) -> Dollar {
        Dollar::new(self.0 as f64 / 100.0)
    }

    pub(crate) fn value(&self) -> i64 {
        self.0
    }
}

impl Dollar {
    pub(crate) fn new(value: f64) -> Self {
        Self(value)
    }

    pub(crate) fn value(&self) -> f64 {
        self.0
    }
}

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

#[derive(PartialEq, Debug)]
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
            to_return[0].push(mom_yoy_earning.to_string());
        }

        if let Some(mom_yoy_expense) = &self.mom_yoy_expense {
            to_return[0].push(mom_yoy_expense.to_string());
        }

        to_return
    }
}

#[derive(PartialEq, Debug)]
pub(crate) enum LargestType {
    Earning,
    Expense,
}

#[derive(PartialEq, Debug)]
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

#[derive(PartialEq, Debug)]
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
                self.method.to_string(),
            ]
        } else {
            vec![
                self.largest_type.to_string(),
                self.date.format("%d-%m-%Y").to_string(),
                format!("{:.2}", self.amount.value()),
                self.method.to_string(),
            ]
        }
    }
}

#[derive(PartialEq, Debug)]
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

#[derive(Debug, PartialEq)]
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
            to_return.push(mom_yoy_earning.to_string());
        }
        if let Some(mom_yoy_expense) = &self.mom_yoy_expense {
            to_return.push(mom_yoy_expense.to_string());
        }

        to_return
    }
}
