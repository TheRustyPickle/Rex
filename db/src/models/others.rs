use chrono::NaiveDate;

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
    pub fn extract(self) -> i64 {
        match self {
            AmountNature::Exact(i) => i,
            AmountNature::MoreThan(i) => i,
            AmountNature::MoreThanEqual(i) => i,
            AmountNature::LessThan(i) => i,
            AmountNature::LessThanEqual(i) => i,
        }
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
