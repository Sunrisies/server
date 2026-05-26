use crate::config::AppError;
use crate::config::manager::CONFIG;
use crate::dto::clipboard::{ClipboardEntryResponse, CreateTextRequest, UploadResult};
use crate::dto::common::{PaginatedResp, Pagination};
use crate::models::clipboard_entries;
use crate::models::clipboard_entries::Entity as ClipboardEntity;
use actix_multipart::Field;
use chrono::{Datelike, Local, Utc};
use futures_util::StreamExt;
use log::error;
use qiniu_upload_manager::{
    AutoUploader, AutoUploaderObjectParams, UploadManager as QiNiuUploadManager, UploadTokenSigner,
    apis::credential::Credential,
};
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder,
};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;
use uuid::Uuid;

const MAX_FILE_SIZE: i64 = 50 * 1024 * 1024; // 50MB
const MAX_TEXT_LENGTH: usize = 100_000;

pub struct ClipboardService;

impl ClipboardService {
    /// 上传文本
    pub async fn create_text(
        db: &DatabaseConnection,
        user_id: i32,
        req: CreateTextRequest,
    ) -> Result<ClipboardEntryResponse, AppError> {
        let content = req.content.trim().to_string();
        if content.is_empty() {
            return Err(AppError::BadRequest("内容不能为空".to_string()));
        }
        if content.len() > MAX_TEXT_LENGTH {
            return Err(AppError::BadRequest(format!(
                "内容过长，最大允许 {} 字符",
                MAX_TEXT_LENGTH
            )));
        }

        let now = Utc::now();
        let model = clipboard_entries::ActiveModel {
            uuid: Set(Uuid::new_v4().to_string()),
            user_id: Set(user_id),
            r#type: Set("text".to_string()),
            content: Set(Some(content)),
            pinned: Set(false),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        let entry = model.insert(db).await.map_err(|e| {
            error!("创建剪贴文本失败: {:?}", e);
            AppError::DatabaseError("创建失败".to_string())
        })?;

        Ok(entry.into())
    }

    /// 上传文件（图片/任意文件）
    pub async fn create_file(
        db: &DatabaseConnection,
        user_id: i32,
        field: &mut Field,
    ) -> Result<ClipboardEntryResponse, AppError> {
        // 1. 获取文件名
        let content_disposition = field
            .content_disposition()
            .ok_or_else(|| AppError::BadRequest("缺少文件字段".to_string()))?;
        let file_name = content_disposition
            .get_filename()
            .ok_or_else(|| AppError::BadRequest("缺少文件名".to_string()))?
            .to_string();

        // 2. 写入临时文件，同时检测大小
        let (temp_path, file_size, mime_type) = Self::save_temp_file(field, &file_name).await?;

        // 3. 上传到七牛
        let upload_result = Self::upload_to_qiniu(&temp_path, &file_name, file_size, &mime_type)
            .map_err(|e| {
                let _ = fs::remove_file(&temp_path);
                error!("七牛上传失败: {:?}", e);
                AppError::UploadFailed("文件上传失败，请重试".to_string())
            })?;

        // 清理临时文件
        let _ = fs::remove_file(&temp_path);

        // 4. 检测类型
        let entry_type = if mime_type.starts_with("image/") {
            "image"
        } else {
            "file"
        };

        // 5. 写入 DB
        let now = Utc::now();
        let model = clipboard_entries::ActiveModel {
            uuid: Set(Uuid::new_v4().to_string()),
            user_id: Set(user_id),
            r#type: Set(entry_type.to_string()),
            content: Set(None),
            file_url: Set(Some(upload_result.url)),
            file_name: Set(Some(upload_result.file_name)),
            file_size: Set(Some(upload_result.file_size)),
            mime_type: Set(Some(upload_result.mime_type)),
            pinned: Set(false),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        let entry = model.insert(db).await.map_err(|e| {
            error!("保存剪贴文件失败: {:?}", e);
            AppError::DatabaseError("保存失败".to_string())
        })?;

        Ok(entry.into())
    }

    /// 获取列表
    pub async fn list(
        db: &DatabaseConnection,
        user_id: i32,
        page: u64,
        limit: u64,
        type_filter: Option<String>,
    ) -> Result<PaginatedResp<ClipboardEntryResponse>, AppError> {
        let mut query = ClipboardEntity::find()
            .filter(clipboard_entries::Column::UserId.eq(user_id))
            .order_by_desc(clipboard_entries::Column::Pinned)
            .order_by_desc(clipboard_entries::Column::CreatedAt);

        if let Some(ref t) = type_filter {
            query = query.filter(clipboard_entries::Column::Type.eq(t));
        }

        let paginator = query.paginate(db, limit);
        let total = paginator.num_items().await.map_err(|e| {
            error!("查询剪贴板总数失败: {:?}", e);
            AppError::DatabaseError("查询失败".to_string())
        })?;

        let data = paginator
            .fetch_page(page.saturating_sub(1))
            .await
            .map_err(|e| {
                error!("查询剪贴板列表失败: {:?}", e);
                AppError::DatabaseError("查询失败".to_string())
            })?;

        let items: Vec<ClipboardEntryResponse> = data.into_iter().map(Into::into).collect();

        Ok(PaginatedResp {
            data: items,
            pagination: Pagination { total, page, limit },
        })
    }

    /// 删除
    pub async fn delete(db: &DatabaseConnection, user_id: i32, uuid: &str) -> Result<(), AppError> {
        let entry = ClipboardEntity::find_by_uuid(uuid)
            .one(db)
            .await
            .map_err(|e| {
                error!("查询剪贴条目失败: {:?}", e);
                AppError::DatabaseError("查询失败".to_string())
            })?
            .ok_or_else(|| AppError::NotFound("条目不存在".to_string()))?;

        if entry.user_id != user_id {
            return Err(AppError::Forbidden("无权操作该条目".to_string()));
        }

        // 如果是文件类型，尝试删除七牛文件（失败不阻塞）
        if entry.r#type != "text"
            && let Some(ref file_url) = entry.file_url
            && let Some(key) = extract_key_from_url(file_url)
            && let Err(e) = Self::delete_from_qiniu(&key)
        {
            log::warn!("删除七牛文件失败(忽略): {} - {:?}", key, e);
        }

        ClipboardEntity::delete_by_id(entry.id)
            .exec(db)
            .await
            .map_err(|e| {
                error!("删除剪贴条目失败: {:?}", e);
                AppError::DatabaseError("删除失败".to_string())
            })?;

        Ok(())
    }

    // --- 辅助方法 ---

    /// 保存 multipart 文件到临时目录，返回 (路径, 大小, MIME)
    async fn save_temp_file(
        field: &mut Field,
        file_name: &str,
    ) -> Result<(PathBuf, i64, String), AppError> {
        let temp_dir = PathBuf::from("temp_uploads");
        fs::create_dir_all(&temp_dir)
            .map_err(|e| AppError::InternalServerError(format!("创建临时目录失败: {}", e)))?;

        let ext = Path::new(file_name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("bin");
        let temp_path = temp_dir.join(format!("clip_{}.{}", Uuid::new_v4(), ext));

        let mut temp_file = fs::File::create(&temp_path)
            .map_err(|e| AppError::InternalServerError(format!("创建临时文件失败: {}", e)))?;

        let mut total_size: i64 = 0;
        let mut detected_mime = "application/octet-stream".to_string();
        let mut first_chunk = true;

        while let Some(chunk) = field.next().await {
            let data = chunk.map_err(|e| AppError::InternalServerError(e.to_string()))?;

            if first_chunk && !data.is_empty() {
                // 通过魔数检测 MIME（简化：只检测常见图片格式）
                detected_mime = detect_mime(&data, file_name);
                first_chunk = false;
            }

            total_size += data.len() as i64;
            if total_size > MAX_FILE_SIZE {
                let _ = fs::remove_file(&temp_path);
                return Err(AppError::BadRequest(format!(
                    "文件过大，最大允许 {}MB",
                    MAX_FILE_SIZE / (1024 * 1024)
                )));
            }

            temp_file
                .write_all(&data)
                .map_err(|e| AppError::InternalServerError(format!("写入临时文件失败: {}", e)))?;
        }

        Ok((temp_path, total_size, detected_mime))
    }

    /// 上传到七牛云
    fn upload_to_qiniu(
        file_path: &Path,
        original_filename: &str,
        _file_size: i64,
        mime_type: &str,
    ) -> Result<UploadResult, anyhow::Error> {
        let credential = Credential::new(&CONFIG.qi_niu.access_key, &CONFIG.qi_niu.secret_key);
        let upload_manager =
            QiNiuUploadManager::builder(UploadTokenSigner::new_credential_provider(
                credential,
                &CONFIG.qi_niu.bucket_name,
                Duration::from_secs(CONFIG.qi_niu.token_expiry_secs),
            ))
            .build();

        let uploader: AutoUploader = upload_manager.auto_uploader();
        let object_key = Self::generate_object_key(original_filename);

        let params = AutoUploaderObjectParams::builder()
            .object_name(&object_key)
            .file_name(original_filename)
            .build();

        let response = uploader
            .upload_path(file_path, params)
            .map_err(|e| anyhow::anyhow!("七牛上传失败: {}", e))?;

        let key = response["key"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("响应中缺少 key"))?
            .to_string();

        let url = build_final_url(&key);

        Ok(UploadResult {
            url,
            key,
            file_name: original_filename.to_string(),
            file_size: _file_size,
            mime_type: mime_type.to_string(),
        })
    }

    /// 生成七牛对象存储键
    fn generate_object_key(filename: &str) -> String {
        let now = Local::now();
        let ext = Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("bin");
        let stem = Path::new(filename)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("file");
        let uuid = Uuid::new_v4();

        format!(
            "clipboard/{}/{}{:02}/{:02}_{}_{}.{}",
            uuid,
            now.year(),
            now.month(),
            now.day(),
            stem,
            now.timestamp(),
            ext
        )
    }

    /// 从七牛删除文件
    fn delete_from_qiniu(_key: &str) -> Result<(), anyhow::Error> {
        // 七牛上传管理器 SDK 暂不支持直接删除
        // 需要使用七牛管理 API（bucketManager）单独调用
        // 此处预留，删除失败仅 warn 日志，不阻塞 DB 删除
        log::warn!("七牛对象删除未实现，需手动清理或配置过期策略: {}", _key);
        Ok(())
    }
}

/// 从完整 URL 中提取七牛 key
fn extract_key_from_url(url: &str) -> Option<String> {
    // URL 格式: https://domain.com/clipboard/uuid/...
    // 提取 domain 后面的部分
    if let Some(pos) = url.find(".top/") {
        Some(url[pos + 5..].to_string())
    } else if let Some(pos) = url.find(".com/") {
        Some(url[pos + 5..].to_string())
    } else if let Some(pos) = url.find(".cn/") {
        Some(url[pos + 4..].to_string())
    } else {
        // fallback: 尝试取 path
        url.split_once('/').map(|(_, rest)| rest.to_string())
    }
}

/// 构建完整 CDN URL
fn build_final_url(key: &str) -> String {
    if CONFIG.qi_niu.domain_url.ends_with('/') {
        format!("{}{}", CONFIG.qi_niu.domain_url, key)
    } else {
        format!("{}/{}", CONFIG.qi_niu.domain_url, key)
    }
}

/// 通过魔数检测 MIME 类型
fn detect_mime(data: &[u8], filename: &str) -> String {
    if data.len() >= 4 && data[0..4] == [0x89, 0x50, 0x4E, 0x47] {
        return "image/png".to_string();
    }
    if data.len() >= 2 && data[0..2] == [0xFF, 0xD8] {
        return "image/jpeg".to_string();
    }
    if data.len() >= 4 && data[0..4] == [0x52, 0x49, 0x46, 0x46] {
        return "image/webp".to_string(); // simplified
    }
    // fallback: 根据扩展名判断
    let ext = Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "mp4" => "video/mp4",
        "pdf" => "application/pdf",
        "zip" => "application/zip",
        "json" => "application/json",
        "txt" | "md" => "text/plain",
        "html" | "htm" => "text/html",
        _ => "application/octet-stream",
    }
    .to_string()
}
