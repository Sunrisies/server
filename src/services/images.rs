//! 图片管理服务 — 上传委托 qiniu 模块，列表/详情/删除为存根

use crate::config::AppError;
use crate::dto::common::PaginationQuery;
use crate::dto::image::ImageUploadResponse;
use crate::models::images;
use crate::services::qiniu;
use crate::utils::file_size::FileSize;
use crate::{ApiResponse, HttpResult};
use actix_multipart::Multipart;
use chrono::Utc;
use futures_util::StreamExt;
use log::info;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, DatabaseConnection};

pub struct ImageService;

impl ImageService {
    /// 上传图片（委托 qiniu 模块，保存记录到 DB）
    pub async fn handle_upload(db: &DatabaseConnection, mut payload: Multipart) -> HttpResult {
        while let Some(item) = payload.next().await {
            let mut field = item.map_err(|e| AppError::MultipartError(e.to_string()))?;
            let field_name = field
                .content_disposition()
                .and_then(|cd| cd.get_name().map(|s| s.to_string()))
                .unwrap_or_default();

            if field_name == "file" {
                let result = qiniu::upload_file(&mut field, "images", true).await?;

                info!("上传成功: {} ({} bytes)", result.url, result.size);

                let image_record = Self::save_image_to_db(db, &result).await?;

                // 同步记录到上传管理表
                let _ = crate::services::uploads::UploadsService::save(db, &result).await;

                let response = ImageUploadResponse {
                    id: image_record.id,
                    url: result.url,
                    key: result.key,
                    filename: result.filename.clone(),
                    size: result.size,
                    human_readable_size: FileSize::from(result.size).to_string(),
                    created_at: image_record.created_at,
                };

                return Ok(ApiResponse::success(response, "图片上传成功").to_http_response());
            }
        }
        Err(AppError::BadRequest(
            "未找到文件字段 (field name: file)".to_string(),
        ))
    }

    /// 获取图片列表（存根）
    pub async fn get_images(_db: &DatabaseConnection, _query: PaginationQuery) -> HttpResult {
        Ok(ApiResponse::success("response", "获取图片列表成功").to_http_response())
    }

    /// 获取图片详情（存根）
    pub async fn get_image_by_id(_db: &DatabaseConnection, _image_id: i32) -> HttpResult {
        Ok(ApiResponse::success("image", "获取图片成功").to_http_response())
    }

    /// 删除图片（存根）
    pub async fn delete_image(_db: &DatabaseConnection, _image_id: i32) -> HttpResult {
        Ok(ApiResponse::success((), "图片删除成功").to_http_response())
    }

    /// 保存图片信息到数据库
    async fn save_image_to_db(
        db: &DatabaseConnection,
        result: &qiniu::UploadResult,
    ) -> Result<images::Model, AppError> {
        let now = Utc::now();
        let human_readable = FileSize::from(result.size).to_string();
        let active_model = images::ActiveModel {
            url: Set(result.url.clone()),
            key: Set(result.key.clone()),
            filename: Set(result.filename.clone()),
            size: Set(result.size),
            human_readable_size: Set(human_readable),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };
        active_model
            .insert(db)
            .await
            .map_err(|e| AppError::InternalServerError(format!("保存图片信息失败: {e}")))
    }
}
