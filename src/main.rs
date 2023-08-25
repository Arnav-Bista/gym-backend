mod core_functions;
mod web_scraper;
mod firebase;
mod sleeper;

use std::fs;

use chrono::{Datelike, DateTime};
use chrono_tz::Tz;
use core_functions::{uk_datetime_now, get_start_of_week, weekday_matcher, error_logger::error_logger};
use firebase::firebase::Firebase;
use serde_json::json;
use sleeper::Sleeper;
use web_scraper::extractor;

use tokio::{self, join, spawn};


#[tokio::main]
async fn main() {
    let mut extractor = extractor::Extractor::new_default();
    let db_url: String = fs::read_to_string("databaseUrl.secret").unwrap();
    let mut firebase = Firebase::new("serviceAccountKey.json.secret", db_url);
    let mut sleeper = Sleeper::new(10, 10, None);

    loop {
        let scrape_result = extractor.scrape().await;
        if scrape_result.is_err() {
            sleeper.async_sleep_error().await;
            continue;
        }
        let schedule = extractor.scrape_schedule().await;
        let occupancy = extractor.scrape_occupancy().await;

        if schedule.is_none() || occupancy.is_none() {
            sleeper.async_sleep_error().await;
            continue;
        }

        let schedule = schedule.expect("Unexpected Error");
        let occupancy = occupancy.expect("Unexpected Error");

        let schedule_data = json!(schedule).to_string();
        sleeper.set_schedule(schedule);

        let uk_now = uk_datetime_now::now();
        let key = uk_now.format("%H%M").to_string();
        let occupancy_data = prepare_occupancy_json(&key, occupancy);
        let latest_occupancy_location = format!("rs_data/data/latest/{}",uk_now.format("%Y-%m-%d"));
        let latest_schedule_location = "rs_data/data/latest/schedule";

        if firebase.handle_auth_token().await.is_err() {
            error_logger("Firebase Error - Auth Token").await;
            sleeper.async_sleep_error().await;
            continue;
        }



        let (occupancy_location, schedule_location) = prepare_location(uk_now);
        let data_insert = firebase.update(occupancy_location, &occupancy_data);
        let schedule_insert = firebase.set(schedule_location, &schedule_data);
        let latest_occupancy_set = firebase.set(latest_occupancy_location, &occupancy_data);
        let latest_schedule_set = firebase.set(latest_schedule_location.to_string(), &schedule_data);


        join!(
            sleeper.sleep(),
            data_insert,
            schedule_insert,
            latest_schedule_set,
            latest_occupancy_set,
        );
    }

}

fn prepare_occupancy_json(key: &str, occupancy: u8) -> String {
    format!("{{ \"{}\":{} }}",key, occupancy)
}

// Returns (Occupancy Location, Schedule Location)
fn prepare_location(now: DateTime<Tz>) -> (String, String) {
    let start_of_week = get_start_of_week::get(now.date_naive());
    let start_of_week = start_of_week.format("%Y-%m-%d");
    let weekday_num = weekday_matcher::get_num(now.weekday()).to_string();
    let occupancy_location = format!("rs_data/data/{}/{}",start_of_week, weekday_num);
    let schedule_location = format!("rs_data/data/schedule/{}", start_of_week);

    // let key = now.format("%H%M").to_string();
    // let data = prepare_json_occupancy(&key ,occupancy);

    (occupancy_location, schedule_location) 
}


