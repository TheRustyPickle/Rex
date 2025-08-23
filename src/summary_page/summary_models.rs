use std::fmt;

#[derive(Debug)]
pub struct SummaryNet {
    pub total_income: f64,
    pub total_expense: f64,
    average_income: Option<f64>,
    average_expense: Option<f64>,
    income_percentage: f64,
    expense_percentage: f64,
    mom_yoy_earning: Option<String>,
    mom_yoy_expense: Option<String>,
}

impl SummaryNet {
    pub fn new(
        total_income: f64,
        total_expense: f64,
        average_income: Option<f64>,
        average_expense: Option<f64>,
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

    pub fn array(self) -> Vec<Vec<String>> {
        let mut to_return = if let Some(average_income) = self.average_income
            && let Some(average_expense) = self.average_expense
        {
            vec![vec![
                "Net".to_string(),
                format!("{:.2}", self.total_income),
                format!("{:.2}", self.total_expense),
                format!("{:.2}", average_income),
                format!("{:.2}", average_expense),
                format!("{:.2}", self.income_percentage),
                format!("{:.2}", self.expense_percentage),
            ]]
        } else {
            vec![vec![
                "Net".to_string(),
                format!("{:.2}", self.total_income),
                format!("{:.2}", self.total_expense),
                format!("{:.2}", self.income_percentage),
                format!("{:.2}", self.expense_percentage),
            ]]
        };

        if let Some(mom_yoy_earning) = self.mom_yoy_earning {
            to_return[0].push(mom_yoy_earning);
        }

        if let Some(mom_yoy_expense) = self.mom_yoy_expense {
            to_return[0].push(mom_yoy_expense);
        }

        to_return
    }
}

pub enum LargestType {
    Earning,
    Expense,
}

pub enum PeakType {
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

pub struct SummaryLargest {
    largest_type: LargestType,
    method: String,
    amount: f64,
    date: String,
}

impl SummaryLargest {
    pub fn new(largest_type: LargestType, method: String, amount: f64, date: String) -> Self {
        Self {
            largest_type,
            method,
            amount,
            date,
        }
    }

    pub fn array(self) -> Vec<String> {
        vec![
            self.largest_type.to_string(),
            self.date,
            format!("{:.2}", self.amount),
            self.method,
        ]
    }
}

pub struct SummaryPeak {
    peak_type: PeakType,
    amount: f64,
    date: String,
}

impl SummaryPeak {
    pub fn new(peak_type: PeakType, amount: f64, date: String) -> Self {
        Self {
            peak_type,
            amount,
            date,
        }
    }

    pub fn array(self) -> Vec<String> {
        vec![
            self.peak_type.to_string(),
            self.date,
            format!("{:.2}", self.amount),
        ]
    }
}

#[derive(Debug)]
pub struct SummaryMethods {
    method: String,
    pub total_earning: f64,
    pub total_expense: f64,
    percentage_earning: f64,
    percentage_expense: f64,
    average_earning: Option<f64>,
    average_expense: Option<f64>,
    mom_yoy_earning: Option<String>,
    mom_yoy_expense: Option<String>,
}

impl SummaryMethods {
    pub fn new(
        method: String,
        total_earning: f64,
        total_expense: f64,
        percentage_earning: f64,
        percentage_expense: f64,
        average_earning: Option<f64>,
        average_expense: Option<f64>,
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

    pub fn array(self) -> Vec<String> {
        let mut to_return = if let Some(average_income) = self.average_earning
            && let Some(average_expense) = self.average_expense
        {
            vec![
                self.method,
                format!("{:.2}", self.total_earning),
                format!("{:.2}", self.total_expense),
                format!("{:.2}", average_income),
                format!("{:.2}", average_expense),
                format!("{:.2}", self.percentage_earning),
                format!("{:.2}", self.percentage_expense),
            ]
        } else {
            vec![
                self.method,
                format!("{:.2}", self.total_earning),
                format!("{:.2}", self.total_expense),
                format!("{:.2}", self.percentage_earning),
                format!("{:.2}", self.percentage_expense),
            ]
        };

        if let Some(mom_yoy_earning) = self.mom_yoy_earning {
            to_return.push(mom_yoy_earning);
        }
        if let Some(mom_yoy_expense) = self.mom_yoy_expense {
            to_return.push(mom_yoy_expense);
        }

        to_return
    }
}
