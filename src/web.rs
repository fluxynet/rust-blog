use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Listing<T> {
    pub items: Vec<T>,
    pub pages: i64,
}
