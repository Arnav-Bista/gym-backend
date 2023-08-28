use crate::core_functions::weekday_matcher;

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

        for i in 0..item_count {
            let time = start + i * frequency;
            result.push((time, self.predict_one(week_day, time)));
        }

        result
    }

    pub fn predict_one(&self, week_day: usize, time: u16) -> u16 {
        let mut k_nearest: Vec<(u16,u16)> = vec![(u16::MAX,u16::MAX); self.k];
        for day_data in &self.data.get_data()[week_day] {
            let distance = day_data.0.abs_diff(time);
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
            }
        }

        // Average Occupancy of k nearest
        let mut total: u64 = 0;
        for ele in k_nearest {
            total += ele.1 as u64;
        }

        (total / self.k as u64) as u16
    }

}
