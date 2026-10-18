//! 七牛云上传通用模块 — 统一 UploadManager 和 ImageService 的重复逻辑

use crate::config::AppError;
use crate::config::manager::CONFIG;
use actix_multipart::Field;
use chrono::{Datelike, Local};
use futures_util::StreamExt;
use image::{ImageFormat, ImageReader, Limits};
use log::error;
use qiniu_upload_manager::{
    AutoUploader, AutoUploaderObjectParams, UploadManager as QiNiuUploadManager, UploadTokenSigner,
    apis::credential::Credential,
};
use std::fs;
use std::path::Path;
use std::sync::{Arc, LazyLock};
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
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
const ALLOWED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp"];
const MAX_IMAGE_DIMENSION: u32 = 8192;
const MAX_IMAGE_ALLOC: u64 = 64 * 1024 * 1024;
// Process-wide across Actix workers. The permit stays with detached blocking work.
static UPLOAD_SLOTS: LazyLock<Arc<Semaphore>> = LazyLock::new(|| Arc::new(Semaphore::new(2)));
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
    // Own the file, not just its path. RAII also handles errors, panic and cancelled futures.
    file: tempfile::NamedTempFile,
    pub size: i64,
    pub filename: String,
    pub mime_type: String,
}

impl TempFile {
    fn path(&self) -> &Path {
        self.file.path()
    }
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
    let filename = field
        .content_disposition()
        .and_then(|cd| cd.get_filename())
        .ok_or_else(|| AppError::BadRequest("缺少文件名".into()))?
        .to_owned();
    let (mut saved, mut file) = create_temp(filename, Path::new("temp_uploads").to_owned()).await?;
    while let Some(chunk) = field.next().await {
        let data = chunk.map_err(|_| AppError::BadRequest("文件上传中断，请重试".into()))?;
        if saved.size == 0 && !data.is_empty() {
            saved.mime_type = detect_mime(&data, &saved.filename);
        }
        saved.size += data.len() as i64;
        if saved.size > max_size {
            return Err(AppError::FileTooLarge(format!(
                "文件过大，最大允许 {}MB",
                max_size / (1024 * 1024)
            )));
        }
        file.write_all(&data)
            .await
            .map_err(|_| AppError::StorageServiceError("临时文件写入失败".into()))?;
    }
    file.flush()
        .await
        .map_err(|_| AppError::StorageServiceError("临时文件写入失败".into()))?;
    drop(file);
    if saved.size == 0 {
        return Err(AppError::BadRequest("不能上传空文件".into()));
    }
    Ok(saved)
}

async fn create_temp(
    filename: String,
    directory: std::path::PathBuf,
) -> Result<(TempFile, tokio::fs::File), AppError> {
    actix_web::web::block(move || {
        fs::create_dir_all(&directory)?;
        let file = tempfile::Builder::new()
            .prefix("upload_")
            .tempfile_in(directory)?;
        let writer = file.reopen()?;
        Ok::<_, AppError>((
            TempFile {
                file,
                size: 0,
                filename,
                mime_type: "application/octet-stream".into(),
            },
            tokio::fs::File::from_std(writer),
        ))
    })
    .await
    .map_err(|_| AppError::StorageServiceError("创建临时文件失败".into()))?
}

/// Bounded decoder work; called only from a blocking worker in the upload pipeline.
pub fn process_image(temp_path: &Path, filename: &str) -> Result<(), AppError> {
    validate_extension(filename)?;
    let reader = ImageReader::open(temp_path)?.with_guessed_format()?;
    if !matches!(
        reader.format(),
        Some(ImageFormat::Png | ImageFormat::Jpeg | ImageFormat::WebP)
    ) {
        return Err(AppError::UnsupportedFileType(
            "请上传 PNG、JPG 或 WebP 图片".into(),
        ));
    }
    let image = limited_decode(reader)
        .map_err(|_| AppError::BadRequest("图片损坏或尺寸过大，请选择其他图片".into()))?;
    let image = if image.width() > RESIZE_WIDTH || image.height() > RESIZE_HEIGHT {
        image.resize(
            RESIZE_WIDTH,
            RESIZE_HEIGHT,
            image::imageops::FilterType::Lanczos3,
        )
    } else {
        image
    };
    let ext = Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let format = ImageFormat::from_extension(&ext)
        .ok_or_else(|| AppError::UnsupportedFileType("不支持的图片格式".into()))?;
    image
        .save_with_format(temp_path, format)
        .map_err(|_| AppError::UploadFailed("图片处理失败，请重试".into()))?;
    Ok(())
}

fn limited_decode<R: std::io::BufRead + std::io::Seek>(
    mut reader: ImageReader<R>,
) -> image::ImageResult<image::DynamicImage> {
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_IMAGE_DIMENSION);
    limits.max_image_height = Some(MAX_IMAGE_DIMENSION);
    limits.max_alloc = Some(MAX_IMAGE_ALLOC);
    reader.limits(limits);
    reader.decode()
}

/// 上传文件到七牛云
pub fn upload_to_qiniu(
    file_path: &Path,
    filename: &str,
    key_prefix: &str,
) -> Result<UploadResult, AppError> {
    upload_to_qiniu_with_key(
        file_path,
        filename,
        &generate_object_key(filename, key_prefix),
    )
}

fn upload_to_qiniu_with_key(
    file_path: &Path,
    filename: &str,
    object_key: &str,
) -> Result<UploadResult, AppError> {
    let credential = Credential::new(&CONFIG.qi_niu.access_key, &CONFIG.qi_niu.secret_key);
    let manager = QiNiuUploadManager::builder(UploadTokenSigner::new_credential_provider(
        credential,
        &CONFIG.qi_niu.bucket_name,
        Duration::from_secs(CONFIG.qi_niu.token_expiry_secs),
    ))
    .build();

    let uploader: AutoUploader = manager.auto_uploader();
    let params = AutoUploaderObjectParams::builder()
        .object_name(object_key)
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
    prepare_upload(field, key_prefix, need_resize)
        .await?
        .run()
        .await
}

pub(crate) struct PreparedUpload {
    temp: TempFile,
    permit: OwnedSemaphorePermit,
    resize: bool,
    prefix: String,
    pub key: String,
}
impl PreparedUpload {
    pub async fn run(self) -> Result<UploadResult, AppError> {
        let key = self.key;
        finish_upload(
            self.temp,
            self.permit,
            self.resize,
            self.prefix,
            move |path, name, _| upload_to_qiniu_with_key(path, name, &key),
        )
        .await
    }
}

pub(crate) async fn prepare_upload(
    field: &mut Field,
    key_prefix: &str,
    need_resize: bool,
) -> Result<PreparedUpload, AppError> {
    if need_resize {
        let name = field
            .content_disposition()
            .and_then(|cd| cd.get_filename())
            .ok_or_else(|| AppError::BadRequest("缺少文件名".into()))?;
        validate_extension(name)?;
    }
    let permit = acquire_upload_slot(UPLOAD_SLOTS.clone()).await?;
    let max_size = if need_resize {
        MAX_IMAGE_SIZE
    } else {
        MAX_FILE_SIZE
    };
    let temp = save_to_temp(field, max_size).await?;
    // Reject invalid images before recording a remote intent or contacting storage.
    let (temp, permit) = actix_web::web::block(move || {
        let permit = permit;
        let temp = temp;
        if need_resize {
            process_image(temp.path(), &temp.filename)?;
        }
        Ok::<_, AppError>((temp, permit))
    })
    .await
    .map_err(|_| AppError::UploadFailed("图片处理任务失败".into()))??;
    let key = generate_object_key(&temp.filename, key_prefix);
    Ok(PreparedUpload {
        temp,
        permit,
        resize: false,
        prefix: key_prefix.to_owned(),
        key,
    })
}

async fn acquire_upload_slot(slots: Arc<Semaphore>) -> Result<OwnedSemaphorePermit, AppError> {
    tokio::time::timeout(Duration::from_secs(2), slots.acquire_owned())
        .await
        .map_err(|_| AppError::MaintenanceMode("上传任务较多，请稍后重试".into()))?
        .map_err(|_| AppError::MaintenanceMode("上传服务暂时不可用".into()))
}

async fn finish_upload<F>(
    temp: TempFile,
    permit: OwnedSemaphorePermit,
    resize: bool,
    prefix: String,
    upload: F,
) -> Result<UploadResult, AppError>
where
    F: FnOnce(&Path, &str, &str) -> Result<UploadResult, AppError> + Send + 'static,
{
    actix_web::web::block(move || {
        let _permit = permit;
        // Declared after the permit so cleanup precedes slot release on error/panic too.
        let temp = temp;
        if resize {
            process_image(temp.path(), &temp.filename)?;
        }
        let result = upload(temp.path(), &temp.filename, &prefix);
        // temp drops inside the blocking worker on both success and error.
        drop(temp);
        result
    })
    .await
    .map_err(|_| AppError::UploadFailed("图片处理或上传任务失败，请重试".into()))?
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
    let entry = format!("{}:{}", CONFIG.qi_niu.bucket_name, key);
    let encoded = base64::engine::general_purpose::URL_SAFE.encode(entry.as_bytes());
    let path = format!("/delete/{}", encoded);
    let auth = delete_authorization(&CONFIG.qi_niu.access_key, &CONFIG.qi_niu.secret_key, &path);
    let url = format!("https://rs.qiniu.com{}", path);

    let resp = ureq::post(&url)
        .set("Authorization", &auth)
        .timeout(Duration::from_secs(5))
        .call();
    let resp = match resp {
        Ok(resp) => resp,
        Err(ureq::Error::Status(612, _)) => return Ok(()), // already absent; safe retry
        Err(error) => return Err(format!("HTTP 请求失败: {error}")),
    };

    let status = resp.status();
    if status == 200 {
        log::info!("七牛文件删除成功: {}", key);
        Ok(())
    } else {
        let body = resp.into_string().unwrap_or_default();
        Err(format!("七牛删除失败 ({}): {}", status, body))
    }
}

// QBox v1 signs the path plus a mandatory newline. SDK signing retains URL-safe padding.
fn delete_authorization(access_key: &str, secret_key: &str, path: &str) -> String {
    format!(
        "QBox {}",
        Credential::new(access_key, secret_key).sign(format!("{path}\n").as_bytes())
    )
}

#[cfg(test)]
mod management_signature_tests {
    use super::*;
    #[test]
    fn delete_signature_includes_newline_and_sdk_padding() {
        use base64::Engine;
        use hmac::Mac;
        let path = "/delete/dGVzdDppbWFnZS5wbmc=";
        let mut mac = hmac::Hmac::<sha1::Sha1>::new_from_slice(b"test-secret").unwrap();
        mac.update(format!("{path}\n").as_bytes());
        let expected =
            base64::engine::general_purpose::URL_SAFE.encode(mac.finalize().into_bytes());
        assert_eq!(
            delete_authorization("test-access", "test-secret", path),
            format!("QBox test-access:{expected}")
        );
        assert!(expected.ends_with('='));
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

#[cfg(test)]
mod upload_safety_tests;
