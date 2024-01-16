use std::{f64, iter::Sum};

use super::data::Data;

pub struct Regressor {
    data: Data,
    k: usize
}

impl Regressor {
    pub fn new(data: Data, k: usize) -> Self {
        Self {
            data,
            k
        }
    }

    pub fn predict_range(&self, start: u16, end: u16, frequency: u16, week_day: usize) -> Vec<(u16,u16)> {
        let item_count = (end - start) / frequency;
        let mut result: Vec<(u16,u16)> = Vec::with_capacity(item_count as usize);

        for i in 0..item_count + 1 {
            let time = start + i * frequency;
            // Minutes cannot be greater than 59 minutes
            if time % 100 >= 60 {
                continue;
            }
            result.push((time, self.predict_one(week_day, time)));
        }

        result
    }
     

    pub fn predict_one(&self, weekday: usize, time: u16) -> u16 {
        // u16 limit is 65536
        let mut k_nearest: Vec<(u16,u16)> = Vec::with_capacity(self.k);
        let mut k_weights: Vec<f64> = Vec::with_capacity(self.k);

        for (weeks_away, day_data) in self.data.get_data()[weekday].iter().enumerate() {
            let distance = day_data.0.abs_diff(time);
            // Weights take into consideration the Week Date (how 'fresh' the data is)
            // And the distance away.
            // This will probably work best when the K used here greater than the number of weeks
            // of data available.
            let weight: f64 = 1.0 / ((weeks_away + 1) as f64) + 1.0 / distance as f64;

            // Fill before starting to swap
            if k_nearest.len() <= self.k {
                k_nearest.push(*day_data);
                k_weights.push(weight);
                continue;
            }
            // We don't have Y for normal metrics.
            let mut max_index = 0;
            let mut swap = false;
            for (index, nearest_data) in k_nearest.iter().enumerate() {
                if distance < nearest_data.0.abs_diff(time) {
                    swap = true;
                    max_index = index;
                }
            }
            if swap {
                k_nearest[max_index] = *day_data;
                k_weights[max_index] = weight;
            }
        }
        
        // Weighted Average of K Nearest
        let total_weights: f64 = k_weights.iter().sum();
        let mut total: u16 = 0;
        for (neighbour, weight) in k_nearest.iter().zip(k_weights.iter()) {
            total += ((weight / total_weights) * neighbour.1 as f64) as u16;
        }

        total 
    }

}
