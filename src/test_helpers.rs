//! 测试辅助模块 — 每次运行前自动重建表结构，保证隔离

use sea_orm::{ConnectionTrait, Database, DatabaseConnection, Statement};

/// 获取测试数据库连接
///
/// 从 `.env` 文件的 `DATABASE_URL_TEST` 读取连接地址。
/// 每次调用时会删除所有旧表并重建，确保测试环境干净。
pub async fn setup_test_db() -> DatabaseConnection {
    let _ = dotenvy::dotenv();

    let url = std::env::var("DATABASE_URL_TEST").expect("请在 .env 中设置 DATABASE_URL_TEST");

    log::info!("测试数据库: {}", url);
    let db = Database::connect(&url).await.expect("无法连接到测试数据库");

    drop_all_tables(&db).await;
    run_migrations(&db).await;
    db
}

async fn drop_all_tables(db: &DatabaseConnection) {
    let tables = [
        "post_views",
        "clipboard_entries",
        "clipboard_channels",
        "uploads",
        "images",
        "external_links",
        "post_tags",
        "posts",
        "tags",
        "categories",
        "user_roles",
        "role_permissions",
        "user_permissions",
        "permissions",
        "room_messages",
        "rooms",
        "users",
        "roles",
    ];
    for table in tables {
        let sql = format!("DROP TABLE IF EXISTS {} CASCADE", table);
        let _ = db
            .execute(Statement::from_string(db.get_database_backend(), sql))
            .await;
    }
}

async fn run_migrations(db: &DatabaseConnection) {
    for (name, sql) in MIGRATIONS {
        let r = db
            .execute(Statement::from_string(
                db.get_database_backend(),
                sql.to_string(),
            ))
            .await;
        if let Err(e) = r
            && !e.to_string().contains("already exists")
        {
            panic!("迁移 {} 失败: {}", name, e);
        }
    }
}

const MIGRATIONS: &[(&str, &str)] = &[
    (
        "users",
        "CREATE TABLE IF NOT EXISTS users (
        id SERIAL PRIMARY KEY, uuid VARCHAR(36) NOT NULL UNIQUE,
        user_name VARCHAR(255) NOT NULL UNIQUE, pass_word VARCHAR(255) NOT NULL,
        email VARCHAR(255), image VARCHAR(255), phone VARCHAR(20),
        binding VARCHAR(255),
        created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
    )",
    ),
    (
        "verification_codes",
        "CREATE TABLE IF NOT EXISTS verification_codes (
        id BIGSERIAL PRIMARY KEY, email VARCHAR(255) NOT NULL,
        code VARCHAR(10) NOT NULL, expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
        used BOOLEAN NOT NULL DEFAULT FALSE,
        created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
    )",
    ),
    (
        "roles",
        "CREATE TABLE IF NOT EXISTS roles (
        id SERIAL PRIMARY KEY, code VARCHAR(50) NOT NULL UNIQUE,
        name VARCHAR(255) NOT NULL, description TEXT, is_system BOOLEAN DEFAULT FALSE,
        created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
    )",
    ),
    (
        "verification_codes",
        "CREATE TABLE IF NOT EXISTS verification_codes (
        id BIGSERIAL PRIMARY KEY, email VARCHAR(255) NOT NULL,
        code VARCHAR(10) NOT NULL, expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
        used BOOLEAN NOT NULL DEFAULT FALSE,
        created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
    )",
    ),
    (
        "categories",
        "CREATE TABLE IF NOT EXISTS categories (
        id SERIAL PRIMARY KEY, name VARCHAR(255) NOT NULL,
        slug VARCHAR(255) NOT NULL UNIQUE, description TEXT,
        created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
    )",
    ),
    (
        "verification_codes",
        "CREATE TABLE IF NOT EXISTS verification_codes (
        id BIGSERIAL PRIMARY KEY, email VARCHAR(255) NOT NULL,
        code VARCHAR(10) NOT NULL, expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
        used BOOLEAN NOT NULL DEFAULT FALSE,
        created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
    )",
    ),
    (
        "posts",
        "CREATE TABLE IF NOT EXISTS posts (
        id SERIAL PRIMARY KEY, uuid VARCHAR(36) NOT NULL UNIQUE,
        author_id INTEGER NOT NULL REFERENCES users(id),
        category_id INTEGER NOT NULL, title VARCHAR(255) NOT NULL,
        summary TEXT, content TEXT NOT NULL,
        markdowncontent TEXT NOT NULL DEFAULT '',
        cover_image VARCHAR(512), status SMALLINT DEFAULT 0,
        featured BOOLEAN DEFAULT FALSE, view_count INTEGER DEFAULT 0,
        size INTEGER DEFAULT 0,
        created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
        published_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
    )",
    ),
    (
        "verification_codes",
        "CREATE TABLE IF NOT EXISTS verification_codes (
        id BIGSERIAL PRIMARY KEY, email VARCHAR(255) NOT NULL,
        code VARCHAR(10) NOT NULL, expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
        used BOOLEAN NOT NULL DEFAULT FALSE,
        created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
    )",
    ),
    (
        "tags",
        "CREATE TABLE IF NOT EXISTS tags (
        id SERIAL PRIMARY KEY, name VARCHAR(100) NOT NULL UNIQUE,
        description TEXT,
        created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
    )",
    ),
    (
        "verification_codes",
        "CREATE TABLE IF NOT EXISTS verification_codes (
        id BIGSERIAL PRIMARY KEY, email VARCHAR(255) NOT NULL,
        code VARCHAR(10) NOT NULL, expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
        used BOOLEAN NOT NULL DEFAULT FALSE,
        created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
    )",
    ),
    (
        "post_tags",
        "CREATE TABLE IF NOT EXISTS post_tags (
        id SERIAL PRIMARY KEY, post_id INTEGER NOT NULL REFERENCES posts(id),
        tag_id INTEGER NOT NULL REFERENCES tags(id)
    )",
    ),
    (
        "verification_codes",
        "CREATE TABLE IF NOT EXISTS verification_codes (
        id BIGSERIAL PRIMARY KEY, email VARCHAR(255) NOT NULL,
        code VARCHAR(10) NOT NULL, expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
        used BOOLEAN NOT NULL DEFAULT FALSE,
        created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
    )",
    ),
    (
        "user_roles",
        "CREATE TABLE IF NOT EXISTS user_roles (
        id SERIAL PRIMARY KEY, user_id INTEGER REFERENCES users(id) ON DELETE CASCADE,
        role_id INTEGER REFERENCES roles(id) ON DELETE CASCADE,
        is_primary BOOLEAN DEFAULT FALSE,
        created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
    )",
    ),
    (
        "verification_codes",
        "CREATE TABLE IF NOT EXISTS verification_codes (
        id BIGSERIAL PRIMARY KEY, email VARCHAR(255) NOT NULL,
        code VARCHAR(10) NOT NULL, expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
        used BOOLEAN NOT NULL DEFAULT FALSE,
        created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
    )",
    ),
    (
        "post_views",
        "CREATE TABLE IF NOT EXISTS post_views (
        id BIGSERIAL PRIMARY KEY, post_id INTEGER NOT NULL REFERENCES posts(id),
        ip VARCHAR(45) NOT NULL, viewed_date DATE NOT NULL,
        created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
        UNIQUE(post_id, ip, viewed_date)
    )",
    ),
    (
        "verification_codes",
        "CREATE TABLE IF NOT EXISTS verification_codes (
        id BIGSERIAL PRIMARY KEY, email VARCHAR(255) NOT NULL,
        code VARCHAR(10) NOT NULL, expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
        used BOOLEAN NOT NULL DEFAULT FALSE,
        created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
    )",
    ),
    (
        "uploads",
        "CREATE TABLE IF NOT EXISTS uploads (
        id BIGSERIAL PRIMARY KEY, uuid VARCHAR(36) NOT NULL UNIQUE,
        url VARCHAR(512) NOT NULL, key VARCHAR(512) NOT NULL,
        filename VARCHAR(255) NOT NULL, file_size BIGINT NOT NULL,
        mime_type VARCHAR(128) NOT NULL,
        created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
    )",
    ),
    (
        "verification_codes",
        "CREATE TABLE IF NOT EXISTS verification_codes (
        id BIGSERIAL PRIMARY KEY, email VARCHAR(255) NOT NULL,
        code VARCHAR(10) NOT NULL, expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
        used BOOLEAN NOT NULL DEFAULT FALSE,
        created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
    )",
    ),
];

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{EntityTrait, PaginatorTrait};

    #[tokio::test]
    async fn test_setup_and_tables_exist() {
        let db = setup_test_db().await;
        assert!(db.ping().await.is_ok(), "连接失败");
        let count: Result<u64, _> = crate::models::users::Entity::find().count(&db).await;
        assert!(count.is_ok(), "users 表不存在");
        println!("测试数据库连接正常，users 表已创建");
    }
}
