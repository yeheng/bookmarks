use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
    pub has_more: bool,
}

#[allow(dead_code)]
impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, total: i64, limit: i64, offset: i64) -> Self {
        let has_more = offset + (data.len() as i64) < total;
        Self {
            data,
            total,
            limit,
            offset,
            has_more,
        }
    }
}
