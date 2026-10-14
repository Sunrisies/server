use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::models::tags;

/// 创建标签的请求体
#[derive(Deserialize, ToSchema, Debug, Validate)]
pub struct CreateTagRequest {
    #[validate(length(min = 1, max = 100, message = "标签名称需为1到100个字符"))]
    #[schema(example = "Rust")]
    pub name: String,
}

impl From<CreateTagRequest> for tags::ActiveModel {
    fn from(req: CreateTagRequest) -> Self {
        Self {
            name: sea_orm::Set(req.name),
            created_at: sea_orm::Set(chrono::Utc::now()),
            ..Default::default()
        }
    }
}

#[derive(Debug, Serialize, ToSchema, FromQueryResult)]
pub struct TagCloudItem {
    pub id: i32,
    pub name: String,
    pub count: i64,
}
