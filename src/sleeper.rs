use std::ops::Add;

use chrono::{Datelike, Duration, NaiveTime, Timelike};

use crate::web_scraper::{schedule::Schedule, timing::Timing};
use crate::core_functions::{error_logger::error_logger, uk_datetime_now};

pub struct Sleeper {
    frequency: u64,
    error_time: u64,
    schedule: Option<Schedule>,
    default: Timing
}

impl Sleeper {
    pub fn new(frequency: u64, error_time: u64, schedule: Option<Schedule>) -> Self {
        let default = Timing::open(
            NaiveTime::from_hms_opt(6, 30, 0).expect("Invalid time"),
            NaiveTime::from_hms_opt(10, 30, 00).expect("Invalid time")
        );
        Self {
            frequency,
            error_time,
            schedule,
            default
        }
    }

    pub fn set_schedule(&mut self, schedule: Schedule) {
        self.schedule = Some(schedule);
    }

    pub async fn sleep(&self) {
        let schedule = match &self.schedule {
            Some(schedule) => schedule,
            None => {
                error_logger("No schedule. Error Sleep").await;
                //TODO: add error sleeper
                return;
            }
        };
        let now = uk_datetime_now::now();
        let now_time = now.time();
        let weekday = now.weekday();
        let timing: &Timing = schedule.get_timings_from_weekday(weekday);
        let day_end = NaiveTime::from_hms_opt(23, 59, 59).unwrap();
        // Check if open today
        if !timing.is_open() {
            // TODO: HANDLE CLOSED 
            let default_open = self.default.get_opening().unwrap();
            let diff = day_end - now_time + Duration::minutes(
                (default_open.hour() * 60 + default_open.minute()) as i64
            );
            Self::async_sleep(diff).await;
        }

        let opening_time = timing.get_opening().unwrap();
        let closing_time = timing.get_closing().unwrap();

        // Check if within opening hours
        if opening_time < now_time && now_time < closing_time {
            let now_second_stamp: u64 = (now.minute() * 60 + now_time.second()).into();
            let diff = self.frequency - (now_second_stamp % self.frequency);
            Self::async_sleep(Duration::seconds(diff as i64)).await;
            return;
        }
        else if opening_time > now_time {
            // Too early
            let diff = opening_time - now_time;
            Self::async_sleep(diff).await;
        }
        else if opening_time > closing_time {
            // Too late
            let tomorrow_opening_time = schedule.get_timings_from_weekday_tomorrow(weekday);
            if !tomorrow_opening_time.is_open() {
                //TODO: handle closed days
                // For now let's sleep to the default timing 
                let default_open = self.default.get_opening().unwrap();
                let diff = day_end - now_time + Duration::minutes(
                    (default_open.hour() * 60 + default_open.minute()) as i64
                );
                Self::async_sleep(diff).await;
            }
            let tomorrow_opening_time = tomorrow_opening_time.get_opening().unwrap();
            let diff: Duration = day_end - now_time + Duration::minutes(
                (tomorrow_opening_time.hour() * 60 + tomorrow_opening_time.minute()) as i64
            );
            Self::async_sleep(diff).await;
            // Schedule CAN change - Especially when it's Sunday. TOOD: take care of it
        }
    }

    async fn async_sleep(diff: Duration) {
        println!("Sleeping {} seconds", diff.num_seconds());
        let std_duration = std::time::Duration::new(diff.num_seconds() as u64, 0);
        tokio::time::sleep(std_duration).await;
    }

    pub async fn async_sleep_error(&self) {
        println!("Sleeping {} seconds", self.error_time);
        tokio::time::sleep(std::time::Duration::new(self.error_time, 0)).await;
    }

    pub fn is_standard_interval(&self) -> Option<bool> {
        let now = uk_datetime_now::now();
        let now_time = now.time();
        let weekday = now.weekday();
        let schedule = &self.schedule.as_ref()?;
        let timing: &Timing = schedule.get_timings_from_weekday(weekday);

        let opening_time = timing.get_opening()?;
        let closing_time = timing.get_closing()?;

        Some(opening_time <= now_time && now_time < closing_time)
    }

    pub fn get_frequency(&self) -> u64 {
        self.frequency
    }

    pub fn get_schedule(&self) -> &Schedule {
        &self.schedule.as_ref().unwrap()
    }
}
