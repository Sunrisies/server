use crate::{
    config::AppError,
    models::{post_tags, posts, tags},
};
use chrono::{DateTime, Utc};
use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, FromQueryResult, QueryFilter,
    Statement,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct PostVersion {
    pub id: i64,
    pub title: String,
    pub summary: Option<String>,
    pub content: String,
    pub markdowncontent: String,
    pub cover_image: Option<String>,
    pub category_id: i32,
    pub tag_ids: Vec<i32>,
    pub status: i16,
    pub featured: bool,
    pub created_at: DateTime<Utc>,
}

pub async fn initialize(db: &DatabaseConnection) -> Result<(), AppError> {
    db.execute_unprepared("CREATE TABLE IF NOT EXISTS post_versions (id BIGSERIAL PRIMARY KEY, post_id INTEGER NOT NULL REFERENCES posts(id) ON DELETE CASCADE, title VARCHAR(255) NOT NULL, summary TEXT, content TEXT NOT NULL, markdowncontent TEXT NOT NULL, cover_image TEXT, category_id INTEGER NOT NULL, tag_ids JSONB NOT NULL DEFAULT '[]'::jsonb, status SMALLINT NOT NULL, featured BOOLEAN NOT NULL, created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()); CREATE INDEX IF NOT EXISTS idx_post_versions_post_created ON post_versions(post_id, created_at DESC);").await?;
    Ok(())
}

pub async fn record<C: ConnectionTrait>(db: &C, post: &posts::Model) -> Result<(), AppError> {
    let tag_ids = post_tags::Entity::find()
        .filter(post_tags::Column::PostId.eq(post.id))
        .find_also_related(tags::Entity)
        .all(db)
        .await?
        .into_iter()
        .filter_map(|(_, tag)| tag.map(|tag| tag.id))
        .collect::<Vec<_>>();
    let values = vec![
        post.id.into(),
        post.title.clone().into(),
        post.summary.clone().into(),
        post.content.clone().into(),
        post.markdowncontent.clone().into(),
        post.cover_image.clone().into(),
        post.category_id.into(),
        serde_json::to_value(tag_ids).unwrap().into(),
        post.status.into(),
        post.featured.into(),
    ];
    db.execute(Statement::from_sql_and_values(sea_orm::DatabaseBackend::Postgres, "INSERT INTO post_versions(post_id,title,summary,content,markdowncontent,cover_image,category_id,tag_ids,status,featured) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)", values)).await?;
    Ok(())
}

pub async fn list(db: &DatabaseConnection, post_id: i32) -> Result<Vec<PostVersion>, AppError> {
    #[derive(sea_orm::FromQueryResult)]
    struct Row {
        id: i64,
        title: String,
        summary: Option<String>,
        content: String,
        markdowncontent: String,
        cover_image: Option<String>,
        category_id: i32,
        tag_ids: serde_json::Value,
        status: i16,
        featured: bool,
        created_at: DateTime<Utc>,
    }
    let rows = Row::find_by_statement(Statement::from_sql_and_values(sea_orm::DatabaseBackend::Postgres, "SELECT id,title,summary,content,markdowncontent,cover_image,category_id,tag_ids,status,featured,created_at FROM post_versions WHERE post_id=$1 ORDER BY created_at DESC,id DESC LIMIT 50", [post_id.into()])).all(db).await?;
    Ok(rows
        .into_iter()
        .map(|row| PostVersion {
            id: row.id,
            title: row.title,
            summary: row.summary,
            content: row.content,
            markdowncontent: row.markdowncontent,
            cover_image: row.cover_image,
            category_id: row.category_id,
            tag_ids: serde_json::from_value(row.tag_ids).unwrap_or_default(),
            status: row.status,
            featured: row.featured,
            created_at: row.created_at,
        })
        .collect())
}
