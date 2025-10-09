pub enum HandlingOutput {
    QuitUi,
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
