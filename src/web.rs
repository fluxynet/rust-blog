use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub mod openapi;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Listing<T> {
    pub items: Vec<T>,
    pub pages: i64,
}
