use chrono::{NaiveTime, NaiveDate, DateTime, Weekday};
use regex::{Regex, CaptureMatches};

use crate::core_functions::{get_start_of_week, uk_datetime_now, weekday_matcher};

use super::timing::Timing;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Schedule {
    // #[serde(with ="naive_date_serialize")]
    #[serde(skip_serializing)]
    week_start: NaiveDate,
    timings: Vec<Timing>,
    #[serde(skip_serializing)]
    schedule_regex: Regex,
    #[serde(skip_serializing)]
    timing_regex: Regex,
}

impl Schedule {
    pub fn empty() -> Self {
        Self {
            week_start: get_start_of_week::get(uk_datetime_now::now().date_naive()),
            timings: Vec::new(),
            schedule_regex: Regex::new(r"(.*)\sto\s(.*)|CLOSED").unwrap(),
            timing_regex: Regex::new(r"(\d+).(\d+)(.*)").unwrap()
        }
    }

    pub fn form_schedule_timings(mut matches: CaptureMatches) -> Option<Self> {
        let mut schedule = Self {
            week_start: get_start_of_week::get(uk_datetime_now::now().date_naive()),
            timings: Vec::with_capacity(7),
            schedule_regex: Regex::new(r"(.*)\sto\s(.*)|CLOSED").unwrap(),
            timing_regex: Regex::new(r"(\d+).(\d+)(.*)").unwrap()
        };

        // To skip base 
        // matches.next();
        while let Some(inner_html) = matches.next() {
            let timings = schedule.schedule_regex.captures(inner_html.get(1)?.as_str())?;
            let timings_match: &str = timings.get(1)?.as_str();
            if timings_match == "CLOSED" {
                schedule.timings.push(Timing::closed());
                continue;
            }
            let opening = timings_match;
            let closing = timings.get(2)?.as_str();
            schedule.timings.push(
                Timing::open(
                    schedule.get_naive_time_from_str(opening),
                    schedule.get_naive_time_from_str(closing),
                )
            );
        }
        Some(schedule) 
    }

    pub fn get_week_start(&self) -> &NaiveDate {
        &self.week_start
    }

    fn get_naive_time_from_str(&self, input: &str) -> NaiveTime {
        let regex_match = self.timing_regex.captures(input).unwrap();
        NaiveTime::parse_from_str(
            format!(
                "{}:{} {}",
                regex_match.get(1).unwrap().as_str().to_string(),
                regex_match.get(2).unwrap().as_str().to_string(),
                regex_match.get(3).unwrap().as_str().to_string(),
            ).as_str(),
            "%I:%M %p"
        ).unwrap()
    }

    pub fn get_timings_from_weekday(&self, weekday: Weekday) -> &Timing {
        let index = weekday_matcher::get_num(weekday);
        &self.timings[index]
    }

    pub fn get_timings_from_weekday_tomorrow(&self, weekday: Weekday) -> &Timing {
        let index = (weekday_matcher::get_num(weekday) + 1) % 7;
        &self.timings[index]
    }
}


impl PartialEq for Schedule{
    fn eq(&self, other: &Self) -> bool {
        self.week_start == other.week_start && self.timings == other.timings
    }
}

// mod naive_date_serialize {
//     use chrono::NaiveDate;
//     use serde::{self, Serializer};
//     pub fn serialize<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
//         serializer.serialize_str(&date.format("%Y-%m-%d").to_string(),)
//     }
// }
