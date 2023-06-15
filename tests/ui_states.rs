extern crate rex_tui;
use chrono::{Datelike, Local};
use rex_tui::db::{MODES, MONTHS, YEARS};
use rex_tui::page_handler::*;

#[test]
fn test_table_data() {
    let table_data = vec![
        vec![
            "15-05-2022".to_string(),
            "Testing transaction".to_string(),
            "test 2".to_string(),
            "100.00".to_string(),
            "Expense".to_string(),
            "Unknown".to_string(),
        ],
        vec![
            "20-05-2022".to_string(),
            "Testing transaction".to_string(),
            "test 2".to_string(),
            "100.00".to_string(),
            "Income".to_string(),
            "Unknown".to_string(),
        ],
        vec![
            "25-05-2022".to_string(),
            "Testing transfer".to_string(),
            "test 2 to test1".to_string(),
            "100.00".to_string(),
            "Transfer".to_string(),
            "Unknown".to_string(),
        ],
    ];
    let mut table = TableData::new(table_data);
    assert_eq!(table.state.selected(), None);

    table.next();
    assert_eq!(table.state.selected(), Some(0));

    table.state.select(None);
    table.previous();
    assert_eq!(table.state.selected(), Some(0));

    table.state.select(Some(0));
    table.next();
    table.next();
    assert_eq!(table.state.selected(), Some(2));
    table.next();
    table.next();
    assert_eq!(table.state.selected(), Some(1));
    table.previous();
    table.previous();
    assert_eq!(table.state.selected(), Some(2));
}

#[test]
fn test_indexed_data() {
    let local_month_index = Local::now().month() as usize - 1;
    let local_year_index = Local::now().year() as usize - 2022;

    let mut index_data_monthly = IndexedData::new_monthly();
    let mut index_data_yearly = IndexedData::new_yearly();
    let index_data_modes = IndexedData::new_modes();

    assert_eq!(index_data_monthly.titles, MONTHS);
    assert_eq!(index_data_yearly.titles, YEARS);
    assert_eq!(index_data_modes.titles, MODES);

    assert_eq!(index_data_monthly.index, local_month_index);
    assert_eq!(index_data_yearly.index, local_year_index);
    assert_eq!(index_data_modes.index, 0);

    index_data_yearly.set_index_zero();
    index_data_monthly.set_index_zero();
    assert_eq!(index_data_yearly.index, 0);
    assert_eq!(index_data_monthly.index, 0);

    index_data_monthly.next();
    index_data_monthly.next();
    assert_eq!(index_data_monthly.index, 2);

    index_data_yearly.previous();
    index_data_yearly.previous();
    assert_eq!(index_data_yearly.index, YEARS.len() - 2);
}
