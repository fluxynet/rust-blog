use serde::{Deserialize, Serialize};

pub struct Pagination {
    pub offset: i64,
    pub limit: i64,
}

impl Pagination {
    pub fn from_page(page: i64, limit: i64) -> Self {
        let offset = (page.saturating_sub(1) * limit) as i64;
        Pagination { offset, limit }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Listing<T> {
    pub items: Vec<T>,
    pub count: i64,
}
