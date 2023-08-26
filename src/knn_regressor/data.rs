use serde::{Serialize,Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct Data {
    data: Vec<Vec<(u16, u16)>>
}
