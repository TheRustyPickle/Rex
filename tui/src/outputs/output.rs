use std::fmt;

pub enum HandlingOutput {
    QuitUi,
    TakeUserInput,
    PrintNewUpdate,
}

#[derive(Debug, PartialEq)]
pub enum TxType {
    IncomeExpense,
    Transfer,
}

pub enum ComparisonType {
    Equal,
    BiggerThan,
    SmallerThan,
    EqualOrBigger,
    EqualOrSmaller,
}

impl fmt::Display for ComparisonType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComparisonType::Equal => write!(f, "amount ="),
            ComparisonType::BiggerThan => write!(f, "CAST(amount AS REAL) >"),
            ComparisonType::SmallerThan => write!(f, "CAST(amount AS REAL) <"),
            ComparisonType::EqualOrBigger => write!(f, "CAST(amount AS REAL) >="),
            ComparisonType::EqualOrSmaller => write!(f, "CAST(amount AS REAL) <="),
        }
    }
}
