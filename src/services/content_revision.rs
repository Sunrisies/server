//! Durable cache version, updated transactionally by database triggers.
use crate::config::AppError;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement};

pub async fn initialize(db: &DatabaseConnection) -> Result<(), AppError> {
    db.execute(Statement::from_string(db.get_database_backend(), "CREATE TABLE IF NOT EXISTS public_content_revision (id INTEGER PRIMARY KEY, epoch VARCHAR(36) NOT NULL, version BIGINT NOT NULL DEFAULT 0)".to_owned())).await?;
    db.execute(Statement::from_sql_and_values(db.get_database_backend(), "INSERT INTO public_content_revision(id,epoch,version) VALUES(1,$1,0) ON CONFLICT(id) DO NOTHING", [uuid::Uuid::new_v4().to_string().into()])).await?;
    if db.get_database_backend() == DbBackend::Postgres {
        db.execute_unprepared(include_str!("../sql/content_revision.sql"))
            .await?;
    }
    Ok(())
}

pub async fn version(db: &DatabaseConnection) -> Result<String, AppError> {
    let row = db
        .query_one(Statement::from_string(
            db.get_database_backend(),
            "SELECT epoch,version FROM public_content_revision WHERE id=1".to_owned(),
        ))
        .await?
        .ok_or_else(|| {
            AppError::ConfigurationError("Content revision is not initialized".into())
        })?;
    Ok(format!(
        "{}:{}",
        row.try_get::<String>("", "epoch")?,
        row.try_get::<i64>("", "version")?
    ))
}

pub async fn handler(db: actix_web::web::Data<DatabaseConnection>) -> crate::HttpResult {
    Ok(actix_web::HttpResponse::Ok()
        .insert_header(("Cache-Control", "no-store"))
        .json(crate::ApiResponse::success(
            version(&db).await?,
            "获取内容版本成功",
        )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{categories, posts, tags, users};
    use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
    #[tokio::test]
    async fn committed_changes_update_revision_but_rollback_and_views_do_not() {
        let db = crate::test_helpers::setup_test_db().await;
        let start = version(&db).await.unwrap();
        let user = users::ActiveModel {
            uuid: Set(uuid::Uuid::new_v4().to_string()),
            user_name: Set("revision-test".into()),
            pass_word: Set("test".into()),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();
        let cat = categories::ActiveModel {
            name: Set("revision".into()),
            slug: Set("revision".into()),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();
        assert_ne!(version(&db).await.unwrap(), start);
        let tag = tags::ActiveModel {
            name: Set("revision".into()),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();
        let post = crate::services::posts::PostService::create_post(
            &db,
            user.id,
            crate::dto::posts::CreatePostRequest {
                title: "revision".into(),
                summary: None,
                content: String::new(),
                markdowncontent: "body".into(),
                cover_image: None,
                category_id: cat.id,
                tag_ids: vec![tag.id],
                status: 1,
                featured: false,
            },
        )
        .await
        .unwrap();
        let before = version(&db).await.unwrap();
        let txn = db.begin().await.unwrap();
        posts::Entity::update_many()
            .filter(posts::Column::Uuid.eq(&post.uuid))
            .col_expr(posts::Column::Status, sea_orm::prelude::Expr::value(2i16))
            .exec(&txn)
            .await
            .unwrap();
        assert_eq!(version(&db).await.unwrap(), before);
        txn.rollback().await.unwrap();
        assert_eq!(version(&db).await.unwrap(), before);
        crate::services::posts::PostService::increment_view(&db, &post.uuid, "127.0.0.9")
            .await
            .unwrap();
        assert_eq!(version(&db).await.unwrap(), before);
        posts::Entity::update_many()
            .filter(posts::Column::Uuid.eq(&post.uuid))
            .col_expr(posts::Column::Status, sea_orm::prelude::Expr::value(2i16))
            .exec(&db)
            .await
            .unwrap();
        assert_ne!(version(&db).await.unwrap(), before);
    }
}
