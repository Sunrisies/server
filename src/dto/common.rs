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
    #[validate(range(min = 1, max = 100, message = "每页数量必须为1到100"))]
    #[serde(default = "default_limit")]
    #[param(example = json!(10))]
    pub limit: u64,
    /// 分类搜索
    pub category: Option<i32>,
    /// 标签搜索
    pub tag: Option<i32>,
}
fn default_page() -> u64 {
    1
}
fn default_limit() -> u64 {
    10
}

#[cfg(test)]
mod pagination_tests {
    use super::*;
    #[test]
    fn invalid_pages_and_limits_are_rejected() {
        for (page, limit) in [(0, 10), (1, 0), (1, 101)] {
            assert!(
                PaginationQuery {
                    page,
                    limit,
                    category: None,
                    tag: None
                }
                .validate()
                .is_err()
            );
        }
        assert!(
            PaginationQuery {
                page: 1,
                limit: 100,
                category: None,
                tag: None
            }
            .validate()
            .is_ok()
        );
    }
}
