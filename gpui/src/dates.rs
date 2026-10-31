//! Calendar arithmetic on plain integers, so nothing depends on time zones and both
//! targets agree. `web/src/core/dates.js` is the same code. Days are counted from
//! 1970-01-01.

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

impl Date {
    pub fn new(year: i32, month: u32, day: u32) -> Self {
        Self { year, month, day }
    }

    pub fn to_days(self) -> i64 {
        let (month, day) = (i64::from(self.month), i64::from(self.day));
        let y = i64::from(self.year) - i64::from(month <= 2);
        let era = y.div_euclid(400);
        let year_of_era = y - era * 400;
        let day_of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
        let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
        era * 146097 + day_of_era - 719468
    }

    pub fn from_days(days: i64) -> Self {
        let shifted = days + 719468;
        let era = shifted.div_euclid(146097);
        let day_of_era = shifted - era * 146097;
        let year_of_era = (day_of_era - day_of_era / 1460 + day_of_era / 36524 - day_of_era / 146096) / 365;
        let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
        let shifted_month = (5 * day_of_year + 2) / 153;
        let month = if shifted_month < 10 { shifted_month + 3 } else { shifted_month - 9 };
        Self {
            year: (year_of_era + era * 400 + i64::from(month <= 2)) as i32,
            month: month as u32,
            day: (day_of_year - (153 * shifted_month + 2) / 5 + 1) as u32,
        }
    }

    /// Day of the week, 0 for Monday.
    pub fn weekday(self) -> u32 {
        (self.to_days() + 3).rem_euclid(7) as u32
    }

    pub fn plus_days(self, days: i64) -> Self {
        Self::from_days(self.to_days() + days)
    }

    pub fn plus_months(self, count: i32) -> Self {
        let total = self.year * 12 + (self.month as i32 - 1) + count;
        let year = total.div_euclid(12);
        let month = (total.rem_euclid(12) + 1) as u32;
        let next = if month == 12 { Date::new(year + 1, 1, 1) } else { Date::new(year, month + 1, 1) };
        let last_day = next.plus_days(-1).day;
        Self { year, month, day: self.day.min(last_day) }
    }

    pub fn iso(self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

/// The six weeks shown for a month, Monday first.
pub fn month_grid(year: i32, month: u32) -> Vec<Date> {
    let first = Date::new(year, month, 1);
    let start = first.to_days() - i64::from(first.weekday());
    (0..42).map(|index| Date::from_days(start + index)).collect()
}
