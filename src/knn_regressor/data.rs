use chrono::{Duration, NaiveDate};
use serde::{Deserialize, Serialize};
use serde_json::{self, Value};
use tokio::fs;

use crate::{
    core_functions::{error_logger::error_logger, get_start_of_week},
    firebase::firebase::Firebase,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Data {
    /// One 2D Vec for each weekday.
    /// First level vec is for how old the data is
    /// The closer the number to 0, the fresher the data.
    /// The rest is just Data Points
    data: [Vec<Vec<DataPoint>>; 7],
    for_date: String,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct DataPoint {
    time: u16,
    occupancy: u16,
}

impl DataPoint {
    pub fn new(time: u16, occupancy: u16) -> Self {
        Self { time, occupancy }
    }

    pub fn get_time(&self) -> u16 {
        return self.time;
    }

    pub fn get_time_mut(&mut self) -> u16 {
        return self.time;
    }

    pub fn get_occupancy(&self) -> u16 {
        return self.occupancy;
    }

    pub fn get_occupancy_mut(&mut self) -> u16 {
        return self.occupancy;
    }
}

impl Data {
    pub async fn from_file(path: &str) -> Option<Self> {
        let data = fs::read_to_string(path).await.ok()?;
        Some(serde_json::from_str(&data).ok()?)
    }

    pub async fn write_to_file(&self, path: &str) {
        let _ = fs::write(path, serde_json::to_string(&self).unwrap()).await;
    }

    pub async fn new(firebase: &Firebase, k: usize, date: NaiveDate) -> Self {
        let mut data = std::array::from_fn(|_| Vec::new());
        for week in 1..k + 1 {
            // Get the week start date as keys
            let week_date: NaiveDate = date - Duration::days(7 * week as i64);
            let week_date = get_start_of_week::get(week_date);
            let key = week_date.to_string();

            let fetch = firebase.get(format!("rs_data/data/{}", key)).await.unwrap();
            let json_data: Value = serde_json::from_str(&fetch).unwrap();

            if json_data.is_array() {
                Self::handle_array(&mut data, json_data, k);
            } else if json_data.is_object() {
                Self::handle_object(&mut data, json_data);
            } else {
                error_logger("Unexpected Error - Unexpected type of response from Firebase").await;
                std::process::exit(1);
            }
        }
        Self {
            data,
            for_date: get_start_of_week::get(date).to_string(),
        }
    }

    /// JSON Objects with consecutive number keys are treated as arrays
    /// Hence, it is handled differently.
    fn handle_array(data: &mut [Vec<Vec<DataPoint>>; 7], json_data: Value, k: usize) {
        for i in 0..7 {
            let new_data = match json_data.get(i) {
                Some(day_data) => Self::get_vec_from_day(day_data),
                None => Vec::new(),
            };
            data[i].push(new_data);
        }
    }

    /// When there are gaps in the indexing, it is treated as an Object instead.
    fn handle_object(data: &mut [Vec<Vec<DataPoint>>], json_data: Value) {
        // May be missing index
        for i in 0..7 {
            let new_data = match json_data.get(i.to_string()) {
                Some(day_data) => Self::get_vec_from_day(day_data),
                None => Vec::new(),
            };
            data[i].push(new_data);
        }
    }

    fn get_vec_from_day(day_data: &Value) -> Vec<DataPoint> {
        let mut data: Vec<DataPoint> = Vec::new();
        if day_data.is_null() {
            println!("Unexpected Error - day_data is null - get_vec_from_day");
            return Vec::new();
        }
        for (key, val) in day_data.as_object().unwrap() {
            data.push(DataPoint {
                time: key.parse().unwrap(),
                occupancy: val.as_u64().unwrap() as u16,
            });
        }
        data
    }

    pub fn get_data(&self) -> &[Vec<Vec<DataPoint>>; 7] {
        &self.data
    }

    pub fn get_data_mut(&self) -> &[Vec<Vec<DataPoint>>; 7] {
        &self.data
    }

    pub fn get_for_date(&self) -> &String {
        &self.for_date
    }
}
