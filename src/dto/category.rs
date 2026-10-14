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
    #[serde(default = "default_slug")]
    pub slug: String,
    #[validate(length(max = 255, message = "描述长度不能超过255个字符"))]
    #[schema(example = "Tech news about programming, software development, and more.")]
    pub description: Option<String>,
}

fn default_slug() -> String {
    uuid::Uuid::new_v4().to_string()
}
impl From<CreateCategoryRequest> for categories::ActiveModel {
    fn from(req: CreateCategoryRequest) -> Self {
        Self {
            name: sea_orm::Set(req.name),
            slug: sea_orm::Set(req.slug),
            description: sea_orm::Set(req.description),
            created_at: sea_orm::Set(chrono::Utc::now()),
            updated_at: sea_orm::Set(chrono::Utc::now()),
            ..Default::default()
        }
    }
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct UpdateCategoryRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
}
