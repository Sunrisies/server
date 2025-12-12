use sea_orm::{ActiveValue::Set, FromQueryResult};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::models::tags;

/// 创建标签的请求体
#[derive(Deserialize, ToSchema, Debug, Validate)]
pub struct CreateTagRequest {
    #[validate(length(min = 1, message = "名称不能为空"))]
    #[schema(example = "Rust")]
    pub name: String,
}

impl From<CreateTagRequest> for tags::ActiveModel {
    fn from(request: CreateTagRequest) -> Self {
        log::info!(
            "Converting CreateTagRequest to Tags ActiveModel: {:?}",
            request
        );
        tags::ActiveModel {
            name: Set(request.name),
            ..Default::default()
        }
    }
}

#[derive(Debug, Serialize, FromQueryResult)]
pub struct TagCloudItem {
    pub id: i32,
    pub name: String,
    pub count: i64,
}
