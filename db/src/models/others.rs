use chrono::NaiveDate;
use std::fmt::{self, Display};

#[derive(Clone, Debug, Copy)]
pub enum TxType {
    Income,
    Expense,
    Transfer,
}

#[derive(Clone, Debug, Copy, Eq, PartialEq)]
pub enum FetchNature {
    Monthly,
    Yearly,
    All,
}

pub enum DateNature {
    Exact(NaiveDate),
    ByMonth {
        start_date: NaiveDate,
        end_date: NaiveDate,
    },
    ByYear {
        start_date: NaiveDate,
        end_date: NaiveDate,
    },
}

pub enum AmountNature {
    Exact(i64),
    MoreThan(i64),
    MoreThanEqual(i64),
    LessThan(i64),
    LessThanEqual(i64),
}

impl AmountNature {
    #[must_use]
    pub fn extract(&self) -> i64 {
        let i = match self {
            AmountNature::Exact(i) => i,
            AmountNature::MoreThan(i) => i,
            AmountNature::MoreThanEqual(i) => i,
            AmountNature::LessThan(i) => i,
            AmountNature::LessThanEqual(i) => i,
        };

        *i
    }

    #[must_use]
    pub fn to_type(&self) -> AmountType {
        match self {
            AmountNature::Exact(_) => AmountType::Exact,
            AmountNature::MoreThan(_) => AmountType::MoreThan,
            AmountNature::MoreThanEqual(_) => AmountType::MoreThanEqual,
            AmountNature::LessThan(_) => AmountType::LessThan,
            AmountNature::LessThanEqual(_) => AmountType::LessThanEqual,
        }
    }

    #[must_use]
    pub fn from_type(t: AmountType, amount: i64) -> Self {
        match t {
            AmountType::Exact => AmountNature::Exact(amount),
            AmountType::MoreThan => AmountNature::MoreThan(amount),
            AmountType::MoreThanEqual => AmountNature::MoreThanEqual(amount),
            AmountType::LessThan => AmountNature::LessThan(amount),
            AmountType::LessThanEqual => AmountNature::LessThanEqual(amount),
        }
    }
}

impl Display for AmountNature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let a = match self {
            AmountNature::Exact(v) => format!("{}", *v as f64 / 100.0),
            AmountNature::MoreThan(v) => format!(">{}", *v as f64 / 100.0),
            AmountNature::MoreThanEqual(v) => format!(">={}", *v as f64 / 100.0),
            AmountNature::LessThan(v) => format!("<{}", *v as f64 / 100.0),
            AmountNature::LessThanEqual(v) => format!("<={}", *v as f64 / 100.0),
        };
        write!(f, "{a}")
    }
}

pub enum AmountType {
    Exact,
    MoreThan,
    MoreThanEqual,
    LessThan,
    LessThanEqual,
}

impl From<AmountType> for String {
    fn from(value: AmountType) -> Self {
        match value {
            AmountType::Exact => "exact".to_string(),
            AmountType::MoreThan => "more_than".to_string(),
            AmountType::MoreThanEqual => "more_than_equal".to_string(),
            AmountType::LessThan => "less_than".to_string(),
            AmountType::LessThanEqual => "less_than_equal".to_string(),
        }
    }
}

impl From<&str> for AmountType {
    fn from(value: &str) -> Self {
        match value {
            "exact" => AmountType::Exact,
            "more_than" => AmountType::MoreThan,
            "more_than_equal" => AmountType::MoreThanEqual,
            "less_than" => AmountType::LessThan,
            "less_than_equal" => AmountType::LessThanEqual,
            other => panic!("Invalid AmountType string: {other}"),
        }
    }
}

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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActivityNature::AddTx => write!(f, "add_tx"),
            ActivityNature::EditTx => write!(f, "edit_tx"),
            ActivityNature::DeleteTx => write!(f, "delete_tx"),
            ActivityNature::SearchTx => write!(f, "search_tx"),
            ActivityNature::PositionSwap => write!(f, "position_swap"),
        }
    }
}

impl From<&str> for TxType {
    fn from(s: &str) -> Self {
        match s {
            "Income" => TxType::Income,
            "Expense" => TxType::Expense,
            "Transfer" => TxType::Transfer,
            other => panic!("Invalid TxType string: {other}"),
        }
    }
}

impl Display for TxType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TxType::Income => "Income",
            TxType::Expense => "Expense",
            TxType::Transfer => "Transfer",
        };
        write!(f, "{s}")
    }
}
