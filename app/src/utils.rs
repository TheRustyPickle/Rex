use anyhow::{Result, anyhow};
use chrono::NaiveDate;
use rex_db::models::AmountNature;
use rex_shared::models::{Cent, Dollar};

pub fn split_tags(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

pub fn parse_month_year(month: &str, year: &str) -> Result<NaiveDate> {
    let year_num = year.parse::<i32>()?;
    let month_num = month_name_to_num(month)?;

    NaiveDate::from_ymd_opt(year_num, month_num, 1)
        .ok_or_else(|| anyhow!("Invalid date: {year}-{month}-01"))
}

pub fn month_name_to_num(name: &str) -> Result<u32> {
    match name {
        "January" => Ok(1),
        "February" => Ok(2),
        "March" => Ok(3),
        "April" => Ok(4),
        "May" => Ok(5),
        "June" => Ok(6),
        "July" => Ok(7),
        "August" => Ok(8),
        "September" => Ok(9),
        "October" => Ok(10),
        "November" => Ok(11),
        "December" => Ok(12),
        _ => Err(anyhow!("Invalid month name {name}")),
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

pub fn parse_amount_nature_cent(amount: &str) -> Result<Option<AmountNature>> {
    if amount.trim().is_empty() {
        return Ok(None);
    }

    let parse = |s: &str| -> Result<Cent> { Ok(Dollar::new(s.parse()?).cent()) };

    let res = if let Some(rest) = amount.strip_prefix("<=") {
        AmountNature::LessThanEqual(parse(rest)?)
    } else if let Some(rest) = amount.strip_prefix(">=") {
        AmountNature::MoreThanEqual(parse(rest)?)
    } else if let Some(rest) = amount.strip_prefix('<') {
        AmountNature::LessThan(parse(rest)?)
    } else if let Some(rest) = amount.strip_prefix('>') {
        AmountNature::MoreThan(parse(rest)?)
    } else {
        AmountNature::Exact(parse(amount)?)
    };

    Ok(Some(res))
}

pub fn compare_change(current: Dollar, previous: Dollar) -> String {
    match current.cent().percent_change(previous.cent()) {
        None => "∞".to_string(),
        Some(diff) if diff < 0.0 => format!("↓{:.2}", diff.abs()),
        Some(diff) => format!("↑{diff:.2}"),
    }
}

pub fn compare_change_opt(current: Dollar, previous: Option<Dollar>) -> String {
    match previous {
        None => "∞".to_string(),
        Some(prev) => match current.cent().percent_change(prev.cent()) {
            None => "∞".to_string(),
            Some(diff) if diff < 0.0 => format!("↓{:.2}", diff.abs()),
            Some(diff) => format!("↑{diff:.2}"),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rex_db::models::AmountNature;
    use rex_shared::models::{Cent, Dollar};

    #[test]
    fn test_month_name_to_num_all_months() {
        assert_eq!(month_name_to_num("January").unwrap(), 1);
        assert_eq!(month_name_to_num("February").unwrap(), 2);
        assert_eq!(month_name_to_num("March").unwrap(), 3);
        assert_eq!(month_name_to_num("April").unwrap(), 4);
        assert_eq!(month_name_to_num("May").unwrap(), 5);
        assert_eq!(month_name_to_num("June").unwrap(), 6);
        assert_eq!(month_name_to_num("July").unwrap(), 7);
        assert_eq!(month_name_to_num("August").unwrap(), 8);
        assert_eq!(month_name_to_num("September").unwrap(), 9);
        assert_eq!(month_name_to_num("October").unwrap(), 10);
        assert_eq!(month_name_to_num("November").unwrap(), 11);
        assert_eq!(month_name_to_num("December").unwrap(), 12);
    }

    #[test]
    fn test_month_name_to_num_invalid_errors() {
        assert!(month_name_to_num("NotAMonth").is_err());
    }

    #[test]
    fn test_month_year_to_unique() {
        assert_eq!(month_year_to_unique(6, 2024), 202406);
        assert_eq!(month_year_to_unique(12, 2024), 202412);
        assert_eq!(month_year_to_unique(1, 2000), 200001);
    }

    #[test]
    fn test_get_percentages_normal() {
        let (p1, p2) = get_percentages(30.0, 70.0);
        assert!((p1 - 30.0).abs() < 0.001);
        assert!((p2 - 70.0).abs() < 0.001);
    }

    #[test]
    fn test_get_percentages_equal() {
        let (p1, p2) = get_percentages(50.0, 50.0);
        assert!((p1 - 50.0).abs() < 0.001);
        assert!((p2 - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_get_percentages_both_zero() {
        let (p1, p2) = get_percentages(0.0, 0.0);
        assert_eq!(p1, 0.0);
        assert_eq!(p2, 0.0);
    }

    #[test]
    fn test_get_percentages_one_zero() {
        let (p1, p2) = get_percentages(0.0, 100.0);
        assert_eq!(p1, 0.0);
        assert!((p2 - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_parse_amount_nature_exact() {
        let result = parse_amount_nature_cent("50.00").unwrap().unwrap();
        assert!(matches!(result, AmountNature::Exact(c) if c == Cent::new(5000)));
    }

    #[test]
    fn test_parse_amount_nature_more_than() {
        let result = parse_amount_nature_cent(">100").unwrap().unwrap();
        assert!(matches!(result, AmountNature::MoreThan(c) if c == Cent::new(10000)));
    }

    #[test]
    fn test_parse_amount_nature_less_than() {
        let result = parse_amount_nature_cent("<50").unwrap().unwrap();
        assert!(matches!(result, AmountNature::LessThan(c) if c == Cent::new(5000)));
    }

    #[test]
    fn test_parse_amount_nature_more_than_equal() {
        let result = parse_amount_nature_cent(">=25.50").unwrap().unwrap();
        assert!(matches!(result, AmountNature::MoreThanEqual(c) if c == Cent::new(2550)));
    }

    #[test]
    fn test_parse_amount_nature_less_than_equal() {
        let result = parse_amount_nature_cent("<=10.00").unwrap().unwrap();
        assert!(matches!(result, AmountNature::LessThanEqual(c) if c == Cent::new(1000)));
    }

    #[test]
    fn test_parse_amount_nature_empty() {
        let result = parse_amount_nature_cent("  ").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_amount_nature_invalid() {
        assert!(parse_amount_nature_cent("abc").is_err());
    }

    #[test]
    fn test_compare_change_increase() {
        let result = compare_change(Dollar::new(150.0), Dollar::new(100.0));
        assert_eq!(result, "↑50.00");
    }

    #[test]
    fn test_compare_change_decrease() {
        let result = compare_change(Dollar::new(50.0), Dollar::new(100.0));
        assert_eq!(result, "↓50.00");
    }

    #[test]
    fn test_compare_change_zero_previous() {
        let result = compare_change(Dollar::new(100.0), Dollar::new(0.0));
        assert_eq!(result, "∞");
    }

    #[test]
    fn test_compare_change_opt_some() {
        let result = compare_change_opt(Dollar::new(200.0), Some(Dollar::new(100.0)));
        assert_eq!(result, "↑100.00");
    }

    #[test]
    fn test_compare_change_opt_none() {
        let result = compare_change_opt(Dollar::new(100.0), None);
        assert_eq!(result, "∞");
    }

    #[test]
    fn test_compare_change_opt_previous_zero() {
        let result = compare_change_opt(Dollar::new(100.0), Some(Dollar::new(0.0)));
        assert_eq!(result, "∞");
    }
}
