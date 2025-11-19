// src/models/responses.rs
use crate::utils::fmt_beijing;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, ToSchema)]
pub struct CategoryResponse {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Serialize, Clone, ToSchema)]
pub struct TagResponse {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PostListResponse {
    pub id: i32,
    pub uuid: String,
    pub title: String,
    pub cover: String,
    pub author: String,
    pub content: String,
    #[schema(value_type = String, format = DateTime)]
    #[serde(serialize_with = "fmt_beijing")]
    pub publish_time: chrono::DateTime<chrono::Utc>,
    #[schema(value_type = String, format = DateTime)]
    #[serde(serialize_with = "fmt_beijing")]
    pub update_time: chrono::DateTime<chrono::Utc>,
    pub views: i32,
    pub is_top: bool,
    pub is_publish: bool,
    pub is_hide: bool,
    pub description: String,
    pub size: i32,
    pub category: Option<CategoryResponse>,
    pub tags: Vec<TagResponse>,
}

/// 不带content,以及markdowncontent的post
#[derive(Debug, Serialize, ToSchema)]
pub struct PostResponse {
    pub id: i32,
    pub uuid: String,
    pub title: String,
    pub cover: String,
    pub author: String,
    #[schema(value_type = String, format = DateTime)]
    #[serde(serialize_with = "fmt_beijing")]
    pub publish_time: chrono::DateTime<chrono::Utc>,
    #[schema(value_type = String, format = DateTime)]
    #[serde(serialize_with = "fmt_beijing")]
    pub update_time: chrono::DateTime<chrono::Utc>,
    pub views: i32,
    pub is_top: bool,
    pub is_publish: bool,
    pub is_hide: bool,
    pub description: String,
    pub size: i32,
    pub category: Option<CategoryResponse>,
    pub tags: Vec<TagResponse>,
}

/// 用于创建文章的请求体
#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct CreatePostRequest {
    #[validate(length(min = 1, max = 255, message = "标题长度必须在1-255个字符之间"))]
    pub title: String,

    #[validate(length(max = 500, message = "摘要长度不能超过500个字符"))]
    pub summary: Option<String>,

    #[validate(length(min = 1, message = "内容不能为空"))]
    pub content: String,

    #[validate(length(min = 1, message = "Markdown内容不能为空"))]
    pub markdowncontent: String,

    pub cover_image: Option<String>,

    pub category_id: i32,

    #[validate(length(min = 1, message = "至少需要选择一个标签"))]
    pub tag_ids: Vec<i32>,

    pub status: i16, // 0 草稿 1 发布 2 下线

    pub featured: bool,
}

/// 用于更新文章的请求体
#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct UpdatePostRequest {
    #[validate(length(min = 1, max = 255, message = "标题长度必须在1-255个字符之间"))]
    pub title: Option<String>,

    #[validate(length(max = 500, message = "摘要长度不能超过500个字符"))]
    pub summary: Option<String>,

    pub content: Option<String>,

    pub markdowncontent: Option<String>,

    pub cover_image: Option<String>,

    pub category_id: Option<i32>,

    pub tag_ids: Option<Vec<i32>>,

    pub status: Option<i16>, // 0 草稿 1 发布 2 下线

    pub featured: Option<bool>,
}
