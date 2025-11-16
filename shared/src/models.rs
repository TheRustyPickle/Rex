use chrono::NaiveTime;
use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

pub const LAST_POSSIBLE_TIME: NaiveTime =
    NaiveTime::from_hms_nano_opt(23, 59, 59, 999_999_999).unwrap();

impl Cent {
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
