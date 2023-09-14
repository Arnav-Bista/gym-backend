use std::str::FromStr;

use chrono::{Duration, NaiveDate};
use serde::{Serialize,Deserialize};
use serde_json::{self, Value};
use tokio::fs;

use crate::{firebase::firebase::Firebase, core_functions::{uk_datetime_now, get_start_of_week, error_logger::error_logger}};

#[derive(Debug, Serialize, Deserialize)]
pub struct Data {
    data: Vec<Vec<(u16, u16)>>,
    for_date: String,
}

impl Data {
    pub async fn from_file(path: &str) -> Option<Self> {
        let data = fs::read_to_string(path).await.ok()?;
        Some(serde_json::from_str(&data).ok()?)
    }

    pub async fn write_to_file(&self, path: &str) {
        let _ = fs::write(path, serde_json::to_string(&self).unwrap()).await;
    }

    pub async fn new(firebase: &Firebase, k: usize, date: String) -> Self {
        let mut data: Vec<Vec<(u16,u16)>> = Vec::with_capacity(7);
        let now = uk_datetime_now::now().date_naive();
        for week in 1..k + 1 {
            let week_date: NaiveDate = now - Duration::days(7 * week as i64);
            let week_date = get_start_of_week::get(week_date);
            let key = week_date.to_string();
            let fetch = firebase.get(format!("rs_data/data/{}",key)).await.unwrap();
            let json_data: Value = serde_json::from_str(&fetch).unwrap();
            if json_data.is_array() {
                Self::handle_array(&mut data, json_data);
            }
            else if json_data.is_object() {
                Self::handle_object(&mut data, json_data);
            }
            else {
                error_logger("Unexpected Error").await;
                std::process::exit(1);
            }
        };
        Self {
            data,
            for_date: get_start_of_week::get(now).to_string(),
        }
    }
    
    fn handle_array(data: &mut Vec<Vec<(u16,u16)>>, json_data: Value) {
        for i in 0..7 {
            let new_data = match json_data.get(i) {
                Some(day_data) => Self::get_vec_from_day(day_data),
                None => Vec::new() 
            };
            if data.get(i).is_none() {
                data.push(new_data);
            }
            else {
                data[i].extend(new_data);
            }
        }
    }

    fn handle_object(data: &mut Vec<Vec<(u16,u16)>>, json_data: Value) {
        // May be missing index
        for i in 0..7 {
            let new_data = match json_data.get(i.to_string()) {
                Some(day_data) => Self::get_vec_from_day(day_data),
                None => Vec::new() 
            };
            if data.get(i).is_none() {
                data.push(new_data);
            }
            else {
                data[i].extend(new_data);
            }
        }
    }

    fn get_vec_from_day(day_data: &Value) -> Vec<(u16,u16)> {
        let mut data: Vec<(u16,u16)> = Vec::new();
        for (key,val) in day_data.as_object().unwrap() {
            data.push((
                key.parse().unwrap(),
                val.as_u64().unwrap() as u16
            ));
        }
        data
    }
    pub fn get_data(&self) -> &Vec<Vec<(u16,u16)>>{
        &self.data
    }

    pub fn get_for_date(&self) -> &String {
        &self.for_date
    }
}
