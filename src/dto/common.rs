use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;
#[derive(Serialize, ToSchema)]
pub struct Pagination {
    pub page: u64,
    pub limit: u64,
    pub total: u64,
}
/// 统一分页响应
#[derive(Serialize, ToSchema)]
pub struct PaginatedResp<T: Serialize> {
    pub data: Vec<T>,
    pub pagination: Pagination,
}

#[derive(Validate, Debug, Serialize, Deserialize, IntoParams)]
#[into_params(style = Form, parameter_in = Query)]
pub struct PaginationQuery {
    /// 页码
    #[validate(range(min = 1, message = "页码必须大于1"))]
    #[serde(default = "default_page")]
    #[param(example = json!(1))]
    pub page: u64,
    /// 每页数量
    #[validate(range(max = 10, message = "每页数量不能超过100"))]
    #[serde(default = "default_limit")]
    #[param(example = json!(10))]
    pub limit: u64,
}
fn default_page() -> u64 {
    1
}
fn default_limit() -> u64 {
    10
}
