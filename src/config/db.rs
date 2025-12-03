use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use std::time::Duration;

use crate::config::manager::CONFIG;

/// 创建并配置数据库连接池
pub async fn create_db_pool() -> Result<DatabaseConnection, DbErr> {
    let mut opt = ConnectOptions::new(CONFIG.database.url.clone());
    opt.max_connections(CONFIG.database.max_connections)
        .min_connections(CONFIG.database.min_connections)
        .connect_timeout(Duration::from_secs(CONFIG.database.connect_timeout))
        .acquire_timeout(Duration::from_secs(CONFIG.database.acquire_timeout))
        .idle_timeout(Duration::from_secs(CONFIG.database.idle_timeout))
        .max_lifetime(Duration::from_secs(CONFIG.database.max_lifetime))
        .sqlx_logging(CONFIG.database.sqlx_logging)
        .test_before_acquire(CONFIG.database.test_before_acquire)
        .set_schema_search_path(CONFIG.database.set_schema_search_path.clone())
        // .sqlx_logging(true) // 启用 SQL 日志记录，便于调试
        .sqlx_logging_level(log::LevelFilter::Info);
    Database::connect(opt).await
}
