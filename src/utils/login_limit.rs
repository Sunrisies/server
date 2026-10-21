use crate::config::AppError;
use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};
use sha2::{Digest, Sha256};

pub const WINDOW_SECONDS: i64 = 300;

pub async fn initialize(db: &DatabaseConnection) -> Result<(), AppError> {
    // A pre-created table also works with runtime users without CREATE privilege.
    if db
        .query_one(Statement::from_string(
            db.get_database_backend(),
            "SELECT key_hash, attempts, expires_at FROM login_limits LIMIT 1".to_owned(),
        ))
        .await
        .is_ok()
    {
        return Ok(());
    }
    db.execute(Statement::from_string(db.get_database_backend(),
        "CREATE TABLE IF NOT EXISTS login_limits (key_hash VARCHAR(64) PRIMARY KEY, attempts INTEGER NOT NULL, expires_at BIGINT NOT NULL)".to_owned())).await?;
    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE INDEX IF NOT EXISTS login_limits_expiry ON login_limits(expires_at)".to_owned(),
    ))
    .await?;
    Ok(())
}

async fn consume(db: &DatabaseConnection, key: &str, limit: i32, now: i64) -> Result<(), AppError> {
    let key_hash = format!("{:x}", Sha256::digest(key.as_bytes()));
    let statement = Statement::from_sql_and_values(
        db.get_database_backend(),
        "INSERT INTO login_limits (key_hash, attempts, expires_at) VALUES ($1, 1, $2) \
         ON CONFLICT (key_hash) DO UPDATE SET \
         attempts = CASE WHEN login_limits.expires_at <= $3 THEN 1 ELSE login_limits.attempts + 1 END, \
         expires_at = CASE WHEN login_limits.expires_at <= $3 THEN $2 ELSE login_limits.expires_at END \
         WHERE login_limits.expires_at <= $3 OR login_limits.attempts < $4 RETURNING attempts",
        vec![
            key_hash.into(),
            (now + WINDOW_SECONDS).into(),
            now.into(),
            limit.into(),
        ],
    );
    if db.query_one(statement).await?.is_none() {
        return Err(AppError::RateLimited(
            "登录尝试过于频繁，请在 5 分钟后重试".into(),
        ));
    }
    Ok(())
}

pub async fn check(db: &DatabaseConnection, ip: &str, account: &str) -> Result<(), AppError> {
    let now = chrono::Utc::now().timestamp();
    db.execute(Statement::from_sql_and_values(
        db.get_database_backend(),
        "DELETE FROM login_limits WHERE expires_at <= $1",
        vec![now.into()],
    ))
    .await?;
    // Count all admitted attempts, including successful ones. Persisted atomic
    // counters are shared by workers/instances and survive restarts.
    consume(db, &format!("ip:{ip}"), 30, now).await?;
    consume(db, &format!("account:{account}"), 10, now).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::Database;

    #[tokio::test]
    async fn auth_login_limits_expire_and_separate_keys() {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        initialize(&db).await.unwrap();
        for _ in 0..10 {
            consume(&db, "account:a", 10, 100).await.unwrap();
        }
        assert!(matches!(
            consume(&db, "account:a", 10, 101).await,
            Err(AppError::RateLimited(_))
        ));
        consume(&db, "account:b", 10, 101).await.unwrap();
        consume(&db, "account:a", 10, 400).await.unwrap();
    }
}
