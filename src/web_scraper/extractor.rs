use std::fs;

use regex::Regex;

use reqwest::{Client, RequestBuilder, Method};

use crate::core_functions::error_logger::{self, error_logger};

use super::schedule::Schedule;

pub struct Extractor {
    client: Client,
    url: String,
    user_agent: String,
    occupancy_regex: Regex,
    schedule_regex: Regex,
    scrape_result: Option<String>
    // request: RequestBuilder
}

impl Extractor {
    pub fn new_default() -> Self {
        let config_data = Self::parse_config(&"config.cfg".to_string());
        Self {
            client: Client::new(),
            url: config_data[0].to_string(),
            user_agent: config_data[1].to_string(),
            occupancy_regex: Regex::new(r"Occupancy:\s+(\d+)%").unwrap(),
            schedule_regex: Regex::new("<dd class=\"paired-values-list__value\">(.*?)</dd>").unwrap(),
            scrape_result: None,
        }
    }

    pub fn new(config_path: String) -> Self {
        let config_data = Self::parse_config(&config_path);
        Self {
            client: Client::new(),
            url: config_data[0].to_string(),
            user_agent: config_data[1].to_string(),
            occupancy_regex: Regex::new(r"Occupancy:\s+(\d+)%").unwrap(),
            schedule_regex: Regex::new("<dd class=\"paired-values-list__value\">(.*?)</dd>").unwrap(),
            scrape_result: None
        }
    }

    fn parse_config(src: &String) -> Vec<String> {
        let mut config_data: Vec<String> = Vec::with_capacity(2);
        let data = match fs::read_to_string(src) {
            Ok(data) => data,
            Err(_) => {
                println!("Error opening file: {}", src);
                std::process::exit(1);
            }
        };
        for line in data.lines() {
            config_data.push(line.to_string());
        }
        config_data
    }

    fn get_request(&self) -> RequestBuilder {
        self.client.request(Method::GET, &self.url)
        .header("User-Agent", &self.user_agent)
    }

    
    pub async fn scrape(&mut self) -> Result<(),()> {
        let response = match self.get_request().send().await {
            Ok(data) => data,
            Err(_) => {
                error_logger("Network Error").await;
                self.scrape_result = None;
                return Err(());
            }
        };
        let text = match response.text().await {
            Ok(text) => text,
            Err(_) => {
                error_logger("Failed to get response text.").await;
                self.scrape_result = None;
                return Err(());
            }
        };
        self.scrape_result = Some(text);
        Ok(())
    }

    pub async fn scrape_occupancy(&self) -> Option<u8> {
        let text = &self.scrape_result.clone()?;
        let regex_match = match self.occupancy_regex.captures(&text) {
            Some(data) => data,
            None => {
                error_logger("Occupancy Scrape Error. Regex Fail").await;
                return None;
            }
        };
        let result: &str = regex_match.get(1).map_or("0", |m| m.as_str());
        let result: u8 = match result.parse() {
            Ok(num) => num,
            Err(_) => {
                error_logger("Occupancy Scrape Error. Parse to u8 fail.").await;
                return None;
            }
        };
        Some(result)
    }

    pub async fn scrape_schedule(&self) -> Option<Schedule> {
        let text = &self.scrape_result.clone()?;
        let schedules = self.schedule_regex.captures_iter(text);
        let schedule = match Schedule::form_schedule_timings(schedules) {
            Some(data) => data,
            None => {
                error_logger("Scrape Schedule Error").await;
                return None;
            }
        };
        Some(schedule)
    }

}
