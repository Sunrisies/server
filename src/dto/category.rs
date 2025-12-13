use crate::models::categories;
use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;
/// 创建分类的请求体
#[derive(Deserialize, ToSchema, Debug, Validate)]
pub struct CreateCategoryRequest {
    #[validate(length(min = 1, message = "名称不能为空"))]
    #[schema(example = "Tech")]
    pub name: String,
    #[validate(length(min = 1, message = "slug不能为空"))]
    #[schema(example = "tech-news")]
    pub slug: String,
    #[validate(length(max = 255, message = "描述长度不能超过255个字符"))]
    #[schema(example = "Tech news about programming, software development, and more.")]
    pub description: Option<String>,
}

impl_from_request!(CreateCategoryRequest => categories::ActiveModel {
    name,
    slug,
    description,
});
