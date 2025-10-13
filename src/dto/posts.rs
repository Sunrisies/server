// src/models/responses.rs
use crate::utils::fmt_beijing;
use serde::Serialize;
use utoipa::ToSchema;

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
