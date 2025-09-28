use sea_orm::ActiveValue::Set;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::models::categories;
// 创建分类的请求体
#[derive(Deserialize, ToSchema, Debug)]
pub struct CreateCategoryRequest {
    #[schema(example = "Tech")]
    pub name: String,
    #[schema(example = "tech-news")]
    pub slug: String,
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
