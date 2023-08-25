use chrono::{Datelike, NaiveDate, Weekday};

use super::weekday_matcher::get_num;

pub fn get(date: NaiveDate) -> NaiveDate {
    let weekday = date.weekday();
    let weekday_number = get_num(weekday);
    date - chrono::Duration::days(weekday_number as i64)
}
