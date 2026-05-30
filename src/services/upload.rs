//! 文件上传 — 委托 qiniu 模块处理七牛上传

use actix_multipart::Multipart;
use futures_util::StreamExt;

use crate::config::AppError;
use crate::services::qiniu;
use crate::{ApiResponse, HttpResult};

/// 处理文件上传（通用文件/图片，上传到七牛云）
pub async fn handle_upload(mut payload: Multipart) -> HttpResult {
    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|e| AppError::MultipartError(e.to_string()))?;
        let field_name = field
            .content_disposition()
            .and_then(|cd| cd.get_name().map(|s| s.to_string()))
            .unwrap_or_default();

        if field_name == "file" {
            let result = qiniu::upload_file(&mut field, "uploads", true).await?;
            return Ok(ApiResponse::success(result, "文件上传成功").to_http_response());
        }
    }
    Err(AppError::BadRequest(
        "未找到文件字段 (field name: file)".to_string(),
    ))
}
