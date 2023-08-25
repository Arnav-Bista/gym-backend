use chrono::NaiveTime;
use serde::Serialize;


#[derive(PartialEq, Debug, Serialize)]
pub struct Timing {
    #[serde(with = "naive_time_serialize")]
    opening: Option<NaiveTime>,
    #[serde(with = "naive_time_serialize")]
    closing: Option<NaiveTime>,
    open: bool
}

impl Timing {
    pub fn closed() -> Self {
        Self {
            opening: None,
            closing: None,
            open: false
        }
    }

    pub fn open(opening: NaiveTime, closing: NaiveTime) -> Self {
        Self {
            opening: Some(opening),
            closing: Some(closing),
            open: true
        }
    }

    pub fn get_opening(&self) -> Option<NaiveTime> {
        self.opening
    }

    pub fn get_closing(&self) -> Option<NaiveTime> {
        self.closing
    }

    pub fn is_open(&self) -> bool {
        self.open
    }
}


mod naive_time_serialize {
    use chrono::NaiveTime;
    use serde::{self, Serializer};
    pub fn serialize<S>(time: &Option<NaiveTime>, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {

        let s = match time {
            Some(time) => time.format("%H:%M:%S").to_string(),
            None => "null".to_string()
        };
        serializer.serialize_str(&s)
    }
}
