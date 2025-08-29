use serde::{Deserialize, Serialize};

// 查询参数
#[derive(Deserialize, Debug)]
pub struct PaginationQuery {
    pub page: Option<u64>,
    pub limit: Option<u64>,
}

// 统一分页响应
#[derive(Serialize)]
pub struct PaginatedResp<T: Serialize> {
    pub data: Vec<T>,
    pub page: u64,
    pub limit: u64,
    pub total: u64,
}
