use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SearchPaginatedResponse<Model> {
    pub data: Vec<Model>,
    pub total_count: i64,
}

impl<Model> SearchPaginatedResponse<Model> {
    pub fn new(data: Vec<Model>, total_count: i64) -> Self {
        Self { data, total_count }
    }
}
