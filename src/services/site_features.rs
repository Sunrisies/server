use crate::config::AppError;
use sea_orm::{ConnectionTrait, DatabaseConnection};

/// Small, migration-safe tables for reader-facing features. Keeping their schema
/// creation here matches the existing version history and upload-job services.
pub async fn initialize(db: &DatabaseConnection) -> Result<(), AppError> {
    db.execute_unprepared("CREATE TABLE IF NOT EXISTS reader_feedback (id BIGSERIAL PRIMARY KEY, post_uuid VARCHAR(64), kind VARCHAR(32) NOT NULL, message TEXT, contact VARCHAR(255), created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()); CREATE INDEX IF NOT EXISTS idx_reader_feedback_created ON reader_feedback(created_at DESC); CREATE TABLE IF NOT EXISTS email_subscriptions (id BIGSERIAL PRIMARY KEY, email VARCHAR(255) NOT NULL UNIQUE, status VARCHAR(16) NOT NULL DEFAULT 'active', subscribed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(), updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()); CREATE INDEX IF NOT EXISTS idx_email_subscriptions_status ON email_subscriptions(status);").await?;
    Ok(())
}
