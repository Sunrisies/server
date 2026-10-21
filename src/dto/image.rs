use crate::utils::fmt_beijing;
use sea_orm::prelude::DateTimeUtc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageUploadRequest {
    pub file_name: String,
    pub file_size: u64,
    pub file_type: String,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ImageUploadResponse {
    pub id: i32,
    pub url: String,
    pub key: String,
    pub filename: String,
    pub size: i64,
    pub human_readable_size: String,
    #[schema(value_type = String, format = DateTime)]
    #[serde(serialize_with = "fmt_beijing")]
    pub created_at: DateTimeUtc,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageListResponse {
    pub images: Vec<ImageInfo>,
    pub total: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageInfo {
    pub id: i32,
    pub url: String,
    pub key: String,
    pub filename: String,
    pub size: u64,
    pub human_readable_size: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize, validator::Validate)]
pub struct ImageQuery {
    #[serde(default = "crate::dto::common::default_page")]
    #[validate(range(min = 1))]
    pub page: u64,
    #[serde(default = "crate::dto::common::default_limit")]
    #[validate(range(min = 1, max = 100))]
    pub limit: u64,
    #[validate(length(max = 100))]
    pub search: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ImageReference {
    pub uuid: String,
    pub title: String,
    pub status: i16,
    pub body: bool,
    pub cover: bool,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ManagedImage {
    #[serde(flatten)]
    pub image: crate::models::images::Model,
    pub reference_count: usize,
    pub references: Vec<ImageReference>,
}
