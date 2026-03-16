use sea_orm::FromQueryResult;
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

// 使用宏重写你的代码
impl_from_request!(CreateTagRequest => tags::ActiveModel {
    name
});

#[derive(Debug, Serialize, FromQueryResult)]
pub struct TagCloudItem {
    pub id: i32,
    pub name: String,
    pub count: i64,
}
