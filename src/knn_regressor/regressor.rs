use std::{f64, usize};

use super::data::{Data, DataPoint};

pub struct Regressor {
    data: Data,
    k: usize,
}

impl Regressor {
    pub fn new(data: Data, k: usize) -> Self {
        Self { data, k }
    }

    pub fn predict_range(
        &self,
        start: u16,
        end: u16,
        frequency: u16,
        week_day: usize,
    ) -> Vec<DataPoint> {
        let item_count = (end - start) / frequency;
        let mut result: Vec<DataPoint> = Vec::with_capacity(item_count as usize);

        for i in 0..item_count + 1 {
            let time = start + i * frequency;
            // Minutes cannot be greater than 59
            if time % 100 >= 60 {
                continue;
            }
            result.push(DataPoint::new(time, self.predict_one(week_day, time)));
        }

        result
    }

    pub fn predict_one(&self, weekday: usize, time: u16) -> u16 {
        // u16 limit is 65536
        let mut k_nearests: Vec<DataPoint> = Vec::with_capacity(self.k);
        let mut k_weights: Vec<f64> = Vec::with_capacity(self.k);

        let week_data = self.data.get_data();
        let days = &week_data[weekday];
        for (weeks_away, data_points) in days.iter().enumerate() {
            for data_point in data_points {
                let distance = data_point.get_time().abs_diff(time) as f64;
                let weight = 1.0 / (weeks_away as f64 + 1.0) + 1.0 / (distance + 1.0);

                if k_nearests.len() <= self.k {
                    k_nearests.push(*data_point);
                    k_weights.push(weight);
                    continue;
                }

                let mut max_index = 0;
                let mut swap = false;
                for (index, k_weight) in k_weights.iter().enumerate() {
                    if k_weight < &weight {
                        swap = true;
                        max_index = index;
                    }
                }

                if swap {
                    k_nearests[max_index] = *data_point;
                    k_weights[max_index] = weight;
                }
            }
        }

        // Weighted Average of K Nearest
        let total_weights: f64 = k_weights.iter().sum();
        let mut total: u16 = 0;

        for (neighbour, weight) in k_nearests.iter().zip(k_weights.iter()) {
            total += ((weight / total_weights) * (neighbour.get_occupancy() as f64)) as u16;
        }
        total
    }
}
