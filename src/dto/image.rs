use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageUploadRequest {
    pub file_name: String,
    pub file_size: u64,
    pub file_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageUploadResponse {
    pub id: i32,
    pub url: String,
    pub key: String,
    pub filename: String,
    pub size: u64,
    pub human_readable_size: String,
    pub created_at: String,
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
