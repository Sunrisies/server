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
    let mut db = if CONFIG.database.url.starts_with("postgres://")
        || CONFIG.database.url.starts_with("postgresql://")
    {
        use sea_orm::sqlx::{ConnectOptions as _, Postgres, postgres::PgConnectOptions};
        let mut pg: PgConnectOptions = CONFIG
            .database
            .url
            .parse()
            .map_err(|_| DbErr::Custom("Invalid PostgreSQL connection configuration".into()))?;
        if CONFIG.database.sqlx_logging {
            pg = pg.log_statements(log::LevelFilter::Info);
        } else {
            pg = pg.disable_statement_logging();
        }
        // Preserve SeaORM's search_path quoting while binding the setting as data.
        let schema = &CONFIG.database.set_schema_search_path;
        let search_path = if schema.starts_with('"') {
            schema.clone()
        } else {
            schema
                .split(',')
                .map(|part| {
                    if part.starts_with('"') {
                        part.to_owned()
                    } else {
                        format!("\"{part}\"")
                    }
                })
                .collect::<Vec<_>>()
                .join(",")
        };
        let pool = opt
            .sqlx_pool_options::<Postgres>()
            .acquire_time_level(log::LevelFilter::Info)
            .after_connect(move |conn, _| {
                let search_path = search_path.clone();
                Box::pin(async move {
                    sea_orm::sqlx::query("SELECT set_config('search_path', $1, false)")
                        .bind(search_path)
                        .execute(conn)
                        .await?;
                    Ok(())
                })
            })
            .connect_with(pg)
            .await
            .map_err(|err| DbErr::Conn(sea_orm::RuntimeErr::SqlxError(err)))?;
        sea_orm::SqlxPostgresConnector::from_sqlx_postgres_pool(pool)
    } else {
        Database::connect(opt).await?
    };
    crate::observability::instrument_database(&mut db);
    Ok(db)
}
