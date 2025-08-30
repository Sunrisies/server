use serde::{Deserialize, Serialize};
use validator::Validate;

// 统一分页响应
#[derive(Serialize)]
pub struct PaginatedResp<T: Serialize> {
    pub data: Vec<T>,
    pub page: u64,
    pub limit: u64,
    pub total: u64,
}

#[derive(Validate, Debug, Clone, Serialize, Deserialize)]
pub struct PaginationQuery {
    #[validate(range(min = 1, message = "页码必须大于1"))]
    pub page: Option<u64>,
    // 每页数量不能超过100
    #[validate(range(max = 10, message = "每页数量不能超过100"))]
    pub limit: Option<u64>,
}
