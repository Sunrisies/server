use crate::{ApiResponse, HttpResult, config::AppError};

use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
struct VersionInfo {
    name: String,
    version: String,
    #[serde(rename = "previousVersion")]
    previous_version: String,
    build: String,
    #[serde(rename = "buildDate")]
    build_date: String,
    #[serde(rename = "buildTimestamp")]
    build_timestamp: String,
    #[serde(rename = "gitHash")]
    git_hash: String,
    #[serde(rename = "gitBranch")]
    git_branch: String,
    environment: String,
    language: String,
}
pub async fn get_version() -> HttpResult {
    // 读取文件内容
    let version_content = fs::read_to_string(".docker/version.json").map_err(|e| {
        log::error!("Failed to read version.json: {}", e);
        AppError::InternalServerError("读取版本文件失败".to_string())
    })?;
    log::info!("{}", version_content);
    // 解析JSON到结构体
    let version: VersionInfo = serde_json::from_str(&version_content).map_err(|e| {
        log::error!("Failed to parse version.json: {}", e);
        AppError::InternalServerError("解析版本文件失败".to_string())
    })?;

    log::info!(
        "Successfully read version info: {} {}",
        version.name,
        version.version
    );
    Ok(ApiResponse::success(version, "获取版本信息成功").to_http_response())
}
