use crate::models::categories;
use sea_orm::ActiveValue::Set;
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

impl From<CreateCategoryRequest> for categories::ActiveModel {
    fn from(request: CreateCategoryRequest) -> Self {
        log::info!(
            "Converting CreateCategoryRequest to Category ActiveModel: {:?}",
            request
        );
        categories::ActiveModel {
            name: Set(request.name),
            slug: Set(request.slug),
            description: Set(request.description),
            ..Default::default()
        }
    }
}
