use crate::config::AppError;
use crate::{ApiResponse, HttpResult};
use actix_web::web;
use chrono::Utc;
use sea_orm::{ConnectionTrait, DatabaseConnection, FromQueryResult, Statement};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct FeedbackInput {
    #[validate(length(min = 2, max = 32))]
    pub kind: String,
    #[validate(length(max = 2000))]
    pub message: Option<String>,
    #[validate(length(max = 255))]
    pub contact: Option<String>,
    #[validate(length(max = 64))]
    pub post_uuid: Option<String>,
}

pub async fn feedback_handler(
    db: web::Data<DatabaseConnection>,
    body: web::Json<FeedbackInput>,
) -> HttpResult {
    body.validate()
        .map_err(|_| AppError::BadRequest("反馈内容不符合要求".into()))?;
    if !["helpful", "not_helpful", "message"].contains(&body.kind.as_str()) {
        return Err(AppError::BadRequest("不支持的反馈类型".into()).into());
    }
    if body.kind == "message" && body.message.as_deref().unwrap_or("").trim().is_empty() {
        return Err(AppError::BadRequest("请填写反馈内容".into()).into());
    }
    db.execute(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        "INSERT INTO reader_feedback(post_uuid,kind,message,contact) VALUES($1,$2,$3,$4)",
        vec![
            body.post_uuid.clone().into(),
            body.kind.clone().into(),
            body.message.clone().into(),
            body.contact.clone().into(),
        ],
    ))
    .await?;
    Ok(
        ApiResponse::success(serde_json::json!({"accepted": true}), "感谢你的反馈")
            .to_http_response(),
    )
}

#[derive(Deserialize, Validate)]
pub struct SubscriptionInput {
    #[validate(email, length(max = 255))]
    pub email: String,
}
pub async fn subscribe_handler(
    db: web::Data<DatabaseConnection>,
    body: web::Json<SubscriptionInput>,
) -> HttpResult {
    body.validate()
        .map_err(|_| AppError::BadRequest("请输入有效邮箱".into()))?;
    db.execute(Statement::from_sql_and_values(sea_orm::DatabaseBackend::Postgres,
        "INSERT INTO email_subscriptions(email,status,subscribed_at,updated_at) VALUES($1,'active',NOW(),NOW()) ON CONFLICT(email) DO UPDATE SET status='active',updated_at=NOW()",
        [body.email.trim().to_lowercase().into()])).await?;
    Ok(
        ApiResponse::success(serde_json::json!({"subscribed": true}), "订阅已登记")
            .to_http_response(),
    )
}

#[derive(Serialize)]
pub struct Analytics {
    pub total_posts: i64,
    pub published: i64,
    pub drafts: i64,
    pub offline: i64,
    pub total_views: i64,
    pub views_7d: i64,
    pub feedback_7d: i64,
    pub subscribers: i64,
    pub generated_at: chrono::DateTime<Utc>,
}
#[derive(FromQueryResult)]
struct Counts {
    total_posts: i64,
    published: i64,
    drafts: i64,
    offline: i64,
    total_views: i64,
    views_7d: i64,
    feedback_7d: i64,
    subscribers: i64,
}
pub async fn analytics_handler(db: web::Data<DatabaseConnection>) -> HttpResult {
    let sql = "SELECT (SELECT count(*) FROM posts) total_posts,(SELECT count(*) FROM posts WHERE status=1) published,(SELECT count(*) FROM posts WHERE status=0) drafts,(SELECT count(*) FROM posts WHERE status=2) offline,(SELECT COALESCE(sum(view_count),0) FROM posts) total_views,(SELECT count(*) FROM post_views WHERE created_at >= NOW()-INTERVAL '7 days') views_7d,(SELECT count(*) FROM reader_feedback WHERE created_at >= NOW()-INTERVAL '7 days') feedback_7d,(SELECT count(*) FROM email_subscriptions WHERE status='active') subscribers";
    let row = Counts::find_by_statement(Statement::from_string(
        sea_orm::DatabaseBackend::Postgres,
        sql,
    ))
    .one(db.as_ref())
    .await?
    .ok_or_else(|| AppError::DatabaseError("统计数据为空".into()))?;
    Ok(ApiResponse::success(
        Analytics {
            total_posts: row.total_posts,
            published: row.published,
            drafts: row.drafts,
            offline: row.offline,
            total_views: row.total_views,
            views_7d: row.views_7d,
            feedback_7d: row.feedback_7d,
            subscribers: row.subscribers,
            generated_at: Utc::now(),
        },
        "成功",
    )
    .to_http_response())
}
