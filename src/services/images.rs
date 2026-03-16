use crate::config::manager::CONFIG;
use crate::dto::common::PaginationQuery;
use crate::dto::image::ImageUploadResponse;
// use crate::dto::image::ImageUploadResponse;
use crate::utils::file_size::FileSize;
use crate::{ApiResponse, HttpResult, config::AppError};
use actix_multipart::Multipart;
use anyhow::{Context, Result as AnyhowResult};
use chrono::{Datelike, Local};
use futures_util::StreamExt as _;
use image::ImageFormat;
use log::{error, info};
use qiniu_upload_manager::{
    AutoUploader, AutoUploaderObjectParams, UploadManager as QiNiuUploadManager, UploadTokenSigner,
    apis::credential::Credential,
};
use sea_orm::DatabaseConnection;
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

#[derive(Debug, Clone)]
pub struct QiNiuSettings {
    pub access_key: String,
    pub secret_key: String,
    pub bucket_name: String,
    pub domain_url: String,
    pub token_expiry_secs: u64,
}

// 图片上传结果
#[derive(Debug)]
struct UploadResult {
    url: String,
    #[allow(dead_code)]
    key: String,
    #[allow(dead_code)]
    filename: String,
    #[allow(dead_code)]
    size: u64,
    #[allow(dead_code)]
    human_readable_size: String,
}

// 文件信息结构
struct FileInfo {
    filename: String,
    temp_path: PathBuf,
    size: u64,
}

// 图片服务
pub struct ImageService;

impl ImageService {
    /// 处理图片上传
    pub async fn handle_upload(_db: &DatabaseConnection, mut payload: Multipart) -> HttpResult {
        let file_info = Self::process_multipart(&mut payload).await?;

        match Self::upload_to_qiniu(&file_info.temp_path, &file_info.filename, file_info.size).await
        {
            Ok(upload_result) => {
                // 清理临时文件
                let _ = tokio::fs::remove_file(&file_info.temp_path).await;

                info!(
                    "Upload successful: {} ({} bytes)",
                    upload_result.url, file_info.size
                );

                // 保存图片信息到数据库
                // let image_record = Self::save_image_to_db(db, &upload_result).await?;

                let response = ImageUploadResponse {
                    // id: image_record.id,
                    id: 12,
                    url: upload_result.url,
                    key: upload_result.key,
                    filename: upload_result.filename,
                    size: upload_result.size,
                    human_readable_size: upload_result.human_readable_size,
                    created_at: "image_record.created_at.to_rfc3339()".to_string(),
                };

                Ok(ApiResponse::success(response, "图片上传成功").to_http_response())
            }
            Err(e) => {
                // 清理临时文件
                let _ = tokio::fs::remove_file(&file_info.temp_path).await;
                error!("Upload failed: {}", e);
                Err(AppError::UploadFailed(format!("上传失败: {}", e)))
            }
        }
    }

    /// 获取图片列表
    pub async fn get_images(_db: &DatabaseConnection, _query: PaginationQuery) -> HttpResult {
        // let page = query.page.unwrap_or(1);
        // let page_size = query.page_size.unwrap_or(10);

        // let paginator = Images::find()
        //     .order_by_desc(images::Column::CreatedAt)
        //     .paginate(db, page_size);

        // let total = paginator.num_pages().await.unwrap_or(0) as u64 * page_size;
        // let records = paginator
        //     .fetch_page(page - 1)
        //     .await
        //     .map_err(|e| AppError::InternalServerError(format!("获取图片列表失败: {}", e)))?;

        // let images: Vec<ImageInfo> = records
        //     .into_iter()
        //     .map(|record| ImageInfo {
        //         id: record.id,
        //         url: record.url,
        //         key: record.key,
        //         filename: record.filename,
        //         size: record.size,
        //         human_readable_size: record.human_readable_size,
        //         created_at: record.created_at.to_rfc3339(),
        //     })
        //     .collect();

        // let response = ImageListResponse { images, total };

        Ok(ApiResponse::success("response", "获取图片列表成功").to_http_response())
    }

    /// 根据ID获取图片
    pub async fn get_image_by_id(_db: &DatabaseConnection, _image_id: i32) -> HttpResult {
        // let record = Images::find_by_id(image_id)
        //     .one(db)
        //     .await
        //     .map_err(|e| AppError::InternalServerError(format!("获取图片失败: {}", e)))?
        //     .ok_or_else(|| AppError::NotFound("图片不存在".to_string()))?;

        // let image = ImageInfo {
        //     id: record.id,
        //     url: record.url,
        //     key: record.key,
        //     filename: record.filename,
        //     size: record.size,
        //     human_readable_size: record.human_readable_size,
        //     created_at: record.created_at.to_rfc3339(),
        // };

        Ok(ApiResponse::success("image", "获取图片成功").to_http_response())
    }

    /// 删除图片
    pub async fn delete_image(_db: &DatabaseConnection, _image_id: i32) -> HttpResult {
        // let record = Images::find_by_id(image_id)
        //     .one(db)
        //     .await
        //     .map_err(|e| AppError::InternalServerError(format!("获取图片失败: {}", e)))?
        //     .ok_or_else(|| AppError::NotFound("图片不存在".to_string()))?;

        // // TODO: 从七牛云删除文件
        // // Self::delete_from_qiniu(&record.key).await?;

        // // 从数据库删除记录
        // let active_model = images::ActiveModel {
        //     id: Set(record.id),
        //     ..Default::default()
        // };
        // active_model
        //     .delete(db)
        //     .await
        //     .map_err(|e| AppError::InternalServerError(format!("删除图片失败: {}", e)))?;

        Ok(ApiResponse::success((), "图片删除成功").to_http_response())
    }

    /// 处理 multipart 数据，提取文件信息
    async fn process_multipart(payload: &mut Multipart) -> Result<FileInfo, AppError> {
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
            Self::validate_extension(&filename)?;

            // 创建临时目录和文件
            let (temp_path, mut temp_file) = Self::create_temp_file(&filename).await?;

            // 写入文件数据
            let file_size = Self::write_file_data(&mut field, &mut temp_file).await?;

            // 处理图片（调整大小等）
            Self::process_image(&temp_path, &filename).await?;

            return Ok(FileInfo {
                filename,
                temp_path,
                size: file_size,
            });
        }

        Err(AppError::BadRequest("未找到文件".to_string()))
    }

    /// 验证文件扩展名
    fn validate_extension(filename: &str) -> Result<(), AppError> {
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
    async fn create_temp_file(filename: &str) -> Result<(PathBuf, std::fs::File), AppError> {
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
    async fn process_image(temp_path: &Path, filename: &str) -> Result<(), AppError> {
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
        file_path: &Path,
        original_filename: &str,
        file_size: u64,
    ) -> AnyhowResult<UploadResult> {
        log::info!("开始上传文件到七牛云{:?}", &CONFIG.qi_niu);
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

        let final_url = Self::build_final_url(&key);
        let human_readable_size = FileSize::from(file_size).to_string();

        Ok(UploadResult {
            url: final_url,
            key,
            filename: original_filename.to_string(),
            size: file_size,
            human_readable_size,
        })
    }

    /// 生成对象存储键
    fn generate_object_key(filename: &str) -> String {
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
            "images/{}/{:02}/{:02}/{}_{}.{}",
            now.year(),
            now.month(),
            now.day(),
            file_stem,
            timestamp,
            file_extension
        )
    }

    // 构建最终 URL
    fn build_final_url(key: &str) -> String {
        if CONFIG.qi_niu.domain_url.ends_with('/') {
            format!("{}{}", CONFIG.qi_niu.domain_url, key)
        } else {
            format!("{}/{}", CONFIG.qi_niu.domain_url, key)
        }
    }

    // 保存图片信息到数据库
    // async fn save_image_to_db(
    //     &self,
    //     upload_result: &UploadResult,
    // ) -> Result<images::Model, AppError> {
    //     let now = Utc::now();
    //     let active_model = images::ActiveModel {
    //         url: Set(upload_result.url.clone()),
    //         key: Set(upload_result.key.clone()),
    //         filename: Set(upload_result.filename.clone()),
    //         size: Set(upload_result.size),
    //         human_readable_size: Set(upload_result.human_readable_size.clone()),
    //         created_at: Set(now),
    //         updated_at: Set(now),
    //         ..Default::default()
    //     };

    //     let result = active_model
    //         .insert(&self.db)
    //         .await
    //         .map_err(|e| AppError::InternalServerError(format!("保存图片信息失败: {}", e)))?;

    //     Ok(result)
    // }
}
