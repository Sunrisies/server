use crate::utils::file_size::FileSize;
use crate::{ApiResponse, HttpResult, config::AppError};
use actix_multipart::Multipart;
use actix_web::Result;
use anyhow::{Context, Result as AnyhowResult};
use chrono::{Datelike, Local};
use futures_util::StreamExt as _;
use image::ImageFormat;
use log::{error, info};
use qiniu_upload_manager::{
    AutoUploader, AutoUploaderObjectParams, UploadManager as QiNiuUploadManager, UploadTokenSigner,
    apis::credential::Credential,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;
use uuid::Uuid;
// 常量定义
const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024; // 10MB
const ALLOWED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "webp"];
const RESIZE_WIDTH: u32 = 1920;
const RESIZE_HEIGHT: u32 = 1080;

// 配置结构
#[derive(Debug, Clone)]
pub struct QiniuConfig {
    pub access_key: String,
    pub secret_key: String,
    pub bucket_name: String,
    pub base_url: String,
    pub token_expiry_secs: u64,
}

impl Default for QiniuConfig {
    fn default() -> Self {
        dotenvy::dotenv().ok();
        Self {
            access_key: std::env::var("ACCESS_KEY").expect("ACCESS_KEY must be set"),
            secret_key: std::env::var("SECRET_KEY").expect("SECRET_KEY must be set"),
            bucket_name: std::env::var("BUCKET_NAME").expect("BUCKET_NAME must be set"),
            base_url: std::env::var("BASE_URL").expect("BASE_URL must be set"),
            token_expiry_secs: 3600,
        }
    }
}

// 上传结果
#[derive(Debug, Serialize, Deserialize)]
pub struct UploadResult {
    pub url: String,
    pub key: String,
    pub filename: String,
    pub size: u64,
    pub human_readable_size: String, // 新增人类可读的尺寸字段
}

// 文件上传管理器
#[derive(Clone)]
pub struct UploadManager {
    config: QiniuConfig,
}

impl Default for UploadManager {
    fn default() -> Self {
        Self::new(QiniuConfig::default())
    }
}

impl UploadManager {
    pub fn new(config: QiniuConfig) -> Self {
        info!("Upload manager started with bucket: {}", config.bucket_name);
        Self { config }
    }

    /// 处理文件上传请求
    pub async fn handle_upload(&self, mut payload: Multipart) -> HttpResult {
        let file_info = self.process_multipart(&mut payload).await?;

        match self
            .upload_to_qiniu(&file_info.temp_path, &file_info.filename, file_info.size)
            .await
        {
            Ok((final_url, key)) => {
                // 清理临时文件
                let _ = tokio::fs::remove_file(&file_info.temp_path).await;

                info!(
                    "Upload successful: {} ({} bytes)",
                    final_url, file_info.size
                );
                let human_readable_size = FileSize::from(file_info.size).to_string();
                let result = UploadResult {
                    url: final_url,
                    key,
                    filename: file_info.filename,
                    size: file_info.size,
                    human_readable_size,
                };

                Ok(ApiResponse::success(result, "文件上传成功").to_http_response())
            }
            Err(e) => {
                // 清理临时文件
                let _ = tokio::fs::remove_file(&file_info.temp_path).await;
                error!("Upload failed: {}", e);
                Err(AppError::UploadFailed(format!("上传失败: {}", e)))
            }
        }
    }

    /// 处理 multipart 数据，提取文件信息
    async fn process_multipart(&self, payload: &mut Multipart) -> Result<FileInfo, AppError> {
        while let Some(item) = payload.next().await {
            let mut field =
                item.map_err(|e| AppError::InternalServerError(format!("Multipart error: {}", e)))?;

            let content_disposition = match field.content_disposition() {
                Some(cd) => cd,
                None => continue,
            };

            let filename = match content_disposition.get_filename() {
                Some(name) => name.to_string(),
                None => continue,
            };

            info!("开始上传文件: {}", filename);

            // 验证文件扩展名
            self.validate_extension(&filename)?;

            // 创建临时目录和文件
            let (temp_path, mut temp_file) = self.create_temp_file(&filename).await?;

            // 写入文件数据
            let file_size = self.write_file_data(&mut field, &mut temp_file).await?;

            // 处理图片（调整大小等）
            self.process_image(&temp_path, &filename).await?;

            return Ok(FileInfo {
                filename,
                temp_path,
                size: file_size,
            });
        }

        Err(AppError::BadRequest("未找到文件".to_string()))
    }

    /// 验证文件扩展名
    fn validate_extension(&self, filename: &str) -> Result<(), AppError> {
        let extension = Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| AppError::BadRequest("无效的文件扩展名".to_string()))?;

        if !ALLOWED_EXTENSIONS.contains(&extension.to_lowercase().as_str()) {
            return Err(AppError::BadRequest("不支持的图片格式".to_string()));
        }

        Ok(())
    }

    /// 创建临时文件
    async fn create_temp_file(&self, filename: &str) -> Result<(PathBuf, std::fs::File), AppError> {
        let temp_dir = PathBuf::from("temp_uploads");
        fs::create_dir_all(&temp_dir)
            .map_err(|e| AppError::InternalServerError(format!("创建临时目录失败: {}", e)))?;

        let extension = Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("bin");

        let temp_path = temp_dir.join(format!("{}.{}", Uuid::new_v4(), extension));

        let temp_file = std::fs::File::create(&temp_path)
            .map_err(|e| AppError::InternalServerError(format!("创建临时文件失败: {}", e)))?;

        Ok((temp_path, temp_file))
    }

    /// 写入文件数据
    async fn write_file_data(
        &self,
        field: &mut actix_multipart::Field,
        temp_file: &mut std::fs::File,
    ) -> Result<u64, AppError> {
        let mut file_size = 0u64;

        while let Some(chunk) = field.next().await {
            let data = chunk.map_err(|e| AppError::InternalServerError(e.to_string()))?;

            file_size += data.len() as u64;
            if file_size > MAX_FILE_SIZE {
                return Err(AppError::BadRequest("文件过大".to_string()));
            }

            temp_file
                .write_all(&data)
                .map_err(|e| AppError::InternalServerError(format!("写入临时文件失败: {}", e)))?;
        }

        Ok(file_size)
    }

    /// 处理图片（调整大小等）
    async fn process_image(&self, temp_path: &Path, filename: &str) -> Result<(), AppError> {
        // 验证图片有效性
        let image = image::open(temp_path)
            .map_err(|e| AppError::BadRequest(format!("无效的图片文件: {}", e)))?;

        // 调整图片大小
        let resized_image = image.resize(
            RESIZE_WIDTH,
            RESIZE_HEIGHT,
            image::imageops::FilterType::Lanczos3,
        );

        // 重新保存调整后的图片
        let mut temp_file = std::fs::File::create(temp_path)
            .map_err(|e| AppError::InternalServerError(format!("重新打开临时文件失败: {}", e)))?;

        temp_file
            .seek(SeekFrom::Start(0))
            .map_err(|e| AppError::InternalServerError(format!("文件寻址失败: {}", e)))?;

        let extension = Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| AppError::InternalServerError("无效的图片格式".to_string()))?;

        let format = ImageFormat::from_extension(extension)
            .ok_or_else(|| AppError::InternalServerError("不支持的图片格式".to_string()))?;

        resized_image
            .save_with_format(temp_path, format)
            .map_err(|e| AppError::InternalServerError(format!("保存图片失败: {}", e)))?;

        Ok(())
    }

    /// 上传文件到七牛云
    async fn upload_to_qiniu(
        &self,
        file_path: &Path,
        original_filename: &str,
        file_size: u64,
    ) -> AnyhowResult<(String, String)> {
        let credential = Credential::new(&self.config.access_key, &self.config.secret_key);
        let upload_manager =
            QiNiuUploadManager::builder(UploadTokenSigner::new_credential_provider(
                credential,
                &self.config.bucket_name,
                Duration::from_secs(self.config.token_expiry_secs),
            ))
            .build();

        let uploader: AutoUploader = upload_manager.auto_uploader();
        let object_key = self.generate_object_key(original_filename);

        info!(
            "上传文件 '{}' 到: {} ({} 字节)",
            original_filename, object_key, file_size
        );

        let params = AutoUploaderObjectParams::builder()
            .object_name(&object_key)
            .file_name(original_filename)
            .build();

        let response = uploader
            .upload_path(file_path, params)
            .context("无法将文件上传到七牛云")?;

        let key = response["key"]
            .as_str()
            .context("无法从上传响应中获取密钥")?
            .to_string();

        let final_url = self.build_final_url(&key);

        Ok((final_url, key))
    }

    /// 生成对象存储键
    fn generate_object_key(&self, filename: &str) -> String {
        let now = Local::now();
        let file_extension = Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("bin");

        let file_stem = Path::new(filename)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("file");
        let timestamp = now.timestamp_nanos_opt().unwrap_or_else(|| {
            // 如果时间戳溢出，使用秒数作为后备
            now.timestamp() * 1_000_000_000
        });

        format!(
            "uploads/{}/{:02}/{:02}/{}_{}.{}",
            now.year(),
            now.month(),
            now.day(),
            file_stem,
            timestamp,
            file_extension
        )
    }

    /// 构建最终 URL
    fn build_final_url(&self, key: &str) -> String {
        if self.config.base_url.ends_with('/') {
            format!("{}{}", self.config.base_url, key)
        } else {
            format!("{}/{}", self.config.base_url, key)
        }
    }
}

// 文件信息结构
struct FileInfo {
    filename: String,
    temp_path: PathBuf,
    size: u64,
}
