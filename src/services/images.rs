//! 图片上传、分页查询和服务端引用保护。

use crate::config::AppError;
use crate::dto::common::{PaginatedResp, Pagination};
use crate::dto::image::ImageUploadResponse;
use crate::dto::image::{ImageQuery, ImageReference, ManagedImage};
use crate::models::{images, posts, uploads};
use crate::services::qiniu;
use crate::utils::file_size::FileSize;
use crate::{ApiResponse, HttpResult};
use actix_multipart::Multipart;
use chrono::Utc;
use futures_util::StreamExt;
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, DatabaseConnection,
    DatabaseTransaction, DbBackend, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    Statement, TransactionTrait,
};
use validator::Validate;

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
                let (result, image_record) =
                    Self::upload_and_store(db.clone(), &mut field, "images").await?;
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

    pub async fn upload_and_store(
        db: DatabaseConnection,
        field: &mut actix_multipart::Field,
        prefix: &str,
    ) -> Result<(qiniu::UploadResult, images::Model), AppError> {
        if std::env::var("IMAGE_STORAGE_PROVIDER").unwrap_or_else(|_| "qiniu".into()) == "local" {
            let result = crate::services::local_images::upload(field).await?;
            let task = actix_web::rt::spawn(async move {
                let id = match crate::services::upload_jobs::enqueue(&db, result.key.clone()).await
                {
                    Ok(id) => id,
                    Err(error) => {
                        let _ = crate::services::local_images::remove(&result.key).await;
                        return Err(error);
                    }
                };
                Self::confirm_or_cleanup(&db, &id, result).await
            });
            return task
                .await
                .map_err(|_| AppError::UploadFailed("保存上传记录失败".into()))?;
        }
        if std::env::var("IMAGE_STORAGE_PROVIDER").unwrap_or_else(|_| "qiniu".into()) != "qiniu" {
            return Err(AppError::ConfigurationError(
                "Unknown IMAGE_STORAGE_PROVIDER".into(),
            ));
        }
        let settings = &crate::config::manager::CONFIG.qi_niu;
        if settings.access_key.is_empty()
            || settings.access_key.starts_with("your")
            || settings.secret_key.is_empty()
            || settings.secret_key.starts_with("your")
            || settings.bucket_name.is_empty()
            || settings.bucket_name.starts_with("your")
        {
            return Err(AppError::MaintenanceMode(
                "图片存储尚未配置，请联系管理员配置后重试".into(),
            ));
        }
        let prepared = qiniu::prepare_upload(field, prefix, true).await?;
        // Persist the intended object key before any remote upload can take place.
        let id = crate::services::upload_jobs::enqueue(&db, prepared.key.clone()).await?;
        Self::finish_detached(db, id, prepared.run()).await
    }

    pub(crate) async fn finish_detached<F>(
        db: DatabaseConnection,
        id: String,
        upload: F,
    ) -> Result<(qiniu::UploadResult, images::Model), AppError>
    where
        F: std::future::Future<Output = Result<qiniu::UploadResult, AppError>> + 'static,
    {
        let task = actix_web::rt::spawn(async move {
            let heartbeat = crate::services::upload_jobs::heartbeat(db.clone(), id.clone());
            let outcome = match upload.await {
                Ok(result) => Self::confirm_or_cleanup(&db, &id, result).await,
                Err(error) => {
                    let _ = crate::services::upload_jobs::cleanup(&db, &id).await;
                    Err(error)
                }
            };
            heartbeat.abort();
            outcome
        });
        task.await
            .map_err(|_| AppError::UploadFailed("上传确认任务失败，已保留恢复记录".into()))?
    }

    async fn confirm_or_cleanup(
        db: &DatabaseConnection,
        id: &str,
        result: qiniu::UploadResult,
    ) -> Result<(qiniu::UploadResult, images::Model), AppError> {
        // Idempotent retry also handles an ambiguous transaction commit response.
        for _ in 0..2 {
            if let Ok(image) = Self::record_upload(db, &result, Some(id)).await {
                return Ok((result, image));
            }
        }
        if crate::services::upload_jobs::cleanup(db, id).await.is_err() {
            log::warn!("Upload confirmation failed; durable cleanup retained");
        }
        Err(AppError::StorageServiceError(
            "上传记录保存失败，已安排清理或恢复".into(),
        ))
    }

    pub(crate) async fn record_upload(
        db: &DatabaseConnection,
        result: &qiniu::UploadResult,
        job_id: Option<&str>,
    ) -> Result<images::Model, AppError> {
        let txn = db.begin().await?;
        media_lock(&txn).await?;
        let existing = images::Entity::find()
            .filter(images::Column::Key.eq(&result.key))
            .one(&txn)
            .await?;
        let uploaded = uploads::Entity::find()
            .filter(uploads::Column::Key.eq(&result.key))
            .one(&txn)
            .await?;
        if let Some(id) = job_id {
            if !(existing.is_some() && uploaded.is_some()) {
                let job = crate::models::media_upload_jobs::Entity::find_by_id(id)
                    .one(&txn)
                    .await?
                    .ok_or_else(|| AppError::Conflict("上传确认任务已结束，请重新上传".into()))?;
                if job.state != "pending" {
                    return Err(AppError::Conflict(
                        "该上传任务已进入清理，请重新上传".into(),
                    ));
                }
            }
        }
        let image = match existing {
            Some(image) => image,
            None => Self::save_image_to_db(&txn, result).await?,
        };
        if uploaded.is_none() {
            crate::services::uploads::UploadsService::save(&txn, result).await?;
        }
        if let Some(id) = job_id {
            crate::models::media_upload_jobs::Entity::delete_by_id(id)
                .exec(&txn)
                .await?;
        }
        txn.commit().await?;
        Ok(image)
    }

    pub async fn backfill_legacy(db: &DatabaseConnection) -> Result<(), AppError> {
        let txn = db.begin().await?;
        media_lock(&txn).await?;
        txn.execute_unprepared("INSERT INTO images(url,key,filename,size,human_readable_size,created_at,updated_at) SELECT u.url,u.key,u.filename,u.file_size,CAST(u.file_size AS TEXT)||' B',u.created_at,u.created_at FROM uploads u WHERE u.mime_type LIKE 'image/%' AND NOT EXISTS(SELECT 1 FROM images i WHERE i.key=u.key OR i.key='deleted/'||u.key)").await?;
        txn.commit().await?;
        Ok(())
    }

    pub async fn import_upload(db: &DatabaseConnection, uuid: &str) -> Result<i32, AppError> {
        let txn = db.begin().await?;
        media_lock(&txn).await?;
        let upload = uploads::Entity::find()
            .filter(uploads::Column::Uuid.eq(uuid))
            .one(&txn)
            .await?
            .ok_or_else(|| AppError::NotFound("上传记录不存在".into()))?;
        let image = match images::Entity::find()
            .filter(images::Column::Key.eq(&upload.key))
            .one(&txn)
            .await?
        {
            Some(image) => image,
            None => {
                Self::save_image_to_db(
                    &txn,
                    &qiniu::UploadResult {
                        url: upload.url,
                        key: upload.key,
                        filename: upload.filename,
                        size: upload.file_size,
                        human_readable_size: String::new(),
                        mime_type: upload.mime_type,
                    },
                )
                .await?
            }
        };
        txn.commit().await?;
        Ok(image.id)
    }

    /// 分页查询真实图片和引用。按页批量查询文章，避免逐张图片查询。
    pub async fn get_images(db: &DatabaseConnection, query: ImageQuery) -> HttpResult {
        query
            .validate()
            .map_err(|_| AppError::BadRequest("分页或搜索参数无效".into()))?;
        let page = query.page;
        let limit = query.limit;
        let mut select = images::Entity::find().filter(images::Column::Key.not_like("deleted/%"));
        if let Some(search) = query.search.filter(|s| !s.trim().is_empty()) {
            let search = search
                .trim()
                .replace('\\', "\\\\")
                .replace('%', "\\%")
                .replace('_', "\\_");
            select = select.filter(
                sea_orm::prelude::Expr::expr(sea_orm::sea_query::Func::lower(
                    sea_orm::prelude::Expr::col(images::Column::Filename),
                ))
                .like(
                    sea_orm::sea_query::LikeExpr::new(format!("%{}%", search.to_lowercase()))
                        .escape('\\'),
                ),
            );
        }
        let paginator = select.order_by_desc(images::Column::Id).paginate(db, limit);
        let total = paginator.num_items().await?;
        let images = paginator.fetch_page(page - 1).await?;
        let candidates = reference_candidates(db, &images).await?;
        let data: Vec<_> = images
            .into_iter()
            .map(|image| with_references(image, &candidates))
            .collect();
        Ok(ApiResponse::success(
            PaginatedResp {
                data,
                pagination: Pagination { page, limit, total },
            },
            "获取图片列表成功",
        )
        .to_http_response())
    }

    pub async fn get_image_by_id(db: &DatabaseConnection, image_id: i32) -> HttpResult {
        let image = active_image(db, image_id).await?;
        let candidates = reference_candidates(db, std::slice::from_ref(&image)).await?;
        Ok(
            ApiResponse::success(with_references(image, &candidates), "获取图片成功")
                .to_http_response(),
        )
    }

    /// 所有文章写入和图片删除共享事务锁，防止引用检查和并发保存之间出现竞态。
    pub async fn delete_image(db: &DatabaseConnection, image_id: i32) -> HttpResult {
        Self::delete_image_with(db, image_id, |key| async move {
            if key.starts_with("local/") {
                crate::services::local_images::remove(&key).await
            } else {
                actix_web::web::block(move || qiniu::delete_from_qiniu(&key))
                    .await
                    .map_err(|_| AppError::StorageServiceError("删除任务失败".into()))?
                    .map_err(|_| {
                        AppError::StorageServiceError("云端图片删除失败，记录已保留，请重试".into())
                    })
            }
        })
        .await
    }

    pub(crate) async fn delete_image_with<F, Fut>(
        db: &DatabaseConnection,
        image_id: i32,
        remove: F,
    ) -> HttpResult
    where
        F: FnOnce(String) -> Fut,
        Fut: std::future::Future<Output = Result<(), AppError>>,
    {
        let txn = db.begin().await?;
        media_lock(&txn).await?;
        let image = active_image(&txn, image_id).await?;
        let candidates = reference_candidates(&txn, std::slice::from_ref(&image)).await?;
        let managed = with_references(image.clone(), &candidates);
        if managed.reference_count > 0 {
            return Err(AppError::Conflict(format!(
                "图片被 {} 篇文章引用，请先移除正文和封面引用后重试",
                managed.reference_count
            )));
        }
        remove(image.key.clone()).await?;
        uploads::Entity::delete_many()
            .filter(uploads::Column::Key.eq(&image.key))
            .exec(&txn)
            .await?;
        // 保留删除标记，用于拒绝旧编辑器再次提交已删除的图片地址。
        // 不改表结构；公开列表和详情永远排除 deleted/ 记录。
        images::Entity::update_many()
            .filter(images::Column::Key.eq(&image.key))
            .col_expr(
                images::Column::Key,
                sea_orm::prelude::Expr::value(format!("deleted/{}", image.key)),
            )
            .exec(&txn)
            .await?;
        txn.commit().await?;
        Ok(ApiResponse::success((), "图片已删除").to_http_response())
    }

    /// 保存图片信息到数据库
    async fn save_image_to_db<C: ConnectionTrait>(
        db: &C,
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

async fn active_image<C: ConnectionTrait>(db: &C, id: i32) -> Result<images::Model, AppError> {
    images::Entity::find_by_id(id)
        .filter(images::Column::Key.not_like("deleted/%"))
        .one(db)
        .await?
        .ok_or_else(|| AppError::NotFound("图片不存在或已删除".into()))
}

pub async fn media_lock(txn: &DatabaseTransaction) -> Result<(), AppError> {
    if txn.get_database_backend() == DbBackend::Postgres {
        txn.execute(Statement::from_string(
            DbBackend::Postgres,
            "SELECT pg_advisory_xact_lock(73021017)".to_owned(),
        ))
        .await?;
    }
    Ok(())
}

// Decode percent escapes so URLs copied with encoded characters are protected too.
fn decoded(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(a), Some(b)) = (
                (bytes[i + 1] as char).to_digit(16),
                (bytes[i + 2] as char).to_digit(16),
            ) {
                out.push((a * 16 + b) as u8);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn refers(value: &str, image: &images::Model) -> bool {
    let value = decoded(value);
    let key = image.key.strip_prefix("deleted/").unwrap_or(&image.key);
    let token = key.strip_prefix("local/").unwrap_or(key);
    let identity = token
        .split('/')
        .nth(1)
        .filter(|part| uuid::Uuid::parse_str(part).is_ok())
        .unwrap_or(token);
    // UUID object identity survives CDN changes, percent encoding and Markdown escaping.
    value.contains(&decoded(&image.url)) || (!identity.is_empty() && value.contains(identity))
}

async fn reference_candidates<C: ConnectionTrait>(
    db: &C,
    images: &[images::Model],
) -> Result<Vec<posts::Model>, AppError> {
    if images.is_empty() {
        return Ok(Vec::new());
    }
    let mut condition = Condition::any();
    for image in images {
        // The UUID prefix stays unchanged even when the filename has been URL-encoded.
        let key = image.key.strip_prefix("local/").unwrap_or(&image.key);
        let token = if image.key.starts_with("local/") {
            key.to_owned()
        } else {
            key.split('/').nth(1).unwrap_or(key).to_owned()
        };
        for column in [
            posts::Column::Content,
            posts::Column::Markdowncontent,
            posts::Column::CoverImage,
        ] {
            condition = condition
                .add(column.contains(&token))
                .add(column.contains(&image.url));
        }
    }
    Ok(posts::Entity::find()
        .filter(condition)
        .order_by_desc(posts::Column::Id)
        .all(db)
        .await?)
}

fn with_references(image: images::Model, posts: &[posts::Model]) -> ManagedImage {
    let references: Vec<_> = posts
        .iter()
        .filter_map(|post| {
            let body = refers(&post.content, &image) || refers(&post.markdowncontent, &image);
            let cover = post
                .cover_image
                .as_deref()
                .is_some_and(|s| refers(s, &image));
            (body || cover).then(|| ImageReference {
                uuid: post.uuid.clone(),
                title: post.title.clone(),
                status: post.status,
                body,
                cover,
            })
        })
        .collect();
    ManagedImage {
        reference_count: references.len(),
        references,
        image,
    }
}

/// Reject stale editor submissions referencing images that were already deleted.
pub async fn validate_media<C: ConnectionTrait>(
    db: &C,
    html: &str,
    markdown: &str,
    cover: Option<&str>,
) -> Result<(), AppError> {
    let deleted = images::Entity::find()
        .filter(images::Column::Key.starts_with("deleted/"))
        .all(db)
        .await?;
    if deleted.iter().any(|image| {
        refers(html, image) || refers(markdown, image) || cover.is_some_and(|s| refers(s, image))
    }) {
        return Err(AppError::Conflict(
            "文章引用了已删除的图片，请重新上传或移除对应图片".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::posts::{CreatePostRequest, UpdatePostRequest};
    use crate::models::{categories, tags, users};
    use crate::services::posts::PostService;

    async fn fixture() -> (DatabaseConnection, images::Model, i32, i32, i32) {
        let db = crate::test_helpers::setup_test_db().await;
        let now = Utc::now();
        let image = images::ActiveModel {
            url: Set(format!("/api/v1/images/file/{}.webp", uuid::Uuid::new_v4())),
            key: Set(String::new()),
            filename: Set("保护测试.png".into()),
            size: Set(100),
            human_readable_size: Set("100 B".into()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();
        let mut active: images::ActiveModel = image.clone().into();
        active.key = Set(format!("local/{}", image.url.rsplit('/').next().unwrap()));
        let image = active.update(&db).await.unwrap();
        let user = users::ActiveModel {
            uuid: Set(uuid::Uuid::new_v4().to_string()),
            user_name: Set("media-test".into()),
            pass_word: Set("test-only".into()),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();
        let category = categories::ActiveModel {
            name: Set("测试分类".into()),
            slug: Set("media-test".into()),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();
        let tag = tags::ActiveModel {
            name: Set("测试标签".into()),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();
        (db, image, user.id, category.id, tag.id)
    }
    fn draft(image: &images::Model, category_id: i32, tag: i32) -> CreatePostRequest {
        CreatePostRequest {
            title: "引用保护".into(),
            summary: None,
            content: String::new(),
            markdowncontent: format!("![图片]({})", image.url),
            cover_image: Some(image.url.clone()),
            category_id,
            tag_ids: vec![tag],
            status: 0,
            featured: false,
        }
    }
    #[tokio::test]
    async fn all_post_states_and_both_sources_are_protected_and_stale_submissions_rejected() {
        let (db, image, user, category, tag) = fixture().await;
        let post = PostService::create_post(&db, user, draft(&image, category, tag))
            .await
            .unwrap();
        for status in [0, 1, 2] {
            let update: UpdatePostRequest =
                serde_json::from_value(serde_json::json!({"status":status})).unwrap();
            PostService::update_post(&db, user, &post.uuid, update)
                .await
                .unwrap();
            let refs = reference_candidates(&db, std::slice::from_ref(&image))
                .await
                .unwrap();
            let detail = with_references(image.clone(), &refs);
            assert_eq!(detail.reference_count, 1);
            assert!(detail.references[0].body && detail.references[0].cover);
            assert!(matches!(
                ImageService::delete_image(&db, image.id).await,
                Err(AppError::Conflict(_))
            ));
        }
        let upload = crate::services::uploads::UploadsService::save(
            &db,
            &qiniu::UploadResult {
                url: image.url.clone(),
                key: image.key.clone(),
                filename: image.filename.clone(),
                size: 100,
                human_readable_size: "100 B".into(),
                mime_type: "image/webp".into(),
            },
        )
        .await
        .unwrap();
        assert!(matches!(
            crate::services::uploads::UploadsService::delete(&db, &upload.uuid).await,
            Err(AppError::Conflict(_))
        ));
        // Legacy HTML-only reference remains protected even after Markdown and cover were removed.
        let update: UpdatePostRequest = serde_json::from_value(serde_json::json!({"markdowncontent":"正文", "content":format!("<img src=\"{}\">",image.url),"cover_image":""})).unwrap();
        PostService::update_post(&db, user, &post.uuid, update)
            .await
            .unwrap();
        assert!(matches!(
            ImageService::delete_image(&db, image.id).await,
            Err(AppError::Conflict(_))
        ));
        let update: UpdatePostRequest = serde_json::from_value(
            serde_json::json!({"content":"", "markdowncontent":"已移除图片", "cover_image":""}),
        )
        .unwrap();
        PostService::update_post(&db, user, &post.uuid, update)
            .await
            .unwrap();
        ImageService::delete_image(&db, image.id).await.unwrap();
        assert!(active_image(&db, image.id).await.is_err());
        assert!(
            uploads::Entity::find_by_id(upload.id)
                .one(&db)
                .await
                .unwrap()
                .is_none()
        );
        assert!(matches!(
            PostService::create_post(&db, user, draft(&image, category, tag)).await,
            Err(AppError::Conflict(_))
        ));
        let update: UpdatePostRequest = serde_json::from_value(
            serde_json::json!({"markdowncontent":format!("![old]({})",image.url)}),
        )
        .unwrap();
        assert!(matches!(
            PostService::update_post(&db, user, &post.uuid, update).await,
            Err(AppError::Conflict(_))
        ));
    }
    #[tokio::test]
    async fn deleting_waits_for_inflight_article_save_and_rechecks_references() {
        let (db, image, user, category, tag) = fixture().await;
        let txn = db.begin().await.unwrap();
        media_lock(&txn).await.unwrap();
        let db2 = db.clone();
        let id = image.id;
        let mut deletion = tokio::spawn(async move {
            ImageService::delete_image(&db2, id)
                .await
                .map(|_| ())
                .map_err(|e| e.code())
        });
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(100), &mut deletion)
                .await
                .is_err()
        );
        let now = Utc::now();
        posts::ActiveModel {
            uuid: Set(uuid::Uuid::new_v4().to_string()),
            author_id: Set(user),
            category_id: Set(category),
            title: Set("并发保存".into()),
            summary: Set(None),
            content: Set(String::new()),
            markdowncontent: Set(image.url.clone()),
            cover_image: Set(None),
            status: Set(0),
            featured: Set(false),
            view_count: Set(0),
            created_at: Set(now),
            updated_at: Set(now),
            published_at: Set(now),
            size: Set(1),
            ..Default::default()
        }
        .insert(&txn)
        .await
        .unwrap();
        let _ = tag;
        txn.commit().await.unwrap();
        assert_eq!(deletion.await.unwrap(), Err(409));
    }
    #[test]
    fn encoded_urls_and_changed_cdn_hosts_keep_same_object_reference() {
        let image = images::Model {
            id: 1,
            url: "https://old.example/images/uuid/图 文.png".into(),
            key: "images/uuid/图 文.png".into(),
            filename: "图 文.png".into(),
            size: 1,
            human_readable_size: "1 B".into(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        assert!(refers(
            "https://new.example/images/uuid/%E5%9B%BE%20%E6%96%87.png?imageView2/2/w/300",
            &image
        ));
        assert!(!refers("https://new.example/images/other/file.png", &image));
    }
}
