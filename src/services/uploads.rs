//! 上传管理服务

use crate::config::AppError;
use crate::models::uploads;
use crate::services::qiniu;
use chrono::Utc;
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder,
};
use uuid::Uuid;

pub struct UploadsService;

impl UploadsService {
    /// 保存上传记录到 DB
    pub async fn save(
        db: &DatabaseConnection,
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
        let limit = limit.min(100);
        let paginator = uploads::Entity::find()
            .order_by_desc(uploads::Column::CreatedAt)
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
        let record = uploads::Entity::find()
            .filter(uploads::Column::Uuid.eq(uuid))
            .one(db)
            .await
            .map_err(|e| AppError::DatabaseError(format!("查询失败: {e}")))?
            .ok_or_else(|| AppError::NotFound("上传记录不存在".to_string()))?;

        // 尝试删除七牛文件（失败仅 warn，不阻塞 DB 删除）
        if let Err(e) = qiniu::delete_from_qiniu(&record.key) {
            log::warn!("七牛删除失败(忽略): {} - {}", record.key, e);
        }

        uploads::Entity::delete_by_id(record.id)
            .exec(db)
            .await
            .map_err(|e| AppError::DatabaseError(format!("删除失败: {e}")))?;

        Ok(())
    }
}
