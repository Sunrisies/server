use serde::Deserialize;
use utoipa::ToSchema;
// 创建分类的请求体
#[derive(Deserialize, ToSchema)]
pub struct CreateCategoryRequest {
    #[schema(example = "Tech")]
    pub name: String,
    #[schema(example = "tech-news")]
    pub slug: String,
    #[schema(example = "Tech news about programming, software development, and more.")]
    pub description: Option<String>,
}
