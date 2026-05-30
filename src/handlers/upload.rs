//! 上传处理器 — 统一文件上传管理

use actix_multipart::Multipart;
use actix_web::web;
use futures_util::StreamExt;
use sea_orm::DatabaseConnection;

use crate::config::AppError;
use crate::dto::common::{PaginatedResp, Pagination};
use crate::services::qiniu;
use crate::services::uploads::UploadsService;
use crate::{ApiResponse, EmptyResponse, HttpResult};

/// 上传文件（保存到七牛 + 记录到 DB）
#[utoipa::path(
    post,
    path = "/api/v1/upload",
    tag = "文件上传",
    summary = "上传文件",
    description = "上传文件到七牛云并记录到数据库，支持统一管理",
    request_body(content = String, description = "文件数据（multipart/form-data），字段名 file"),
    responses(
        (status = 200, description = "上传成功", body = crate::ApiResponse<crate::services::qiniu::UploadResult>),
        (status = 400, description = "文件过大或参数错误"),
        (status = 500, description = "上传失败"),
    ),
)]
pub async fn upload_file_handler(
    db: web::Data<DatabaseConnection>,
    mut payload: Multipart,
) -> HttpResult {
    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|e| AppError::MultipartError(e.to_string()))?;
        let field_name = field
            .content_disposition()
            .and_then(|cd| cd.get_name().map(|s| s.to_string()))
            .unwrap_or_default();
        if field_name == "file" {
            let result = qiniu::upload_file(&mut field, "uploads", true).await?;
            // 记录到 DB
            UploadsService::save(db.as_ref(), &result).await?;
            return Ok(ApiResponse::success(result, "文件上传成功").to_http_response());
        }
    }
    Err(AppError::BadRequest(
        "未找到文件字段 (field name: file)".to_string(),
    ))
}

/// 获取上传文件列表
#[utoipa::path(
    get,
    path = "/api/v1/uploads",
    tag = "文件上传",
    summary = "获取上传文件列表",
    description = "分页获取所有上传文件记录",
    params(
        ("page" = Option<u64>, Query, description = "页码，默认 1", example = 1),
        ("limit" = Option<u64>, Query, description = "每页数量，默认 20", example = 20),
    ),
    responses(
        (status = 200, description = "获取成功", body = crate::ApiResponse<PaginatedResp<crate::models::uploads::Model>>),
    ),
)]
pub async fn list_uploads_handler(
    db: web::Data<DatabaseConnection>,
    query: web::Query<PaginatedQuery>,
) -> HttpResult {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20);
    let (data, total) = UploadsService::list(db.as_ref(), page, limit).await?;
    Ok(ApiResponse::success(
        PaginatedResp {
            data,
            pagination: Pagination { total, page, limit },
        },
        "获取成功",
    )
    .to_http_response())
}

/// 删除上传文件
#[utoipa::path(
    delete,
    path = "/api/v1/uploads/{uuid}",
    tag = "文件上传",
    summary = "删除上传文件",
    description = "删除上传记录并同步删除七牛云上的文件",
    params(
        ("uuid" = String, Path, description = "上传记录 UUID"),
    ),
    responses(
        (status = 200, description = "删除成功", body = crate::ApiResponse<crate::EmptyResponse>),
        (status = 404, description = "记录不存在"),
    ),
)]
pub async fn delete_upload_handler(
    db: web::Data<DatabaseConnection>,
    path: web::Path<String>,
) -> HttpResult {
    UploadsService::delete(db.as_ref(), &path).await?;
    Ok(ApiResponse::<EmptyResponse>::success(EmptyResponse, "删除成功").to_http_response())
}

// Helper — 分页查询参数
#[derive(serde::Deserialize)]
pub struct PaginatedQuery {
    pub page: Option<u64>,
    pub limit: Option<u64>,
}
