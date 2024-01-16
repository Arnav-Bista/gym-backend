mod core_functions;
mod web_scraper;
mod firebase;
mod sleeper;
mod knn_regressor;

use std::{fs, collections::HashMap};

use chrono::{Datelike, DateTime, NaiveDate, Duration, Weekday};
use chrono_tz::Tz;
use core_functions::{uk_datetime_now, get_start_of_week, weekday_matcher, error_logger::error_logger};
use firebase::firebase::Firebase;
use knn_regressor::{data::Data, regressor::{Regressor, self}};

use serde_json::json;
use sleeper::Sleeper;
use web_scraper::{extractor, schedule::Schedule};

use tokio::{self, join};


#[tokio::main]
async fn main() {
    let mut extractor = extractor::Extractor::new_default();
    let db_url: String = fs::read_to_string("databaseUrl.secret").unwrap();
    let mut firebase = Firebase::new("serviceAccountKey.json.secret", db_url);
    let mut sleeper = Sleeper::new(5 * 60, 5 * 60,None);

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

        if !sleeper.is_standard_interval().expect("Unexpected Error - Unwrap on Sleeper Schedule") {
            println!("Too early");
            sleeper.sleep().await;
            continue;
        }

        let uk_now = uk_datetime_now::now();
        let key = uk_now.format("%H%M").to_string();
        let occupancy_data = prepare_occupancy_json(&key, occupancy);
        let latest_occupancy_location = "rs_data/data/latest/data";
        let latest_schedule_location = "rs_data/data/latest/schedule";

        if firebase.handle_auth_token().await.is_err() {
            error_logger("Firebase Error - Auth Token").await;
            sleeper.async_sleep_error().await;
            continue;
        }

        let latest_occupancy_data = prepare_occupancy_json(&uk_now.format("%Y-%m-%d-%H-%M").to_string(), occupancy);

        let (occupancy_location, schedule_location) = prepare_location(uk_now);
        let data_insert = firebase.update(occupancy_location, &occupancy_data);
        let schedule_insert = firebase.set(schedule_location, &schedule_data);
        let latest_occupancy_set = firebase.set(latest_occupancy_location.to_string(), &latest_occupancy_data);
        let latest_schedule_set = firebase.set(latest_schedule_location.to_string(), &schedule_data);

        // Make these concurrent. join! does not do them in parallel! 
        join!(
            sleeper.sleep(),
            make_predictions(&firebase, sleeper.get_schedule(), sleeper.get_frequency() / 60),
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

    (occupancy_location, schedule_location) 
}

async fn make_predictions(firebase: &Firebase, schedule: &Schedule, frequency: u64) {
    // SQRT((15 * 60) / 5) -> Sqrt of number of data points
    // 8.9
    let k = 9;
    let path = "knn_regressor.data";

    let now = uk_datetime_now::now();
    let now_date: NaiveDate = now.date_naive();
    let mut new = false;
    
    if now.weekday() == Weekday::Sun {
        predict_monday(firebase, k, schedule, frequency, now_date).await;
    }

    let data = match Data::from_file(path).await {
        Some(data) => {
            if data.get_for_date() != &get_start_of_week::get(now_date).to_string() {
                // New Week
                new = true;
                Data::new(firebase, k, now_date).await
            }
            else {
                data
            }
        }
        None => {
            new = true;
            Data::new(firebase, k, now_date).await
        }
    };

    if !new {
        return;
    }

    data.write_to_file(path).await;


    let regressor = Regressor::new(data, k);

    // Predict for the entire week
    for i in 0..7 {
        let timings = schedule.get_timings_from_weekday(weekday_matcher::get_weekday(i));
        let start: u16 = timings.get_opening().unwrap().format("%H%M").to_string().parse().unwrap();
        let end: u16 = timings.get_closing().unwrap().format("%H%M").to_string().parse().unwrap();

        let predictions = regressor.predict_range(start, end, frequency as u16, i);

        let location = format!("rs_data/prediction/{}/{}", get_start_of_week::get(now_date).to_string(), i);

        let mut map: HashMap<String,u16> = HashMap::new();

        for (key, value) in predictions {
            map.insert(key.to_string(), value);
        }

        let data = serde_json::to_string(&map).unwrap();
        firebase.set(location, &data).await;
    }
}


async fn predict_monday(firebase: &Firebase, k: usize, schedule: &Schedule, frequency: u64, date: NaiveDate) {
    // This is for that +1 Edge case.
    // This is indeed inefficient as it will be overwritten when monday hits.
    // But this is so much simpler than doing Today + Tomorrow prediction (due to edge cases)
    let date = date + Duration::days(7);
    let path = "knn_regressor_tomorrow.data";
    let mut new = false;
    let data = match Data::from_file(path).await {
        Some(data) => {
            if data.get_for_date() != &get_start_of_week::get(date).to_string() {
                // New Week
                new = true;
                Data::new(firebase, k, date).await
            }
            else {
                data
            }
        }
        None => {
            new = true;
            Data::new(firebase, k, date).await
        }
    };

    if !new {
        return;
    }

    data.write_to_file(path).await;

    let regressor = Regressor::new(data, k);

    let weekday = weekday_matcher::get_num(date.weekday());

    let timings = schedule.get_timings_from_weekday(date.weekday());
    let start: u16 = timings.get_opening().unwrap().format("%H%M").to_string().parse().unwrap();
    let end: u16 = timings.get_closing().unwrap().format("%H%M").to_string().parse().unwrap();

    let predictions = regressor.predict_range(start, end, frequency as u16, weekday);

    let location = format!("rs_data/prediction/{}/{}", get_start_of_week::get(date).to_string(), 0);

    let mut map: HashMap<String,u16> = HashMap::new();

    for (key, value) in predictions {
        map.insert(key.to_string(), value);
    }

    let data = serde_json::to_string(&map).unwrap();
    firebase.set(location, &data).await;



}

