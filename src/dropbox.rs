//! Expiring text and media delivery. Download codes are bearer secrets.
use actix_multipart::Multipart;
use actix_web::{HttpMessage, HttpRequest, HttpResponse, web};
use chrono::{Duration, Utc};
use futures_util::StreamExt;
use rand::Rng;
use sea_orm::{ConnectionTrait, DatabaseConnection, Statement, TransactionTrait};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};
use uuid::Uuid;

use crate::{ApiResponse, AppError, HttpResult};

const MAX_TEXT_SIZE: usize = 10 * 1024 * 1024;
const MAX_MEDIA_SIZE: usize = 100 * 1024 * 1024;
const TTL_HOURS: i64 = 24;
static STORAGE: OnceLock<PathBuf> = OnceLock::new();

#[derive(Deserialize)]
pub struct TextRequest {
    content: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DropCreated {
    id: String,
    code: String,
    name: String,
    kind: &'static str,
    size: usize,
    expires_at: String,
}

fn storage_dir() -> &'static Path {
    STORAGE.get_or_init(|| {
        PathBuf::from(
            std::env::var("DROP_STORAGE_DIR").unwrap_or_else(|_| "data/dropbox".to_owned()),
        )
    })
}

pub async fn initialize(db: &DatabaseConnection) -> Result<(), AppError> {
    db.execute(Statement::from_string(db.get_database_backend(), "CREATE TABLE IF NOT EXISTS temporary_drops (id VARCHAR(36) PRIMARY KEY, code_hash VARCHAR(64) NOT NULL, token_hash VARCHAR(64) NOT NULL, file_name TEXT NOT NULL, mime_type VARCHAR(255) NOT NULL, file_size BIGINT NOT NULL, text_content TEXT, storage_path TEXT, created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(), expires_at TIMESTAMPTZ NOT NULL)".to_owned())).await?;
    db.execute(Statement::from_string(db.get_database_backend(), "CREATE UNIQUE INDEX IF NOT EXISTS idx_temporary_drops_code_hash ON temporary_drops(code_hash)".to_owned())).await?;
    db.execute(Statement::from_string(db.get_database_backend(), "CREATE TABLE IF NOT EXISTS temporary_drop_limits (ip_hash VARCHAR(64) PRIMARY KEY, window_start TIMESTAMPTZ NOT NULL, hits INTEGER NOT NULL)".to_owned())).await?;
    db.execute(Statement::from_string(db.get_database_backend(), "CREATE TABLE IF NOT EXISTS temporary_drop_pickup_limits (ip_hash VARCHAR(64) PRIMARY KEY, window_start TIMESTAMPTZ NOT NULL, hits INTEGER NOT NULL)".to_owned())).await?;
    std::fs::create_dir_all(storage_dir())
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(storage_dir(), std::fs::Permissions::from_mode(0o700))
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    }
    cleanup(db).await?;
    Ok(())
}

pub fn start_cleanup_task(db: DatabaseConnection) {
    actix_web::rt::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
            if let Err(error) = cleanup(&db).await {
                log::error!("Temporary drop cleanup failed: {error:?}");
            }
        }
    });
}

pub fn config_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/v1/drop")
            .route("/text", web::post().to(create_text))
            .route("/file", web::post().to(create_file))
            .route("/{code}", web::get().to(download)),
    );
}

fn pickup_code() -> String {
    format!("{:08}", rand::thread_rng().gen_range(0..100_000_000))
}
fn digest(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}
fn safe_name(name: &str) -> String {
    let leaf = Path::new(name)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("download.txt");
    leaf.chars()
        .filter(|c| !c.is_control() && *c != '/' && *c != '\\')
        .take(180)
        .collect::<String>()
        .trim()
        .to_owned()
}
fn is_text(name: &str, mime: &str) -> bool {
    mime.starts_with("text/")
        || [
            "json",
            "xml",
            "javascript",
            "yaml",
            "toml",
            "x-www-form-urlencoded",
        ]
        .iter()
        .any(|part| mime.contains(part))
        || [
            "txt", "md", "markdown", "log", "json", "csv", "yaml", "yml", "xml", "html", "css",
            "js", "ts", "py", "rs", "sh", "toml", "ini", "conf", "sql",
        ]
        .iter()
        .any(|ext| name.to_ascii_lowercase().ends_with(&format!(".{ext}")))
}

fn is_media(name: &str, mime: &str) -> bool {
    let mime = mime.split(';').next().unwrap_or(mime).trim();
    if ["image/", "audio/", "video/"]
        .iter()
        .any(|prefix| mime.starts_with(prefix))
    {
        return true;
    }
    [
        "jpg", "jpeg", "png", "gif", "webp", "avif", "heic", "mp3", "wav", "ogg", "m4a", "aac",
        "flac", "opus", "mp4", "webm", "mov", "m4v", "mkv", "avi",
    ]
    .iter()
    .any(|ext| name.to_ascii_lowercase().ends_with(&format!(".{ext}")))
}

async fn save_drop(
    db: &DatabaseConnection,
    name: String,
    mime: String,
    bytes: Vec<u8>,
    text_content: Option<String>,
) -> Result<DropCreated, AppError> {
    if bytes.is_empty() {
        return Err(AppError::BadRequest("文件内容不能为空".into()));
    }
    let is_media_file = is_media(&name, &mime);
    let max_size = if is_media_file {
        MAX_MEDIA_SIZE
    } else {
        MAX_TEXT_SIZE
    };
    if bytes.len() > max_size {
        let message = if is_media_file {
            "图片、音频或视频不能超过 100 MB"
        } else {
            "文本文件不能超过 10 MB"
        };
        return Err(AppError::FileTooLarge(message.into()));
    }
    let id = Uuid::new_v4().to_string();
    let is_text = text_content.is_some();
    let expires = Utc::now() + Duration::hours(TTL_HOURS);
    let mut storage_path = None;
    if text_content.is_none() {
        let path = storage_dir().join(&id);
        tokio::fs::write(&path, &bytes)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            tokio::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))
                .await
                .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        }
        storage_path = Some(path.to_string_lossy().to_string());
    }
    let mut code = String::new();
    let mut inserted = false;
    for _ in 0..10 {
        code = pickup_code();
        let result = db.execute(Statement::from_sql_and_values(db.get_database_backend(),
            "INSERT INTO temporary_drops(id,code_hash,token_hash,file_name,mime_type,file_size,text_content,storage_path,expires_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9) ON CONFLICT (code_hash) DO NOTHING",
            [id.clone().into(), digest(&code).into(), digest(&code).into(), name.clone().into(), mime.clone().into(), (bytes.len() as i64).into(), text_content.clone().into(), storage_path.clone().into(), expires.into()])).await;
        match result {
            Ok(result) if result.rows_affected() == 1 => {
                inserted = true;
                break;
            }
            Ok(_) => continue,
            Err(error) => {
                if let Some(path) = storage_path {
                    let _ = tokio::fs::remove_file(path).await;
                }
                return Err(error.into());
            }
        }
    }
    if !inserted {
        if let Some(path) = storage_path {
            let _ = tokio::fs::remove_file(path).await;
        }
        return Err(AppError::InternalServerError(
            "暂时无法生成取件码，请重试".into(),
        ));
    }
    Ok(DropCreated {
        id,
        code,
        name,
        kind: if is_text { "text" } else { "file" },
        size: bytes.len(),
        expires_at: expires.to_rfc3339(),
    })
}

async fn check_rate_limit(req: &HttpRequest, db: &DatabaseConnection) -> Result<(), AppError> {
    let ip_hash = digest(&crate::utils::client_ip::client_ip(req));
    let row = db.query_one(Statement::from_sql_and_values(db.get_database_backend(),
        "INSERT INTO temporary_drop_limits(ip_hash,window_start,hits) VALUES($1,NOW(),1) ON CONFLICT(ip_hash) DO UPDATE SET window_start=CASE WHEN temporary_drop_limits.window_start < NOW()-INTERVAL '1 hour' THEN NOW() ELSE temporary_drop_limits.window_start END,hits=CASE WHEN temporary_drop_limits.window_start < NOW()-INTERVAL '1 hour' THEN 1 ELSE temporary_drop_limits.hits+1 END RETURNING hits",
        [ip_hash.into()])).await?.ok_or_else(|| AppError::DatabaseError("rate limit row missing".into()))?;
    let hits: i32 = row.try_get("", "hits")?;
    if hits > 20 {
        return Err(AppError::RateLimited("上传太频繁，请一小时后再试".into()));
    }
    Ok(())
}

async fn create_text(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    body: web::Json<TextRequest>,
) -> HttpResult {
    check_rate_limit(&req, db.get_ref()).await?;
    let content = body.content.trim();
    if content.is_empty() {
        return Err(AppError::BadRequest("文本不能为空".into()));
    }
    if content.len() > MAX_TEXT_SIZE {
        return Err(AppError::FileTooLarge("文本不能超过 10 MB".into()));
    }
    let created = save_drop(
        db.get_ref(),
        "text.txt".into(),
        "text/plain; charset=utf-8".into(),
        content.as_bytes().to_vec(),
        Some(content.to_owned()),
    )
    .await?;
    Ok(ApiResponse::success(created, "取件链接已生成").to_http_response())
}

async fn create_file(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    mut payload: Multipart,
) -> HttpResult {
    check_rate_limit(&req, db.get_ref()).await?;
    let authenticated = req
        .extensions()
        .get::<crate::utils::jwt::TokenClaims>()
        .is_some();
    while let Some(field) = payload.next().await {
        let mut field = field.map_err(|e| AppError::MultipartError(e.to_string()))?;
        let disposition = field.content_disposition().cloned();
        if disposition.as_ref().and_then(|value| value.get_name()) != Some("file") {
            continue;
        }
        let name = safe_name(
            disposition
                .as_ref()
                .and_then(|value| value.get_filename())
                .unwrap_or("download.txt"),
        );
        let mime = field
            .content_type()
            .map(ToString::to_string)
            .unwrap_or_else(|| "text/plain".into());
        let text_file = is_text(&name, &mime);
        let media_file = is_media(&name, &mime);
        if !text_file && !(authenticated && media_file) {
            let message = if authenticated {
                "只支持文本、图片、音频或视频文件"
            } else {
                "免登录仅支持文本文件；请登录后上传图片、音频或视频"
            };
            return Err(AppError::UnsupportedFileType(message.into()));
        }
        let max_size = if media_file {
            MAX_MEDIA_SIZE
        } else {
            MAX_TEXT_SIZE
        };
        let mut bytes = Vec::new();
        while let Some(chunk) = field.next().await {
            let chunk = chunk.map_err(|e| AppError::MultipartError(e.to_string()))?;
            if bytes.len() + chunk.len() > max_size {
                let message = if media_file {
                    "图片、音频或视频不能超过 100 MB"
                } else {
                    "文本文件不能超过 10 MB"
                };
                return Err(AppError::FileTooLarge(message.into()));
            }
            bytes.extend_from_slice(&chunk);
        }
        if text_file {
            std::str::from_utf8(&bytes)
                .map_err(|_| AppError::UnsupportedFileType("文件必须是 UTF-8 文本".into()))?;
        }
        let created = save_drop(db.get_ref(), name, mime, bytes, None).await?;
        return Ok(ApiResponse::success(created, "取件链接已生成").to_http_response());
    }
    Err(AppError::BadRequest("缺少 file 文件字段".into()))
}

async fn check_pickup_rate_limit(
    req: &HttpRequest,
    db: &DatabaseConnection,
) -> Result<(), AppError> {
    let ip_hash = digest(&crate::utils::client_ip::client_ip(req));
    let row = db.query_one(Statement::from_sql_and_values(db.get_database_backend(),
        "INSERT INTO temporary_drop_pickup_limits(ip_hash,window_start,hits) VALUES($1,NOW(),1) ON CONFLICT(ip_hash) DO UPDATE SET window_start=CASE WHEN temporary_drop_pickup_limits.window_start < NOW()-INTERVAL '15 minutes' THEN NOW() ELSE temporary_drop_pickup_limits.window_start END,hits=CASE WHEN temporary_drop_pickup_limits.window_start < NOW()-INTERVAL '15 minutes' THEN 1 ELSE temporary_drop_pickup_limits.hits+1 END RETURNING hits",
        [ip_hash.into()])).await?.ok_or_else(|| AppError::DatabaseError("取件限流记录缺失".into()))?;
    let hits: i32 = row.try_get("", "hits")?;
    if hits > 30 {
        return Err(AppError::RateLimited(
            "取件尝试过于频繁，请 15 分钟后重试".into(),
        ));
    }
    Ok(())
}

async fn download(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    path: web::Path<String>,
) -> HttpResult {
    check_pickup_rate_limit(&req, db.get_ref()).await?;
    let code = path.into_inner();
    let row = db.query_one(Statement::from_sql_and_values(db.get_database_backend(),
        "SELECT id,file_name,mime_type,text_content,storage_path,expires_at FROM temporary_drops WHERE code_hash=$1 AND expires_at>NOW()",
        [digest(&code).into()])).await?.ok_or_else(|| AppError::NotFound("取件码无效或已过期".into()))?;
    let name: String = row.try_get("", "file_name")?;
    let mime: String = row.try_get("", "mime_type")?;
    let text: Option<String> = row.try_get("", "text_content")?;
    let path: Option<String> = row.try_get("", "storage_path")?;
    let bytes = if let Some(text) = text {
        text.into_bytes()
    } else {
        let path = path.ok_or_else(|| AppError::NotFound("文件不存在".into()))?;
        tokio::fs::read(path)
            .await
            .map_err(|_| AppError::NotFound("文件不存在或已过期".into()))?
    };
    let encoded_name = url::form_urlencoded::byte_serialize(name.as_bytes()).collect::<String>();
    let mut response = HttpResponse::Ok();
    response.insert_header(("Content-Type", mime));
    response.insert_header((
        "Content-Disposition",
        format!("attachment; filename*=UTF-8''{encoded_name}"),
    ));
    response.insert_header(("Cache-Control", "private, no-store"));
    response.insert_header(("X-Content-Type-Options", "nosniff"));
    Ok(response.body(bytes))
}

async fn cleanup(db: &DatabaseConnection) -> Result<(), AppError> {
    let txn = db.begin().await?;
    let rows = txn
        .query_all(Statement::from_string(
            db.get_database_backend(),
            "SELECT storage_path FROM temporary_drops WHERE expires_at<=NOW()".to_owned(),
        ))
        .await?;
    txn.execute(Statement::from_string(
        db.get_database_backend(),
        "DELETE FROM temporary_drops WHERE expires_at<=NOW()".to_owned(),
    ))
    .await?;
    txn.execute(Statement::from_string(
        db.get_database_backend(),
        "DELETE FROM temporary_drop_limits WHERE window_start<NOW()-INTERVAL '24 hours'".to_owned(),
    ))
    .await?;
    txn.execute(Statement::from_string(
        db.get_database_backend(),
        "DELETE FROM temporary_drop_pickup_limits WHERE window_start<NOW()-INTERVAL '24 hours'"
            .to_owned(),
    ))
    .await?;
    txn.commit().await?;
    for row in rows {
        let path: Option<String> = row.try_get("", "storage_path")?;
        if let Some(path) = path {
            let _ = tokio::fs::remove_file(path).await;
        }
    }
    Ok(())
}
