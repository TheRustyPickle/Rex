use chrono::NaiveTime;
use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

pub const LAST_POSSIBLE_TIME: NaiveTime =
    NaiveTime::from_hms_nano_opt(23, 59, 59, 999_999_999).unwrap();

impl Cent {
    #[must_use]
    pub fn percent_change(self, previous: Cent) -> Option<f64> {
        if previous.0 == 0 {
            None
        } else {
            let cur = self.0 as f64;
            let prev = previous.0 as f64;
            Some(((cur - prev) / prev) * 100.0)
        }
    }
}

impl Add<i64> for Cent {
    type Output = Cent;

    fn add(self, rhs: i64) -> Self::Output {
        Cent(self.0 + rhs)
    }
}

impl Sub<i64> for Cent {
    type Output = Cent;

    fn sub(self, rhs: i64) -> Self::Output {
        Cent(self.0 - rhs)
    }
}

impl Mul<i64> for Cent {
    type Output = Cent;

    fn mul(self, rhs: i64) -> Self::Output {
        Cent(self.0 * rhs)
    }
}

impl AddAssign<i64> for Cent {
    fn add_assign(&mut self, rhs: i64) {
        self.0 += rhs;
    }
}

impl SubAssign<i64> for Cent {
    fn sub_assign(&mut self, rhs: i64) {
        self.0 -= rhs;
    }
}

impl SubAssign for Cent {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl AddAssign for Cent {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl Div<f64> for Dollar {
    type Output = Dollar;

    fn div(self, rhs: f64) -> Self::Output {
        Dollar(self.0 / rhs)
    }
}

impl AddAssign<Cent> for i64 {
    fn add_assign(&mut self, rhs: Cent) {
        *self += rhs.0;
    }
}

impl PartialEq<Cent> for i64 {
    fn eq(&self, other: &Cent) -> bool {
        *self == other.0
    }
}

impl SubAssign<Cent> for i64 {
    fn sub_assign(&mut self, rhs: Cent) {
        *self -= rhs.0;
    }
}

impl PartialEq<i64> for Cent {
    fn eq(&self, other: &i64) -> bool {
        self.0 == *other
    }
}

impl PartialEq for Cent {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialOrd for Cent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl PartialOrd<i64> for Cent {
    fn partial_cmp(&self, other: &i64) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}

impl fmt::Display for Dollar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}", self.0)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Cent(i64);

#[derive(Debug, Clone, Copy, Default)]
pub struct Dollar(f64);

impl Cent {
    #[must_use]
    pub fn new(value: i64) -> Self {
        Self(value)
    }

    #[must_use]
    pub fn dollar(&self) -> Dollar {
        Dollar::new(self.0 as f64 / 100.0)
    }

    #[must_use]
    pub fn value(&self) -> i64 {
        self.0
    }
}

impl Dollar {
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value)
    }

    #[must_use]
    pub fn value(&self) -> f64 {
        self.0
    }

    #[must_use]
    pub fn cent(&self) -> Cent {
        Cent::new((self.0 * 100.0).round() as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cent_new_and_value() {
        let c = Cent::new(150);
        assert_eq!(c.value(), 150);
    }

    #[test]
    fn cent_default_is_zero() {
        assert_eq!(Cent::default(), Cent::new(0));
    }

    #[test]
    fn cent_to_dollar() {
        assert_eq!(Cent::new(150).dollar().value(), 1.50);
        assert_eq!(Cent::new(0).dollar().value(), 0.0);
        assert_eq!(Cent::new(-250).dollar().value(), -2.50);
    }

    #[test]
    fn dollar_to_cent() {
        assert_eq!(Dollar::new(1.50).cent(), Cent::new(150));
        assert_eq!(Dollar::new(0.0).cent(), Cent::new(0));
        assert_eq!(Dollar::new(-2.50).cent(), Cent::new(-250));
        assert_eq!(Dollar::new(1.999).cent(), Cent::new(200));
        assert_eq!(Dollar::new(1.004).cent(), Cent::new(100));
    }

    #[test]
    fn dollar_new_and_value() {
        let d = Dollar::new(42.75);
        assert_eq!(d.value(), 42.75);
    }

    #[test]
    fn dollar_display_formats_two_decimals() {
        assert_eq!(format!("{}", Dollar::new(5.0)), "5.00");
        assert_eq!(format!("{}", Dollar::new(3.456)), "3.46");
        assert_eq!(format!("{}", Dollar::new(0.0)), "0.00");
        assert_eq!(format!("{}", Dollar::new(1234.5)), "1234.50");
    }

    #[test]
    fn dollar_div() {
        assert_eq!((Dollar::new(10.0) / 2.0).value(), 5.0);
        assert_eq!((Dollar::new(3.0) / 2.0).value(), 1.5);
        assert_eq!((Dollar::new(0.0) / 5.0).value(), 0.0);
    }

    #[test]
    fn cent_add_i64() {
        assert_eq!(Cent::new(100) + 50, Cent::new(150));
        assert_eq!(Cent::new(-100) + 50, Cent::new(-50));
    }

    #[test]
    fn cent_sub_i64() {
        assert_eq!(Cent::new(100) - 50, Cent::new(50));
        assert_eq!(Cent::new(-100) - 50, Cent::new(-150));
    }

    #[test]
    fn cent_mul_i64() {
        assert_eq!(Cent::new(10) * 3, Cent::new(30));
        assert_eq!(Cent::new(-10) * 3, Cent::new(-30));
        assert_eq!(Cent::new(0) * 100, Cent::new(0));
    }

    #[test]
    fn cent_add_assign_i64() {
        let mut c = Cent::new(100);
        c += 50;
        assert_eq!(c, Cent::new(150));
    }

    #[test]
    fn cent_sub_assign_i64() {
        let mut c = Cent::new(100);
        c -= 50;
        assert_eq!(c, Cent::new(50));
    }

    #[test]
    fn cent_add_assign_cent() {
        let mut c = Cent::new(100);
        c += Cent::new(50);
        assert_eq!(c, Cent::new(150));
    }

    #[test]
    fn cent_sub_assign_cent() {
        let mut c = Cent::new(100);
        c -= Cent::new(50);
        assert_eq!(c, Cent::new(50));
    }

    #[test]
    fn cent_eq_cent() {
        assert_eq!(Cent::new(100), Cent::new(100));
        assert_ne!(Cent::new(100), Cent::new(200));
    }

    #[test]
    fn cent_eq_i64() {
        assert!(Cent::new(100) == 100);
        assert!(Cent::new(100) != 200);
    }

    #[test]
    fn i64_eq_cent() {
        assert!(100 == Cent::new(100));
    }

    #[test]
    fn cent_partial_cmp() {
        assert!(Cent::new(100) < Cent::new(200));
        assert!(Cent::new(200) > Cent::new(100));
        assert!(Cent::new(100) <= Cent::new(100));
        assert!(Cent::new(100) >= Cent::new(100));
    }

    #[test]
    fn cent_partial_cmp_i64() {
        assert!(Cent::new(100) < 200);
        assert!(Cent::new(200) > 100);
        assert!(Cent::new(100) <= 100);
    }

    #[test]
    fn i64_add_assign_cent() {
        let mut v: i64 = 100;
        v += Cent::new(50);
        assert_eq!(v, 150);
    }

    #[test]
    fn i64_sub_assign_cent() {
        let mut v: i64 = 100;
        v -= Cent::new(50);
        assert_eq!(v, 50);
    }

    #[test]
    fn cent_percent_change_normal() {
        let diff = Cent::new(150).percent_change(Cent::new(100)).unwrap();
        assert!((diff - 50.0).abs() < 0.001);
    }

    #[test]
    fn cent_percent_change_decrease() {
        let diff = Cent::new(50).percent_change(Cent::new(100)).unwrap();
        assert!((diff - (-50.0)).abs() < 0.001);
    }

    #[test]
    fn cent_percent_change_zero_previous() {
        assert_eq!(Cent::new(100).percent_change(Cent::new(0)), None);
    }
}
