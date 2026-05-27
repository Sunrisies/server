use actix_multipart::Multipart;
use actix_web::{HttpMessage, HttpRequest, web};
use futures_util::StreamExt;
use sea_orm::DatabaseConnection;

use crate::dto::clipboard::{
    AuthChannelRequest, ClipboardEntryResponse, ClipboardQuery, CreateChannelRequest,
    CreateTextRequest,
};
use crate::dto::common::PaginatedResp;
use crate::services::clipboard::{ClipboardService, extract_channel_claims};
use crate::utils::jwt::TokenClaims;
use crate::{ApiResponse, AppError, EmptyResponse, HttpResult};

// ── 频道 ──

/// 创建频道
#[utoipa::path(
    post,
    path = "/api/v1/clipboard/channel",
    tag = "云剪贴板",
    summary = "创建频道",
    description = "创建一个新的剪贴板频道（需要管理员登录），返回频道 token",
    request_body(content = CreateChannelRequest, examples(
        ("创建频道" = (value = json!({"name": "我的频道", "password": "1234"})))
    )),
    responses(
        (status = 200, description = "创建成功", body = ApiResponse<crate::dto::clipboard::AuthChannelResponse>),
        (status = 400, description = "参数错误"),
        (status = 401, description = "未登录"),
        (status = 409, description = "频道名称已存在"),
    ),
)]
pub async fn create_channel_handler(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    body: web::Json<CreateChannelRequest>,
) -> HttpResult {
    // 需要管理员登录（检查 admin JWT）
    if req.extensions().get::<TokenClaims>().is_none() {
        return Err(AppError::Unauthorized("请先登录".to_string()));
    }
    let result = ClipboardService::create_channel(db.as_ref(), body.into_inner()).await?;
    Ok(ApiResponse::success(result, "频道创建成功").to_http_response())
}

/// 登录频道
#[utoipa::path(
    post,
    path = "/api/v1/clipboard/channel/auth",
    tag = "云剪贴板",
    summary = "登录频道",
    description = "输入频道名称和密码，获取频道 token",
    request_body(content = AuthChannelRequest, examples(
        ("登录频道" = (value = json!({"name": "我的频道", "password": "1234"})))
    )),
    responses(
        (status = 200, description = "登录成功", body = ApiResponse<crate::dto::clipboard::AuthChannelResponse>),
        (status = 401, description = "密码错误"),
        (status = 404, description = "频道不存在"),
    ),
)]
pub async fn auth_channel_handler(
    db: web::Data<DatabaseConnection>,
    body: web::Json<AuthChannelRequest>,
) -> HttpResult {
    let result = ClipboardService::auth_channel(db.as_ref(), body.into_inner()).await?;
    Ok(ApiResponse::success(result, "登录成功").to_http_response())
}

// ── 文本上传 ──

/// 上传文本到剪贴板
#[utoipa::path(
    post,
    path = "/api/v1/clipboard/text",
    tag = "云剪贴板",
    summary = "上传文本",
    description = "将文本内容保存到云剪贴板（需要频道 token）",
    request_body(content = CreateTextRequest, examples(
        ("文本示例" = (value = json!({"content": "这是一段需要跨设备同步的文本内容"})))
    )),
    responses(
        (status = 200, description = "上传成功", body = ApiResponse<ClipboardEntryResponse>),
        (status = 400, description = "内容为空或过长"),
        (status = 401, description = "需要频道 token"),
    ),
)]
pub async fn create_text_handler(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    body: web::Json<CreateTextRequest>,
) -> HttpResult {
    let claims = extract_channel_claims(&req)?;
    let result =
        ClipboardService::create_text(db.as_ref(), claims.channel_id, body.into_inner()).await?;
    Ok(ApiResponse::success(result, "上传成功").to_http_response())
}

/// 上传文件/图片到剪贴板
#[utoipa::path(
    post,
    path = "/api/v1/clipboard/file",
    tag = "云剪贴板",
    summary = "上传文件或图片",
    description = "上传文件到云剪贴板（需要频道 token，支持任意类型，最大 50MB）",
    request_body(content = String, description = "文件数据（multipart/form-data），字段名 file"),
    responses(
        (status = 200, description = "上传成功", body = ApiResponse<ClipboardEntryResponse>),
        (status = 400, description = "文件过大或参数错误"),
        (status = 401, description = "需要频道 token"),
    ),
)]
pub async fn create_file_handler(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    mut payload: Multipart,
) -> HttpResult {
    let claims = extract_channel_claims(&req)?;

    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|e| AppError::MultipartError(e.to_string()))?;
        let field_name = field
            .content_disposition()
            .and_then(|cd| cd.get_name().map(|s| s.to_string()))
            .unwrap_or_default();

        if field_name == "file" {
            let result =
                ClipboardService::create_file(db.as_ref(), claims.channel_id, &mut field).await?;
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
    description = "分页获取当前频道的剪贴板条目（需要频道 token）",
    params(ClipboardQuery),
    responses(
        (status = 200, description = "获取成功", body = ApiResponse<PaginatedResp<ClipboardEntryResponse>>),
        (status = 401, description = "需要频道 token"),
    ),
)]
pub async fn list_handler(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    query: web::Query<ClipboardQuery>,
) -> HttpResult {
    let claims = extract_channel_claims(&req)?;
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20);
    let type_filter = query.r#type.clone();
    let search = query.q.clone();
    let start_date = query.start_date.clone();
    let end_date = query.end_date.clone();

    let result = ClipboardService::list(
        db.as_ref(),
        claims.channel_id,
        page,
        limit,
        type_filter,
        search,
        start_date,
        end_date,
    )
    .await?;
    Ok(ApiResponse::success(result, "获取成功").to_http_response())
}

/// 删除剪贴板条目
#[utoipa::path(
    delete,
    path = "/api/v1/clipboard/{uuid}",
    tag = "云剪贴板",
    summary = "删除剪贴板条目",
    description = "根据 UUID 删除指定的剪贴板条目（需要频道 token）",
    params(
        ("uuid" = String, Path, description = "条目 UUID")
    ),
    responses(
        (status = 200, description = "删除成功", body = ApiResponse<EmptyResponse>),
        (status = 401, description = "需要频道 token"),
        (status = 403, description = "无权操作"),
        (status = 404, description = "条目不存在"),
    ),
)]
pub async fn delete_handler(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    path: web::Path<String>,
) -> HttpResult {
    let claims = extract_channel_claims(&req)?;
    let uuid = path.into_inner();

    ClipboardService::delete(db.as_ref(), claims.channel_id, &uuid).await?;
    Ok(ApiResponse::<EmptyResponse>::success(EmptyResponse, "删除成功").to_http_response())
}
