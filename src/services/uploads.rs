//! 上传管理服务

use crate::config::AppError;
use crate::models::uploads;
use crate::services::qiniu;
use chrono::Utc;
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, ConnectionTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryOrder,
};
use uuid::Uuid;

pub struct UploadsService;

impl UploadsService {
    /// 保存上传记录到 DB
    pub async fn save<C: ConnectionTrait>(
        db: &C,
        result: &qiniu::UploadResult,
    ) -> Result<uploads::Model, AppError> {
        let now = Utc::now();
        let model = uploads::ActiveModel {
            uuid: Set(Uuid::new_v4().to_string()),
            url: Set(result.url.clone()),
            key: Set(result.key.clone()),
            filename: Set(result.filename.clone()),
            file_size: Set(result.size),
            mime_type: Set(result.mime_type.clone()),
            created_at: Set(now),
            ..Default::default()
        };
        model
            .insert(db)
            .await
            .map_err(|e| AppError::DatabaseError(format!("保存上传记录失败: {e}")))
    }

    /// 获取上传列表（分页）
    pub async fn list(
        db: &DatabaseConnection,
        page: u64,
        limit: u64,
    ) -> Result<(Vec<uploads::Model>, u64), AppError> {
        let valid_offset = page
            .checked_sub(1)
            .and_then(|page| page.checked_mul(limit))
            .is_some_and(|offset| offset <= i64::MAX as u64);
        if !valid_offset || !(1..=100).contains(&limit) {
            return Err(AppError::BadRequest(
                "页码必须大于0，每页数量必须为1到100".into(),
            ));
        }
        let paginator = uploads::Entity::find()
            .order_by_desc(uploads::Column::CreatedAt)
            .order_by_desc(uploads::Column::Id)
            .paginate(db, limit);

        let total = paginator
            .num_items()
            .await
            .map_err(|e| AppError::DatabaseError(format!("查询总数失败: {e}")))?;
        let data = paginator
            .fetch_page(page.saturating_sub(1))
            .await
            .map_err(|e| AppError::DatabaseError(format!("查询列表失败: {e}")))?;

        Ok((data, total))
    }

    /// 删除上传记录（同步删除七牛文件）
    pub async fn delete(db: &DatabaseConnection, uuid: &str) -> Result<(), AppError> {
        let image_id = crate::services::images::ImageService::import_upload(db, uuid).await?;
        crate::services::images::ImageService::delete_image(db, image_id).await?;
        Ok(())
    }
}
