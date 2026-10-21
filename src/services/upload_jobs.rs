//! Durable upload intents and idempotent cleanup. No credentials are stored in jobs.
use crate::{
    config::AppError,
    models::{images, media_upload_jobs as jobs, uploads},
    services::{images::media_lock, qiniu},
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, DatabaseConnection, EntityTrait,
    QueryFilter, QueryOrder, QuerySelect, Set, TransactionTrait,
};

pub async fn initialize(db: &DatabaseConnection) -> Result<(), AppError> {
    let schema = sea_orm::Schema::new(db.get_database_backend());
    let mut table = schema.create_table_from_entity(jobs::Entity);
    table.if_not_exists();
    db.execute(db.get_database_backend().build(&table)).await?;
    Ok(())
}

pub async fn enqueue(db: &DatabaseConnection, key: String) -> Result<String, AppError> {
    let now = Utc::now();
    let id = uuid::Uuid::new_v4().to_string();
    jobs::ActiveModel {
        id: Set(id.clone()),
        key: Set(key),
        state: Set("pending".into()),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(db)
    .await?;
    Ok(id)
}

pub fn heartbeat(db: DatabaseConnection, id: String) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(15)).await;
            let result = jobs::Entity::update_many()
                .filter(jobs::Column::Id.eq(&id))
                .filter(jobs::Column::State.eq("pending"))
                .col_expr(
                    jobs::Column::UpdatedAt,
                    sea_orm::prelude::Expr::value(Utc::now()),
                )
                .exec(&db)
                .await;
            if result.is_err() {
                log::warn!("Upload intent heartbeat failed; durable intent retained");
            }
        }
    })
}

/// Cleanup is forbidden whenever a committed metadata record exists.
pub async fn cleanup(db: &DatabaseConnection, id: &str) -> Result<(), AppError> {
    cleanup_with(db, id, |key| async move {
        if key.starts_with("local/") {
            crate::services::local_images::remove(&key).await
        } else {
            actix_web::web::block(move || qiniu::delete_from_qiniu(&key))
                .await
                .map_err(|_| AppError::StorageServiceError("图片清理任务失败".into()))?
                .map_err(|_| AppError::StorageServiceError("云端清理失败，已保留重试任务".into()))
        }
    })
    .await
}

pub(crate) async fn cleanup_with<F, Fut>(
    db: &DatabaseConnection,
    id: &str,
    remove: F,
) -> Result<(), AppError>
where
    F: FnOnce(String) -> Fut,
    Fut: std::future::Future<Output = Result<(), AppError>>,
{
    let txn = db.begin().await?;
    media_lock(&txn).await?;
    let Some(job) = jobs::Entity::find_by_id(id).one(&txn).await? else {
        return Ok(());
    };
    let saved = images::Entity::find()
        .filter(images::Column::Key.eq(&job.key))
        .one(&txn)
        .await?
        .is_some()
        || uploads::Entity::find()
            .filter(uploads::Column::Key.eq(&job.key))
            .one(&txn)
            .await?
            .is_some();
    if saved {
        jobs::Entity::delete_by_id(id).exec(&txn).await?;
        txn.commit().await?;
        return Ok(());
    }
    // Final metadata writes check this state under the same media lock.
    jobs::Entity::update_many()
        .filter(jobs::Column::Id.eq(id))
        .col_expr(
            jobs::Column::State,
            sea_orm::prelude::Expr::value("cleanup"),
        )
        .col_expr(
            jobs::Column::UpdatedAt,
            sea_orm::prelude::Expr::value(Utc::now()),
        )
        .exec(&txn)
        .await?;
    txn.commit().await?;
    remove(job.key).await?;
    // Retain a short-lived receipt: a very late upload completion can request cleanup again.
    jobs::Entity::update_many()
        .filter(jobs::Column::Id.eq(id))
        .col_expr(
            jobs::Column::State,
            sea_orm::prelude::Expr::value("cleaned"),
        )
        .col_expr(
            jobs::Column::UpdatedAt,
            sea_orm::prelude::Expr::value(Utc::now()),
        )
        .exec(db)
        .await?;
    Ok(())
}

async fn recovery_candidates(db: &DatabaseConnection) -> Result<Vec<jobs::Model>, AppError> {
    let old = Utc::now() - chrono::Duration::hours(1);
    let pending = jobs::Entity::find()
        .filter(
            Condition::any().add(jobs::Column::State.eq("cleanup")).add(
                Condition::all()
                    .add(jobs::Column::State.eq("pending"))
                    .add(jobs::Column::UpdatedAt.lt(old)),
            ),
        )
        .order_by_asc(jobs::Column::UpdatedAt)
        .limit(20)
        .all(db)
        .await?;
    Ok(pending)
}

pub async fn recover_once(db: &DatabaseConnection) -> Result<(), AppError> {
    let pending = recovery_candidates(db).await?;
    for job in pending {
        if cleanup(db, &job.id).await.is_err() {
            log::warn!("Upload cleanup will be retried");
        }
    }
    jobs::Entity::delete_many()
        .filter(jobs::Column::State.eq("cleaned"))
        .filter(jobs::Column::UpdatedAt.lt(Utc::now() - chrono::Duration::days(7)))
        .exec(db)
        .await?;
    Ok(())
}

pub fn start_recovery(db: DatabaseConnection) {
    actix_web::rt::spawn(async move {
        loop {
            if recover_once(&db).await.is_err() {
                log::warn!("Upload recovery unavailable; retrying later");
            }
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::images::ImageService;
    use sea_orm::PaginatorTrait;

    fn result() -> qiniu::UploadResult {
        let key = format!("images/{}/test.png", uuid::Uuid::new_v4());
        qiniu::UploadResult {
            url: format!("https://example.invalid/{key}"),
            key,
            filename: "test.png".into(),
            size: 100,
            human_readable_size: "100 B".into(),
            mime_type: "image/png".into(),
        }
    }
    #[tokio::test]
    async fn invalid_upload_pagination_returns_error_without_panic() {
        let db = crate::test_helpers::setup_test_db().await;
        for (page, limit) in [(0, 20), (1, 0), (1, 101), (0, 0), (u64::MAX, 100)] {
            assert!(matches!(
                crate::services::uploads::UploadsService::list(&db, page, limit).await,
                Err(AppError::BadRequest(_))
            ));
        }
        assert!(
            crate::services::uploads::UploadsService::list(&db, 1, 100)
                .await
                .is_ok()
        );
    }
    #[tokio::test]
    async fn upload_metadata_is_atomic_and_confirmation_is_idempotent() {
        let db = crate::test_helpers::setup_test_db().await;
        let upload = result();
        let id = enqueue(&db, upload.key.clone()).await.unwrap();
        // Inject a real database constraint failure on the second metadata insert.
        db.execute_unprepared(
            "ALTER TABLE uploads ADD CONSTRAINT reject_fixture CHECK (file_size < 0)",
        )
        .await
        .unwrap();
        assert!(
            ImageService::record_upload(&db, &upload, Some(&id))
                .await
                .is_err()
        );
        assert_eq!(images::Entity::find().count(&db).await.unwrap(), 0);
        assert_eq!(uploads::Entity::find().count(&db).await.unwrap(), 0);
        assert!(
            jobs::Entity::find_by_id(&id)
                .one(&db)
                .await
                .unwrap()
                .is_some()
        );
        db.execute_unprepared("ALTER TABLE uploads DROP CONSTRAINT reject_fixture")
            .await
            .unwrap();
        let first = ImageService::record_upload(&db, &upload, Some(&id))
            .await
            .unwrap();
        let retry = ImageService::record_upload(&db, &upload, Some(&id))
            .await
            .unwrap();
        assert_eq!(first.id, retry.id);
        assert_eq!(images::Entity::find().count(&db).await.unwrap(), 1);
        assert_eq!(uploads::Entity::find().count(&db).await.unwrap(), 1);
        assert!(
            jobs::Entity::find_by_id(id)
                .one(&db)
                .await
                .unwrap()
                .is_none()
        );
    }
    #[tokio::test]
    async fn cleanup_failure_is_durable_and_retry_is_safe() {
        let db = crate::test_helpers::setup_test_db().await;
        let upload = result();
        let id = enqueue(&db, upload.key.clone()).await.unwrap();
        assert!(
            cleanup_with(&db, &id, |_| async {
                Err(AppError::StorageServiceError(
                    "injected storage failure".into(),
                ))
            })
            .await
            .is_err()
        );
        assert_eq!(
            jobs::Entity::find_by_id(&id)
                .one(&db)
                .await
                .unwrap()
                .unwrap()
                .state,
            "cleanup"
        );
        // Once cleanup started, a late completion cannot publish a deleted image.
        assert!(
            ImageService::record_upload(&db, &upload, Some(&id))
                .await
                .is_err()
        );
        cleanup_with(&db, &id, |_| async { Ok(()) }).await.unwrap();
        assert_eq!(
            jobs::Entity::find_by_id(&id)
                .one(&db)
                .await
                .unwrap()
                .unwrap()
                .state,
            "cleaned"
        );
        cleanup_with(&db, &id, |_| async { Ok(()) }).await.unwrap();
    }
    #[tokio::test]
    async fn cleanup_never_deletes_committed_media_on_ambiguous_commit() {
        let db = crate::test_helpers::setup_test_db().await;
        let upload = result();
        ImageService::record_upload(&db, &upload, None)
            .await
            .unwrap();
        let id = enqueue(&db, upload.key.clone()).await.unwrap();
        cleanup_with(&db, &id, |_| async {
            panic!("must not delete a committed media object")
        })
        .await
        .unwrap();
        assert!(
            jobs::Entity::find_by_id(id)
                .one(&db)
                .await
                .unwrap()
                .is_none()
        );
        assert_eq!(images::Entity::find().count(&db).await.unwrap(), 1);
    }
    #[tokio::test]
    async fn legacy_images_are_backfilled_and_cloud_delete_failures_preserve_records() {
        let db = crate::test_helpers::setup_test_db().await;
        let upload = result();
        let saved = crate::services::uploads::UploadsService::save(&db, &upload)
            .await
            .unwrap();
        ImageService::backfill_legacy(&db).await.unwrap();
        ImageService::backfill_legacy(&db).await.unwrap();
        assert_eq!(images::Entity::find().count(&db).await.unwrap(), 1);
        let id = ImageService::import_upload(&db, &saved.uuid).await.unwrap();
        assert!(
            ImageService::delete_image_with(&db, id, |_| async {
                Err(AppError::StorageServiceError("injected".into()))
            })
            .await
            .is_err()
        );
        assert!(
            images::Entity::find_by_id(id)
                .one(&db)
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            uploads::Entity::find_by_id(saved.id)
                .one(&db)
                .await
                .unwrap()
                .is_some()
        );
    }

    #[actix_web::test]
    async fn request_cancellation_does_not_cancel_upload_confirmation() {
        let db = crate::test_helpers::setup_test_db().await;
        let upload = result();
        let key = upload.key.clone();
        let id = enqueue(&db, key.clone()).await.unwrap();
        let (started, ready) = tokio::sync::oneshot::channel();
        let (release, wait) = tokio::sync::oneshot::channel();
        let caller = actix_web::rt::spawn(ImageService::finish_detached(
            db.clone(),
            id.clone(),
            async move {
                started.send(()).unwrap();
                wait.await.unwrap();
                Ok(upload)
            },
        ));
        ready.await.unwrap();
        caller.abort();
        let _ = caller.await;
        release.send(()).unwrap();
        for _ in 0..100 {
            if uploads::Entity::find()
                .filter(uploads::Column::Key.eq(&key))
                .one(&db)
                .await
                .unwrap()
                .is_some()
            {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        assert!(
            images::Entity::find()
                .filter(images::Column::Key.eq(&key))
                .one(&db)
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            uploads::Entity::find()
                .filter(uploads::Column::Key.eq(&key))
                .one(&db)
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            jobs::Entity::find_by_id(id)
                .one(&db)
                .await
                .unwrap()
                .is_none()
        );
    }
    #[tokio::test]
    async fn recovery_skips_live_leases_and_selects_stale_or_failed_jobs() {
        let db = crate::test_helpers::setup_test_db().await;
        let live = enqueue(&db, result().key).await.unwrap();
        let stale = enqueue(&db, result().key).await.unwrap();
        let failed = enqueue(&db, result().key).await.unwrap();
        jobs::Entity::update_many()
            .filter(jobs::Column::Id.eq(&stale))
            .col_expr(
                jobs::Column::UpdatedAt,
                sea_orm::prelude::Expr::value(Utc::now() - chrono::Duration::hours(2)),
            )
            .exec(&db)
            .await
            .unwrap();
        jobs::Entity::update_many()
            .filter(jobs::Column::Id.eq(&failed))
            .col_expr(
                jobs::Column::State,
                sea_orm::prelude::Expr::value("cleanup"),
            )
            .exec(&db)
            .await
            .unwrap();
        let candidates = recovery_candidates(&db).await.unwrap();
        assert_eq!(candidates.len(), 2);
        assert!(!candidates.iter().any(|job| job.id == live));
    }
}
