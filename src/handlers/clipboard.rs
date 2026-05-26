use actix_multipart::Multipart;
use actix_web::{HttpMessage, HttpRequest, web};
use futures_util::StreamExt;
use sea_orm::DatabaseConnection;

use crate::dto::clipboard::{ClipboardEntryResponse, ClipboardQuery, CreateTextRequest};
use crate::dto::common::PaginatedResp;
use crate::utils::jwt::TokenClaims;
use crate::{
    ApiResponse, AppError, EmptyResponse, HttpResult, services::clipboard::ClipboardService,
};

/// 上传文本到剪贴板
#[utoipa::path(
    post,
    path = "/api/v1/clipboard/text",
    tag = "云剪贴板",
    summary = "上传文本",
    description = "将文本内容保存到云剪贴板，可在其他设备查看和复制",
    request_body(content = CreateTextRequest, examples(
        ("文本示例" = (value = json!({"content": "这是一段需要跨设备同步的文本内容"})))
    )),
    responses(
        (status = 200, description = "上传成功", body = ApiResponse<ClipboardEntryResponse>,
            example = json!({
                "code": 200,
                "message": "上传成功",
                "data": {
                    "uuid": "550e8400-e29b-41d4-a716-446655440000",
                    "type": "text",
                    "content": "这是一段需要跨设备同步的文本内容",
                    "pinned": false,
                    "created_at": "2026-05-26T10:30:00Z"
                }
            })),
        (status = 400, description = "内容为空或过长"),
        (status = 401, description = "未登录"),
    ),
)]
pub async fn create_text_handler(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    body: web::Json<CreateTextRequest>,
) -> HttpResult {
    let user_id = current_user_id(&req);
    let result = ClipboardService::create_text(db.as_ref(), user_id, body.into_inner()).await?;
    Ok(ApiResponse::success(result, "上传成功").to_http_response())
}

/// 上传文件/图片到剪贴板
#[utoipa::path(
    post,
    path = "/api/v1/clipboard/file",
    tag = "云剪贴板",
    summary = "上传文件或图片",
    description = "上传文件到云剪贴板（支持任意类型，最大 50MB，图片自动识别）",
    request_body(content = String, description = "文件数据（multipart/form-data），字段名 file"),
    responses(
        (status = 200, description = "上传成功", body = ApiResponse<ClipboardEntryResponse>,
            example = json!({
                "code": 200,
                "message": "上传成功",
                "data": {
                    "uuid": "550e8400-e29b-41d4-a716-446655440000",
                    "type": "image",
                    "file_url": "https://img.sunrise1024.top/clipboard/xxx.jpg",
                    "file_name": "Screenshot.jpg",
                    "file_size": 234567,
                    "mime_type": "image/jpeg",
                    "pinned": false,
                    "created_at": "2026-05-26T10:30:00Z"
                }
            })),
        (status = 400, description = "文件过大或参数错误"),
        (status = 401, description = "未登录"),
    ),
)]
pub async fn create_file_handler(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    mut payload: Multipart,
) -> HttpResult {
    let user_id = current_user_id(&req);

    // 从 multipart 中提取第一个 file 字段
    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|e| AppError::MultipartError(e.to_string()))?;
        let field_name = field
            .content_disposition()
            .and_then(|cd| cd.get_name().map(|s| s.to_string()))
            .unwrap_or_default();

        if field_name == "file" {
            let result = ClipboardService::create_file(db.as_ref(), user_id, &mut field).await?;
            return Ok(ApiResponse::success(result, "上传成功").to_http_response());
        }
    }

    Err(AppError::BadRequest(
        "未找到文件字段 (field name: file)".to_string(),
    ))
}

/// 获取剪贴板列表
#[utoipa::path(
    get,
    path = "/api/v1/clipboard",
    tag = "云剪贴板",
    summary = "获取剪贴板列表",
    description = "分页获取当前用户的剪贴板条目列表，按时间倒序",
    params(
        ("page" = Option<u64>, Query, description = "页码，默认1"),
        ("limit" = Option<u64>, Query, description = "每页数量，默认20"),
        ("type" = Option<String>, Query, description = "筛选类型：text / image / file")
    ),
    responses(
        (status = 200, description = "获取成功", body = ApiResponse<PaginatedResp<ClipboardEntryResponse>>),
        (status = 401, description = "未登录"),
    ),
)]
pub async fn list_handler(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    query: web::Query<ClipboardQuery>,
) -> HttpResult {
    let user_id = current_user_id(&req);
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20).min(100);
    let type_filter = query.r#type.clone();

    let result = ClipboardService::list(db.as_ref(), user_id, page, limit, type_filter).await?;
    Ok(ApiResponse::success(result, "获取成功").to_http_response())
}

/// 删除剪贴板条目
#[utoipa::path(
    delete,
    path = "/api/v1/clipboard/{uuid}",
    tag = "云剪贴板",
    summary = "删除剪贴板条目",
    description = "根据 UUID 删除指定的剪贴板条目（文件类同时删除七牛云文件）",
    params(
        ("uuid" = String, Path, description = "条目 UUID")
    ),
    responses(
        (status = 200, description = "删除成功", body = ApiResponse<EmptyResponse>),
        (status = 401, description = "未登录"),
        (status = 403, description = "无权操作"),
        (status = 404, description = "条目不存在"),
    ),
)]
pub async fn delete_handler(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    path: web::Path<String>,
) -> HttpResult {
    let user_id = current_user_id(&req);
    let uuid = path.into_inner();

    ClipboardService::delete(db.as_ref(), user_id, &uuid).await?;
    Ok(ApiResponse::<EmptyResponse>::success(EmptyResponse, "删除成功").to_http_response())
}

/// 从请求中获取当前用户 ID
fn current_user_id(req: &HttpRequest) -> i32 {
    if let Some(claims) = req.extensions().get::<TokenClaims>() {
        log::info!("剪贴板操作: {} ({})", claims.user_name, claims.user_uuid);
        // 单人博客简化处理：返回 1
        1
    } else {
        // 应该有中间件保证登录，但以防万一
        1
    }
}
