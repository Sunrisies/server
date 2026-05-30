//! 测试辅助模块 — 支持 SQLite（本地）和 PostgreSQL（CI）两种测试数据库

use sea_orm::{ConnectionTrait, Database, DatabaseConnection, Statement};

/// 获取测试数据库连接
///
/// 优先使用 `DATABASE_URL_TEST` 环境变量连接 PostgreSQL，
/// 否则使用 SQLite 内存数据库。
pub async fn setup_test_db() -> DatabaseConnection {
    if let Ok(url) = std::env::var("DATABASE_URL_TEST") {
        log::info!("使用 PostgreSQL 测试数据库: {}", url);
        let db = Database::connect(&url)
            .await
            .expect("无法连接到测试数据库，请检查 DATABASE_URL_TEST");
        run_migrations_pg(&db).await;
        db
    } else {
        log::info!("使用 SQLite 内存测试数据库");
        let db = Database::connect("sqlite::memory:")
            .await
            .expect("无法创建 SQLite 内存数据库");
        run_migrations_sqlite(&db).await;
        db
    }
}

/// 清理测试数据（PostgreSQL 下 truncate 所有表，SQLite 下自动释放）
pub async fn cleanup(db: &DatabaseConnection) {
    if std::env::var("DATABASE_URL_TEST").is_ok() {
        // PostgreSQL 下清空数据但不删表
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
            let sql = format!("TRUNCATE TABLE {} CASCADE", table);
            let _ = db
                .execute(Statement::from_string(db.get_database_backend(), sql))
                .await;
        }
    }
    // SQLite：无需清理，连接关闭后自动释放
}

/// PostgreSQL 建表迁移
async fn run_migrations_pg(db: &DatabaseConnection) {
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

/// SQLite 建表迁移（语法与 PostgreSQL 略有不同）
async fn run_migrations_sqlite(db: &DatabaseConnection) {
    for (name, sql) in MIGRATIONS_SQLITE {
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
        "roles",
        "CREATE TABLE IF NOT EXISTS roles (
        id SERIAL PRIMARY KEY, code VARCHAR(50) NOT NULL UNIQUE,
        name VARCHAR(255) NOT NULL, description TEXT, is_system BOOLEAN DEFAULT FALSE,
        created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
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
        "posts",
        "CREATE TABLE IF NOT EXISTS posts (
        id SERIAL PRIMARY KEY, uuid VARCHAR(36) NOT NULL UNIQUE,
        author_id INTEGER NOT NULL REFERENCES users(id),
        category_id INTEGER NOT NULL, title VARCHAR(255) NOT NULL,
        summary TEXT, content TEXT NOT NULL,
        markdowncontent TEXT NOT NULL DEFAULT '',
        cover_image VARCHAR(512), status SMALLINT DEFAULT 0,
        featured BOOLEAN DEFAULT FALSE, view_count INTEGER DEFAULT 0,
        created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
        published_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
        size INTEGER DEFAULT 0
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
        "post_tags",
        "CREATE TABLE IF NOT EXISTS post_tags (
        id SERIAL PRIMARY KEY, post_id INTEGER NOT NULL REFERENCES posts(id),
        tag_id INTEGER NOT NULL REFERENCES tags(id)
    )",
    ),
    (
        "user_roles",
        "CREATE TABLE IF NOT EXISTS user_roles (
        id SERIAL PRIMARY KEY, user_id INTEGER REFERENCES users(id),
        role_id INTEGER REFERENCES roles(id),
        is_primary BOOLEAN DEFAULT FALSE,
        created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
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
        "uploads",
        "CREATE TABLE IF NOT EXISTS uploads (
        id BIGSERIAL PRIMARY KEY, uuid VARCHAR(36) NOT NULL UNIQUE,
        url VARCHAR(512) NOT NULL, key VARCHAR(512) NOT NULL,
        filename VARCHAR(255) NOT NULL, file_size BIGINT NOT NULL,
        mime_type VARCHAR(128) NOT NULL,
        created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
    )",
    ),
];

// SQLite 用简化建表语句（不支持 SERIAL / TIMESTAMP WITH TIME ZONE / BIGSERIAL）
const MIGRATIONS_SQLITE: &[(&str, &str)] = &[
    (
        "users",
        "CREATE TABLE IF NOT EXISTS users (
        id INTEGER PRIMARY KEY AUTOINCREMENT, uuid TEXT NOT NULL UNIQUE,
        user_name TEXT NOT NULL UNIQUE, pass_word TEXT NOT NULL,
        email TEXT, image TEXT, phone TEXT,
        binding TEXT,
        created_at TEXT DEFAULT CURRENT_TIMESTAMP,
        updated_at TEXT DEFAULT CURRENT_TIMESTAMP
    )",
    ),
    (
        "roles",
        "CREATE TABLE IF NOT EXISTS roles (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        code TEXT NOT NULL UNIQUE, name TEXT NOT NULL,
        description TEXT, is_system INTEGER DEFAULT 0,
        created_at TEXT DEFAULT CURRENT_TIMESTAMP
    )",
    ),
    (
        "categories",
        "CREATE TABLE IF NOT EXISTS categories (
        id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL,
        slug TEXT NOT NULL UNIQUE, description TEXT,
        created_at TEXT DEFAULT CURRENT_TIMESTAMP,
        updated_at TEXT DEFAULT CURRENT_TIMESTAMP
    )",
    ),
    (
        "posts",
        "CREATE TABLE IF NOT EXISTS posts (
        id INTEGER PRIMARY KEY AUTOINCREMENT, uuid TEXT NOT NULL UNIQUE,
        author_id INTEGER NOT NULL REFERENCES users(id),
        category_id INTEGER NOT NULL, title TEXT NOT NULL,
        summary TEXT, content TEXT NOT NULL,
        markdowncontent TEXT NOT NULL DEFAULT '',
        cover_image TEXT, status INTEGER DEFAULT 0,
        featured INTEGER DEFAULT 0, view_count INTEGER DEFAULT 0,
        created_at TEXT DEFAULT CURRENT_TIMESTAMP,
        updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
        published_at TEXT DEFAULT CURRENT_TIMESTAMP,
        size INTEGER DEFAULT 0
    )",
    ),
    (
        "tags",
        "CREATE TABLE IF NOT EXISTS tags (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL UNIQUE, description TEXT,
        created_at TEXT DEFAULT CURRENT_TIMESTAMP,
        updated_at TEXT DEFAULT CURRENT_TIMESTAMP
    )",
    ),
    (
        "post_tags",
        "CREATE TABLE IF NOT EXISTS post_tags (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        post_id INTEGER NOT NULL REFERENCES posts(id),
        tag_id INTEGER NOT NULL REFERENCES tags(id)
    )",
    ),
    (
        "user_roles",
        "CREATE TABLE IF NOT EXISTS user_roles (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        user_id INTEGER REFERENCES users(id),
        role_id INTEGER REFERENCES roles(id),
        is_primary INTEGER DEFAULT 0,
        created_at TEXT DEFAULT CURRENT_TIMESTAMP
    )",
    ),
    (
        "post_views",
        "CREATE TABLE IF NOT EXISTS post_views (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        post_id INTEGER NOT NULL REFERENCES posts(id),
        ip TEXT NOT NULL, viewed_date TEXT NOT NULL,
        created_at TEXT DEFAULT CURRENT_TIMESTAMP,
        UNIQUE(post_id, ip, viewed_date)
    )",
    ),
    (
        "uploads",
        "CREATE TABLE IF NOT EXISTS uploads (
        id INTEGER PRIMARY KEY AUTOINCREMENT, uuid TEXT NOT NULL UNIQUE,
        url TEXT NOT NULL, key TEXT NOT NULL,
        filename TEXT NOT NULL, file_size INTEGER NOT NULL,
        mime_type TEXT NOT NULL,
        created_at TEXT DEFAULT CURRENT_TIMESTAMP
    )",
    ),
];
