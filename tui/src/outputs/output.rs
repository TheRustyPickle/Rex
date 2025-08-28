use std::fmt;

pub enum HandlingOutput {
    QuitUi,
    TakeUserInput,
    PrintNewUpdate,
}

#[derive(PartialEq, Debug)]
pub enum VerifyingOutput {
    Nothing(AType),
    Accepted(AType),
    NotAccepted(NAType),
}

impl fmt::Display for VerifyingOutput {
    #[cfg(not(tarpaulin_include))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VerifyingOutput::Nothing(value) => write!(f, "{value}: Nothing to check"),
            VerifyingOutput::Accepted(value) => write!(f, "{value}: Accepted"),
            VerifyingOutput::NotAccepted(value) => write!(f, "{value}"),
        }
    }
}

/// Accepted Type
#[derive(PartialEq, Debug)]
pub enum AType {
    Date,
    TxMethod,
    Amount,
    TxType,
    Tags,
}

impl fmt::Display for AType {
    #[cfg(not(tarpaulin_include))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AType::Date => write!(f, "Date"),
            AType::TxMethod => write!(f, "Tx Method"),
            AType::Amount => write!(f, "Amount"),
            AType::TxType => write!(f, "Tx Type"),
            AType::Tags => write!(f, "Tags"),
        }
    }
}

/// Non Accepted Type
#[derive(PartialEq, Debug)]
pub enum NAType {
    InvalidDate,
    InvalidYear,
    InvalidMonth,
    InvalidDay,
    YearTooBig,
    MonthTooBig,
    DayTooBig,
    NonExistingDate,
    AmountBelowZero,
    InvalidTxMethod,
    InvalidTxType,
    ParsingError(AType),
    InvalidBValue,
    NonExistingTag,
}

impl fmt::Display for NAType {
    #[cfg(not(tarpaulin_include))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NAType::InvalidDate => write!(f, "Date: Unknown date"),
            NAType::InvalidYear => write!(
                f,
                "Date: Year length not acceptable. Example Date: 2022-05-01"
            ),
            NAType::InvalidMonth => write!(
                f,
                "Date: Month length not acceptable. Example Date: 2022-05-01"
            ),
            NAType::InvalidDay => write!(
                f,
                "Date: Day length not acceptable. Example Date: 2022-05-01"
            ),
            NAType::YearTooBig => write!(f, "Date: Year must be between 2022-2037"),
            NAType::MonthTooBig => write!(f, "Date: Month must be between 01-12"),
            NAType::DayTooBig => write!(f, "Date: Day must be between 01-31"),
            NAType::NonExistingDate => {
                write!(f, "Date: Date not acceptable and possibly non-existing")
            }
            NAType::AmountBelowZero => write!(f, "Amount: Value must be bigger than zero"),
            NAType::InvalidTxMethod => write!(f, "TX Method: Transaction Method not found"),
            NAType::InvalidTxType => write!(
                f,
                "TX Type: Transaction Type not acceptable. Values: Expense/Income/E/I"
            ),
            NAType::ParsingError(error) => {
                write!(f, "{error}: Error acquired while validating input")
            }
            NAType::InvalidBValue => write!(
                f,
                "Amount: TX Method cannot be empty. Value of B cannot be determined"
            ),
            NAType::NonExistingTag => write!(f, "Tags: Non-existing tags cannot be accepted"),
        }
    }
}

pub enum StepType {
    StepUp,
    StepDown,
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
    #[cfg(not(tarpaulin_include))]
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
