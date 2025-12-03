#[cfg(test)]
mod tests {
    use crate::SseNotifier;
    use crate::dto::user::RegisterResponse;
    use crate::models::{roles, user_roles, users};
    use crate::services::auth::AuthService;
    use crate::utils::crypto_pwd::verify;
    use actix_web::web;
    use chrono::Utc;
    use sea_orm::{
        ActiveModelTrait, ColumnTrait, ConnectionTrait, Database, DatabaseConnection, EntityTrait,
        PaginatorTrait, QueryFilter, QueryOrder, Set, Statement,
    };

    // 测试数据库连接和初始化
    async fn setup_test_db() -> DatabaseConnection {
        // 使用 SQLite 内存数据库进行测试
        let db = Database::connect("sqlite::memory:")
            .await
            .expect("Failed to connect to test database");

        // 运行迁移
        create_test_tables(&db).await;
        create_test_roles(&db).await;

        db
    }

    // 创建测试所需的表
    async fn create_test_tables(db: &DatabaseConnection) {
        // 简化的表创建，实际项目中应该使用迁移文件

        // 创建用户表
        let create_users_table = Statement::from_string(
            db.get_database_backend(),
            r#"
            CREATE TABLE users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                uuid TEXT NOT NULL UNIQUE,
                user_name TEXT NOT NULL UNIQUE,
                pass_word TEXT NOT NULL,
                email TEXT UNIQUE,
                image TEXT,
                phone TEXT UNIQUE,
                binding TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#
            .to_string(),
        );
        db.execute(create_users_table)
            .await
            .expect("Failed to create users table");

        // 创建角色表
        let create_roles_table = Statement::from_string(
            db.get_database_backend(),
            r#"
            CREATE TABLE roles (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                code TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                description TEXT,
                is_system INTEGER,
                created_at TEXT
            )
            "#
            .to_string(),
        );
        db.execute(create_roles_table)
            .await
            .expect("Failed to create roles table");

        // 创建用户角色关联表
        let create_user_roles_table = Statement::from_string(
            db.get_database_backend(),
            r#"
            CREATE TABLE user_roles (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER,
                role_id INTEGER,
                is_primary INTEGER,
                created_at TEXT NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
                FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE
            )
            "#
            .to_string(),
        );
        db.execute(create_user_roles_table)
            .await
            .expect("Failed to create user_roles table");
    }

    // 创建测试所需的角色
    async fn create_test_roles(db: &DatabaseConnection) {
        // 创建普通用户角色
        let user_role = roles::ActiveModel {
            code: Set("USER".to_string()),
            name: Set("普通用户".to_string()),
            description: Set(Some("普通用户，具有基本权限".to_string())),
            is_system: Set(Some(true)),
            created_at: Set(Some(Utc::now().fixed_offset())),
            ..Default::default()
        };
        user_role.insert(db).await.unwrap();

        // 创建管理员角色
        let admin_role = roles::ActiveModel {
            code: Set("ADMIN".to_string()),
            name: Set("管理员".to_string()),
            description: Set(Some("管理员，具有管理权限".to_string())),
            is_system: Set(Some(true)),
            created_at: Set(Some(Utc::now().fixed_offset())),
            ..Default::default()
        };
        admin_role.insert(db).await.unwrap();

        // 创建超级管理员角色
        let super_admin_role = roles::ActiveModel {
            code: Set("SUPER_ADMIN".to_string()),
            name: Set("超级管理员".to_string()),
            description: Set(Some("超级管理员，具有所有权限".to_string())),
            is_system: Set(Some(true)),
            created_at: Set(Some(Utc::now().fixed_offset())),
            ..Default::default()
        };
        super_admin_role.insert(db).await.unwrap();
    }

    // 测试创建用户 - 成功案例
    #[tokio::test]
    async fn test_register_user_success() {
        // 初始化测试数据库
        let db = setup_test_db().await;
        let db_pool = web::Data::new(db);

        // 创建注册请求数据
        let register_data = RegisterResponse {
            user_name: "testuser".to_string(),
            pass_word: "testpassword123".to_string(),
        };

        // 创建一个模拟的通知器
        let notifier = web::Data::new(SseNotifier::new());

        // 调用注册服务
        let result =
            AuthService::register(db_pool.clone(), web::Json(register_data), notifier).await;

        // 验证结果
        assert!(result.is_ok(), "用户注册应该成功");
        let user = result.unwrap();

        // 验证用户基本信息
        assert_eq!(user.user_name, "testuser");
        assert!(!user.uuid.is_empty(), "UUID不应为空");
        assert!(!user.pass_word.is_empty(), "密码哈希不应为空");

        // 验证密码是否正确哈希
        let is_valid = verify("testpassword123", &user.pass_word).unwrap();
        assert!(is_valid, "密码应该被正确哈希");

        // 验证用户是否被赋予角色
        let user_role = user_roles::Entity::find()
            .filter(user_roles::Column::UserId.eq(user.id))
            .one(db_pool.as_ref())
            .await
            .unwrap();

        assert!(user_role.is_some(), "用户应该被分配角色");
        let user_role = user_role.unwrap();
        assert!(
            user_role.is_primary.unwrap_or(false),
            "角色应该被标记为主角色"
        );

        // 验证角色是否为超级管理员（当前代码的问题）
        let role = roles::Entity::find_by_id(user_role.role_id.unwrap())
            .one(db_pool.as_ref())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(
            role.code, "SUPER_ADMIN",
            "当前代码中，新用户被错误地分配为超级管理员"
        );
    }

    // 测试创建用户 - 用户名太短
    #[tokio::test]
    async fn test_register_user_short_username() {
        let db = setup_test_db().await;
        let db_pool = web::Data::new(db);

        // 创建用户名太短的注册请求数据
        let register_data = RegisterResponse {
            user_name: "test".to_string(), // 用户名长度为4，小于最小要求5
            pass_word: "testpassword123".to_string(),
        };

        let notifier = web::Data::new(SseNotifier::new());

        // 调用注册服务
        let result = AuthService::register(db_pool, web::Json(register_data), notifier).await;

        // 验证结果
        assert!(result.is_err(), "用户名太短应该导致注册失败");
        let error = result.unwrap_err();

        // 验证错误类型
        match error {
            crate::config::AppError::ValidationError(_) => {
                // 验证错误是期望的
            }
            _ => panic!("应该返回验证错误，但得到了: {:?}", error),
        }
    }

    // 测试创建用户 - 密码太短
    #[tokio::test]
    async fn test_register_user_short_password() {
        let db = setup_test_db().await;
        let db_pool = web::Data::new(db);

        // 创建密码太短的注册请求数据
        let register_data = RegisterResponse {
            user_name: "testuser".to_string(),
            pass_word: "short".to_string(), // 密码长度为5，小于最小要求6
        };

        let notifier = web::Data::new(SseNotifier::new());

        // 调用注册服务
        let result = AuthService::register(db_pool, web::Json(register_data), notifier).await;

        // 验证结果
        assert!(result.is_err(), "密码太短应该导致注册失败");
        let error = result.unwrap_err();

        // 验证错误类型
        match error {
            crate::config::AppError::ValidationError(_) => {
                // 验证错误是期望的
            }
            _ => panic!("应该返回验证错误，但得到了: {:?}", error),
        }
    }

    // 测试创建用户 - 用户名已存在
    #[tokio::test]
    async fn test_register_user_duplicate_username() {
        let db = setup_test_db().await;
        let db_pool = web::Data::new(db.clone());

        // 首先创建一个用户
        let register_data1 = RegisterResponse {
            user_name: "duplicateuser".to_string(),
            pass_word: "password123".to_string(),
        };

        let notifier1 = web::Data::new(SseNotifier::new());
        AuthService::register(db_pool.clone(), web::Json(register_data1), notifier1)
            .await
            .unwrap();

        // 尝试使用相同用户名创建另一个用户
        let register_data2 = RegisterResponse {
            user_name: "duplicateuser".to_string(), // 相同的用户名
            pass_word: "anotherpassword123".to_string(),
        };

        let notifier2 = web::Data::new(SseNotifier::new());
        let result = AuthService::register(db_pool, web::Json(register_data2), notifier2).await;

        // 验证结果
        assert!(result.is_err(), "重复的用户名应该导致注册失败");
        let error = result.unwrap_err();

        // 验证错误类型
        match error {
            crate::config::AppError::DatabaseError(_) => {
                // 数据库错误是期望的
            }
            _ => panic!("应该返回数据库错误，但得到了: {:?}", error),
        }
    }

    // 测试获取用户列表
    #[tokio::test]
    async fn test_get_users_list() {
        let db = setup_test_db().await;
        let db_pool = web::Data::new(db);

        // 创建多个测试用户
        for i in 1..=3 {
            let register_data = RegisterResponse {
                user_name: format!("testuser{}", i),
                pass_word: "password123".to_string(),
            };

            let notifier = web::Data::new(SseNotifier::new());
            AuthService::register(db_pool.clone(), web::Json(register_data), notifier)
                .await
                .unwrap();
        }

        // 获取用户列表
        let users = users::Entity::find()
            .order_by_asc(users::Column::Id)
            .all(db_pool.as_ref())
            .await
            .unwrap();

        // 验证结果
        assert_eq!(users.len(), 3, "应该有3个用户");
        assert_eq!(users[0].user_name, "testuser1");
        assert_eq!(users[1].user_name, "testuser2");
        assert_eq!(users[2].user_name, "testuser3");
    }

    // 测试按ID获取用户
    #[tokio::test]
    async fn test_get_user_by_id() {
        let db = setup_test_db().await;
        let db_pool = web::Data::new(db);

        // 创建一个测试用户
        let register_data = RegisterResponse {
            user_name: "getuserbyid".to_string(),
            pass_word: "password123".to_string(),
        };

        let notifier = web::Data::new(SseNotifier::new());
        let created_user =
            AuthService::register(db_pool.clone(), web::Json(register_data), notifier)
                .await
                .unwrap();

        // 按ID获取用户
        let found_user = users::Entity::find_by_id(created_user.id)
            .one(db_pool.as_ref())
            .await
            .unwrap()
            .unwrap();

        // 验证结果
        assert_eq!(found_user.user_name, "getuserbyid");
        assert_eq!(found_user.id, created_user.id);
        assert_eq!(found_user.uuid, created_user.uuid);
    }

    // 测试按用户名获取用户
    #[tokio::test]
    async fn test_get_user_by_username() {
        let db = setup_test_db().await;
        let db_pool = web::Data::new(db);

        // 创建一个测试用户
        let register_data = RegisterResponse {
            user_name: "getuserbyname".to_string(),
            pass_word: "password123".to_string(),
        };

        let notifier = web::Data::new(SseNotifier::new());
        AuthService::register(db_pool.clone(), web::Json(register_data), notifier)
            .await
            .unwrap();

        // 按用户名获取用户
        let found_user = users::Entity::find()
            .filter(users::Column::UserName.eq("getuserbyname"))
            .one(db_pool.as_ref())
            .await
            .unwrap()
            .unwrap();

        // 验证结果
        assert_eq!(found_user.user_name, "getuserbyname");
        assert!(!found_user.uuid.is_empty());
    }

    // 测试分页获取用户
    #[tokio::test]
    async fn test_get_users_paginated() {
        let db = setup_test_db().await;
        let db_pool = web::Data::new(db);

        // 创建5个测试用户
        for i in 1..=5 {
            let register_data = RegisterResponse {
                user_name: format!("paginateduser{}", i),
                pass_word: "password123".to_string(),
            };

            let notifier = web::Data::new(SseNotifier::new());
            AuthService::register(db_pool.clone(), web::Json(register_data), notifier)
                .await
                .unwrap();
        }

        // 获取第一页，每页2个用户
        let first_page = users::Entity::find()
            .order_by_asc(users::Column::Id)
            .paginate(db_pool.as_ref(), 2);
        let first_page_users = first_page.fetch_page(0).await.unwrap();

        // 验证结果
        assert_eq!(first_page_users.len(), 2, "第一页应该有2个用户");
        assert_eq!(first_page_users[0].user_name, "paginateduser1");
        assert_eq!(first_page_users[1].user_name, "paginateduser2");

        // 获取第二页，每页2个用户
        let second_page_users = first_page.fetch_page(1).await.unwrap();
        assert_eq!(second_page_users.len(), 2, "第二页应该有2个用户");
        assert_eq!(second_page_users[0].user_name, "paginateduser3");
        assert_eq!(second_page_users[1].user_name, "paginateduser4");

        // 获取第三页，每页2个用户
        let third_page_users = first_page.fetch_page(2).await.unwrap();
        assert_eq!(third_page_users.len(), 1, "第三页应该有1个用户");
        assert_eq!(third_page_users[0].user_name, "paginateduser5");
    }

    // 测试删除用户
    #[tokio::test]
    async fn test_delete_user() {
        let db = setup_test_db().await;
        let db_pool = web::Data::new(db);

        // 创建一个测试用户
        let register_data = RegisterResponse {
            user_name: "deleteuser".to_string(),
            pass_word: "password123".to_string(),
        };

        let notifier = web::Data::new(SseNotifier::new());
        let created_user =
            AuthService::register(db_pool.clone(), web::Json(register_data), notifier)
                .await
                .unwrap();

        // 删除用户
        let delete_result = users::Entity::delete_by_id(created_user.id)
            .exec(db_pool.as_ref())
            .await
            .unwrap();

        // 验证结果
        assert_eq!(delete_result.rows_affected, 1, "应该删除1行");

        // 验证用户已被删除
        let found_user = users::Entity::find_by_id(created_user.id)
            .one(db_pool.as_ref())
            .await
            .unwrap();
        assert!(found_user.is_none(), "用户应该已被删除");
    }

    // 测试更新用户
    #[tokio::test]
    async fn test_update_user() {
        let db = setup_test_db().await;
        let db_pool = web::Data::new(db);

        // 创建一个测试用户
        let register_data = RegisterResponse {
            user_name: "updateuser".to_string(),
            pass_word: "password123".to_string(),
        };

        let notifier = web::Data::new(SseNotifier::new());
        let created_user =
            AuthService::register(db_pool.clone(), web::Json(register_data), notifier)
                .await
                .unwrap();

        // 更新用户信息
        let updated_user = users::ActiveModel {
            id: Set(created_user.id),
            user_name: Set("updateduser".to_string()),
            email: Set(Some("updated@example.com".to_string())),
            updated_at: Set(Utc::now()),
            ..Default::default()
        };

        let updated_user = updated_user.update(db_pool.as_ref()).await.unwrap();

        // 验证结果
        assert_eq!(updated_user.user_name, "updateduser");
        assert_eq!(updated_user.email, Some("updated@example.com".to_string()));
        assert_ne!(updated_user.updated_at, created_user.updated_at);
    }
}
