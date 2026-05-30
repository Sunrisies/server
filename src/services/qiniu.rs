//! 七牛云上传通用模块 — 统一 UploadManager 和 ImageService 的重复逻辑

use crate::config::AppError;
use crate::config::manager::CONFIG;
use actix_multipart::Field;
use chrono::{Datelike, Local};
use futures_util::StreamExt;
use image::ImageFormat;
use log::error;
use qiniu_upload_manager::{
    AutoUploader, AutoUploaderObjectParams, UploadManager as QiNiuUploadManager, UploadTokenSigner,
    apis::credential::Credential,
};
use std::fs;
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;
use uuid::Uuid;

/// 七牛云配置（与 config/manager.rs 共用）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QiNiuSettings {
    pub access_key: String,
    pub secret_key: String,
    pub bucket_name: String,
    pub domain_url: String,
    pub token_expiry_secs: u64,
}

// ── 常量 ──

pub const MAX_IMAGE_SIZE: i64 = 20 * 1024 * 1024; // 图片最大 20MB
pub const MAX_FILE_SIZE: i64 = 200 * 1024 * 1024; // 其他文件最大 200MB
const ALLOWED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "webp"];
const RESIZE_WIDTH: u32 = 1920;
const RESIZE_HEIGHT: u32 = 1080;

// ── 上传结果 ──

/// 七牛上传结果
#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct UploadResult {
    pub url: String,
    pub key: String,
    pub filename: String,
    pub size: i64,
    pub human_readable_size: String,
    pub mime_type: String,
}

// ── 临时文件信息 ──

pub struct TempFile {
    pub path: PathBuf,
    pub size: i64,
    pub filename: String,
    pub mime_type: String,
}

// ── 公共函数 ──

/// 验证文件扩展名是否为图片格式
pub fn validate_extension(filename: &str) -> Result<(), AppError> {
    let ext = Path::new(filename)
        .extension()
        .and_then(|ext| ext.to_str())
        .ok_or_else(|| AppError::BadRequest("无效的文件扩展名".to_string()))?;

    if !ALLOWED_EXTENSIONS.contains(&ext.to_lowercase().as_str()) {
        return Err(AppError::BadRequest("不支持的图片格式".to_string()));
    }
    Ok(())
}

/// 从 multipart 字段写入临时文件，返回文件信息
/// max_size 指定最大允许的字节数，超过立即返回错误
pub async fn save_to_temp(field: &mut Field, max_size: i64) -> Result<TempFile, AppError> {
    let content_disposition = field
        .content_disposition()
        .ok_or_else(|| AppError::BadRequest("缺少文件字段".to_string()))?;
    let filename = content_disposition
        .get_filename()
        .ok_or_else(|| AppError::BadRequest("缺少文件名".to_string()))?
        .to_string();

    let temp_dir = PathBuf::from("temp_uploads");
    fs::create_dir_all(&temp_dir)
        .map_err(|e| AppError::InternalServerError(format!("创建临时目录失败: {e}")))?;

    let ext = Path::new(&filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("bin");
    let temp_path = temp_dir.join(format!("upload_{}.{}", Uuid::new_v4(), ext));

    let mut temp_file = fs::File::create(&temp_path)
        .map_err(|e| AppError::InternalServerError(format!("创建临时文件失败: {e}")))?;

    let mut total_size: i64 = 0;
    let mut detected_mime = "application/octet-stream".to_string();
    let mut is_first = true;

    while let Some(chunk) = field.next().await {
        let data = chunk.map_err(|e| AppError::InternalServerError(e.to_string()))?;
        if is_first && !data.is_empty() {
            detected_mime = detect_mime(&data, &filename);
            is_first = false;
        }
        total_size += data.len() as i64;
        if total_size > max_size {
            let _ = fs::remove_file(&temp_path);
            return Err(AppError::BadRequest(format!(
                "文件过大，最大允许 {}MB",
                max_size / (1024 * 1024)
            )));
        }
        temp_file
            .write_all(&data)
            .map_err(|e| AppError::InternalServerError(format!("写入临时文件失败: {e}")))?;
    }

    Ok(TempFile {
        path: temp_path,
        size: total_size,
        filename,
        mime_type: detected_mime,
    })
}

/// 处理图片（调整大小）
pub fn process_image(temp_path: &Path, filename: &str) -> Result<(), AppError> {
    let image = image::open(temp_path).map_err(|e| {
        error!("无效的图片文件(已隐藏): {:?}", e);
        AppError::BadRequest("无效的图片文件，请上传正确的图片格式".to_string())
    })?;

    let resized = image.resize(
        RESIZE_WIDTH,
        RESIZE_HEIGHT,
        image::imageops::FilterType::Lanczos3,
    );

    let mut file = fs::File::create(temp_path)
        .map_err(|e| AppError::InternalServerError(format!("重新打开临时文件失败: {e}")))?;
    file.seek(SeekFrom::Start(0))
        .map_err(|e| AppError::InternalServerError(format!("文件寻址失败: {e}")))?;

    let ext = Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("bin");
    let fmt = ImageFormat::from_extension(ext)
        .ok_or_else(|| AppError::InternalServerError("不支持的图片格式".to_string()))?;

    resized
        .save_with_format(temp_path, fmt)
        .map_err(|e| AppError::InternalServerError(format!("保存图片失败: {e}")))?;
    Ok(())
}

/// 上传文件到七牛云
pub fn upload_to_qiniu(
    file_path: &Path,
    filename: &str,
    key_prefix: &str,
) -> Result<UploadResult, AppError> {
    let credential = Credential::new(&CONFIG.qi_niu.access_key, &CONFIG.qi_niu.secret_key);
    let manager = QiNiuUploadManager::builder(UploadTokenSigner::new_credential_provider(
        credential,
        &CONFIG.qi_niu.bucket_name,
        Duration::from_secs(CONFIG.qi_niu.token_expiry_secs),
    ))
    .build();

    let uploader: AutoUploader = manager.auto_uploader();
    let object_key = generate_object_key(filename, key_prefix);

    let params = AutoUploaderObjectParams::builder()
        .object_name(&object_key)
        .file_name(filename)
        .build();

    let response = uploader.upload_path(file_path, params).map_err(|e| {
        error!("七牛上传失败(已隐藏): {:?}", e);
        AppError::UploadFailed("文件上传失败，请重试".to_string())
    })?;

    let key = response["key"]
        .as_str()
        .ok_or_else(|| AppError::UploadFailed("七牛响应缺少 key".to_string()))?
        .to_string();

    let url = build_final_url(&key);

    let file_size = fs::metadata(file_path).map(|m| m.len() as i64).unwrap_or(0);
    let mime_type = detect_mime_by_ext(filename);
    let human_readable = crate::utils::file_size::FileSize::from(file_size).to_string();

    Ok(UploadResult {
        url,
        key,
        filename: filename.to_string(),
        size: file_size,
        human_readable_size: human_readable,
        mime_type,
    })
}

/// 上传完整流程：验证 → 临时文件 → resize（图片） → 七牛上传 → 清理临时文件
pub async fn upload_file(
    field: &mut Field,
    key_prefix: &str,
    need_resize: bool,
) -> Result<UploadResult, AppError> {
    // 判断是否为图片（需要 resize 的就是图片）
    let max_size = if need_resize {
        MAX_IMAGE_SIZE
    } else {
        MAX_FILE_SIZE
    };
    let temp = save_to_temp(field, max_size).await?;

    if need_resize && let Err(e) = process_image(&temp.path, &temp.filename) {
        let _ = fs::remove_file(&temp.path);
        return Err(e);
    }

    let result = upload_to_qiniu(&temp.path, &temp.filename, key_prefix)?;
    let _ = fs::remove_file(&temp.path);
    Ok(result)
}

// ── 内部工具 ──

fn generate_object_key(filename: &str, prefix: &str) -> String {
    let now = Local::now();
    let ext = Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("bin");
    let stem = Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file");
    format!(
        "{}/{}/{}{:02}/{:02}_{}_{}.{}",
        prefix,
        Uuid::new_v4(),
        now.year(),
        now.month(),
        now.day(),
        stem,
        now.timestamp(),
        ext
    )
}

fn build_final_url(key: &str) -> String {
    let domain = &CONFIG.qi_niu.domain_url;
    if domain.ends_with('/') {
        format!("{}{}", domain, key)
    } else {
        format!("{}/{}", domain, key)
    }
}

fn detect_mime(data: &[u8], filename: &str) -> String {
    if data.len() >= 4 && data[0..4] == [0x89, 0x50, 0x4E, 0x47] {
        return "image/png".to_string();
    }
    if data.len() >= 2 && data[0..2] == [0xFF, 0xD8] {
        return "image/jpeg".to_string();
    }
    if data.len() >= 4 && data[0..4] == [0x52, 0x49, 0x46, 0x46] {
        return "image/webp".to_string();
    }
    detect_mime_by_ext(filename)
}

/// 从七牛云删除文件
pub fn delete_from_qiniu(key: &str) -> std::result::Result<(), String> {
    use base64::engine::Engine as _;
    use hmac::Mac;

    let entry = format!("{}:{}", CONFIG.qi_niu.bucket_name, key);
    let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(entry.as_bytes());
    let path = format!("/delete/{}", encoded);

    // HMAC-SHA1 签名
    let mut mac = hmac::Hmac::<sha1::Sha1>::new_from_slice(CONFIG.qi_niu.secret_key.as_bytes())
        .map_err(|e| format!("HMAC 初始化失败: {}", e))?;
    mac.update(path.as_bytes());
    let result = mac.finalize();
    let digest = result.into_bytes();
    let sign = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(digest);

    let auth = format!("QBox {}:{}", CONFIG.qi_niu.access_key, sign);
    let url = format!("http://rs.qiniu.com{}", path);

    let resp = ureq::post(&url)
        .set("Authorization", &auth)
        .call()
        .map_err(|e| format!("HTTP 请求失败: {}", e))?;

    let status = resp.status();
    if status == 200 {
        log::info!("七牛文件删除成功: {}", key);
        Ok(())
    } else {
        let body = resp.into_string().unwrap_or_default();
        Err(format!("七牛删除失败 ({}): {}", status, body))
    }
}

fn detect_mime_by_ext(filename: &str) -> String {
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
