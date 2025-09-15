pub fn month_name_to_num(name: &str) -> u32 {
    match name {
        "January" => 1,
        "February" => 2,
        "March" => 3,
        "April" => 4,
        "May" => 5,
        "June" => 6,
        "July" => 7,
        "August" => 8,
        "September" => 9,
        "October" => 10,
        "November" => 11,
        "December" => 12,
        _ => panic!("Invalid month name {name}"),
    }
}

pub fn month_year_to_unique(month: i32, year: i32) -> i32 {
    year * 100 + month
}

/// Takes 2 numbers and returns how much % are each of them
pub fn get_percentages(value1: f64, value2: f64) -> (f64, f64) {
    if value1 == 0.0 && value2 == 0.0 {
        return (0.0, 0.0);
    }
    let total = value1 + value2;
    let percentage1 = (value1 / total) * 100.0;
    let percentage2 = (value2 / total) * 100.0;
    (percentage1, percentage2)
}
