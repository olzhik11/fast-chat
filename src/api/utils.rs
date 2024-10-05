use serde::{Deserialize, Serialize};


static PAGE_LIMIT: i16 = 20;
static PAGE_OFFSET: i16 = 0;

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

#[derive(Debug, Deserialize, Serialize)]
pub struct SearchParams {
    pub query: Option<String>,
    pub limit: Option<i16>,
    pub offset: Option<i16>,
}

impl Default for SearchParams {
    fn default() -> Self {
        SearchParams {
            query: None,
            limit: Some(PAGE_LIMIT),
            offset: Some(PAGE_OFFSET),
        }
    }
}