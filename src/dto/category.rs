use serde::Deserialize;
use utoipa::ToSchema;
// 创建分类的请求体
#[derive(Deserialize, ToSchema)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
}
