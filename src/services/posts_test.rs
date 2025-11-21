#[cfg(test)]
mod tests {
    use crate::dto::posts::CreatePostRequest;
    use crate::models::{categories, post_tags, posts, tags, users};
    use crate::services::posts::PostService;
    use sea_orm::{
        ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
    };
    use uuid::Uuid;

    // 测试创建文章
    #[tokio::test]
    async fn test_create_post() {
        // 初始化测试数据库连接
        let db = setup_test_db().await;

        // 创建测试用户
        let user = create_test_user(&db).await;

        // 创建测试分类
        let category = create_test_category(&db).await;

        // 创建测试标签
        let tag1 = create_test_tag(&db, "Rust".to_string()).await;
        let tag2 = create_test_tag(&db, "Web开发".to_string()).await;

        // 构建创建文章的请求
        let create_post_request = CreatePostRequest {
            title: "测试文章标题".to_string(),
            summary: Some("这是测试文章的摘要".to_string()),
            content: "这是测试文章的内容，包含了一些示例文本。".to_string(),
            markdowncontent: "# 测试文章\n\n这是测试文章的内容。".to_string(),
            cover_image: Some("https://example.com/cover.jpg".to_string()),
            category_id: category.id,
            tag_ids: vec![tag1.id, tag2.id],
            status: 1, // 发布状态
            featured: false,
        };

        // 调用创建文章的服务方法
        let result = PostService::create_post(&db, user.id, create_post_request).await;

        // 验证结果
        assert!(result.is_ok(), "创建文章应该成功");
        let post_response = result.unwrap();

        // 验证文章基本信息
        assert_eq!(post_response.title, "测试文章标题");
        assert_eq!(post_response.description, "这是测试文章的摘要");
        assert_eq!(post_response.author, user.user_name);
        assert!(post_response.is_publish);
        assert!(!post_response.is_top);
        assert!(!post_response.is_hide);

        // 验证分类信息
        assert!(post_response.category.is_some());
        let category_name = &post_response.category.as_ref().unwrap().name;
        assert!(category_name.starts_with("测试分类-"));

        // 验证标签信息
        assert_eq!(post_response.tags.len(), 2);
        let tag_names: Vec<String> = post_response.tags.iter().map(|t| t.name.clone()).collect();
        assert!(tag_names.iter().any(|name| name.starts_with("Rust-")));
        assert!(tag_names.iter().any(|name| name.starts_with("Web开发-")));

        // 清理测试数据
        cleanup_test_data(&db, post_response.id).await;
    }

    // 测试创建草稿文章
    #[tokio::test]
    async fn test_create_draft_post() {
        // 初始化测试数据库连接
        let db = setup_test_db().await;

        // 创建测试用户
        let user = create_test_user(&db).await;

        // 创建测试分类
        let category = create_test_category(&db).await;

        // 创建测试标签
        let tag1 = create_test_tag(&db, "草稿".to_string()).await;

        // 构建创建草稿文章的请求
        let create_post_request = CreatePostRequest {
            title: "草稿文章标题".to_string(),
            summary: None,
            content: "这是草稿文章的内容。".to_string(),
            markdowncontent: "# 草稿文章\n\n这是草稿文章的内容。".to_string(),
            cover_image: None,
            category_id: category.id,
            tag_ids: vec![tag1.id],
            status: 0, // 草稿状态
            featured: false,
        };

        // 调用创建文章的服务方法
        let result = PostService::create_post(&db, user.id, create_post_request).await;

        // 验证结果
        assert!(result.is_ok(), "创建草稿文章应该成功");
        let post_response = result.unwrap();

        // 验证文章基本信息
        assert_eq!(post_response.title, "草稿文章标题");
        assert!(!post_response.is_publish);
        assert!(!post_response.is_hide);

        // 验证标签信息
        assert_eq!(post_response.tags.len(), 1);
        let tag_name = &post_response.tags[0].name;
        assert!(tag_name.starts_with("草稿-"));

        // 清理测试数据
        cleanup_test_data(&db, post_response.id).await;
    }

    // 测试创建隐藏文章
    #[tokio::test]
    async fn test_create_hidden_post() {
        // 初始化测试数据库连接
        let db = setup_test_db().await;

        // 创建测试用户
        let user = create_test_user(&db).await;

        // 创建测试分类
        let category = create_test_category(&db).await;

        // 创建测试标签
        let tag1 = create_test_tag(&db, "隐藏".to_string()).await;

        // 构建创建隐藏文章的请求
        let create_post_request = CreatePostRequest {
            title: "隐藏文章标题".to_string(),
            summary: Some("这是隐藏文章的摘要".to_string()),
            content: "这是隐藏文章的内容。".to_string(),
            markdowncontent: "# 隐藏文章\n\n这是隐藏文章的内容。".to_string(),
            cover_image: Some("https://example.com/hidden.jpg".to_string()),
            category_id: category.id,
            tag_ids: vec![tag1.id],
            status: 2, // 隐藏状态
            featured: false,
        };

        // 调用创建文章的服务方法
        let result = PostService::create_post(&db, user.id, create_post_request).await;

        // 验证结果
        assert!(result.is_ok(), "创建隐藏文章应该成功");
        let post_response = result.unwrap();

        // 验证文章基本信息
        assert_eq!(post_response.title, "隐藏文章标题");
        assert!(!post_response.is_publish);
        assert!(post_response.is_hide);

        // 验证标签信息
        assert_eq!(post_response.tags.len(), 1);
        let tag_name = &post_response.tags[0].name;
        assert!(tag_name.starts_with("隐藏-"));

        // 清理测试数据
        cleanup_test_data(&db, post_response.id).await;
    }

    // 测试创建置顶文章
    #[tokio::test]
    async fn test_create_featured_post() {
        // 初始化测试数据库连接
        let db = setup_test_db().await;

        // 创建测试用户
        let user = create_test_user(&db).await;

        // 创建测试分类
        let category = create_test_category(&db).await;

        // 创建测试标签
        let tag1 = create_test_tag(&db, "置顶".to_string()).await;

        // 构建创建置顶文章的请求
        let create_post_request = CreatePostRequest {
            title: "置顶文章标题".to_string(),
            summary: Some("这是置顶文章的摘要".to_string()),
            content: "这是置顶文章的内容。".to_string(),
            markdowncontent: "# 置顶文章\n\n这是置顶文章的内容。".to_string(),
            cover_image: Some("https://example.com/featured.jpg".to_string()),
            category_id: category.id,
            tag_ids: vec![tag1.id],
            status: 1,      // 发布状态
            featured: true, // 置顶
        };

        // 调用创建文章的服务方法
        let result = PostService::create_post(&db, user.id, create_post_request).await;

        // 验证结果
        assert!(result.is_ok(), "创建置顶文章应该成功");
        let post_response = result.unwrap();

        // 验证文章基本信息
        assert_eq!(post_response.title, "置顶文章标题");
        assert!(post_response.is_publish);
        assert!(post_response.is_top);

        // 验证标签信息
        assert_eq!(post_response.tags.len(), 1);
        let tag_name = &post_response.tags[0].name;
        assert!(tag_name.starts_with("置顶-"));

        // 清理测试数据
        cleanup_test_data(&db, post_response.id).await;
    }

    // 设置测试数据库连接
    async fn setup_test_db() -> DatabaseConnection {
        use dotenvy::dotenv;
        use sea_orm::{ConnectOptions, Database};
        use std::env;
        use std::time::Duration;

        // 加载环境变量
        dotenv().ok();

        // 尝试获取测试数据库URL，如果没有则使用主数据库URL
        let database_url = env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| env::var("DATABASE_URL").expect("DATABASE_URL must be set"));

        let mut opt = ConnectOptions::new(database_url);
        opt.max_connections(10)
            .min_connections(1)
            .connect_timeout(Duration::from_secs(8))
            .acquire_timeout(Duration::from_secs(8))
            .idle_timeout(Duration::from_secs(8))
            .max_lifetime(Duration::from_secs(8))
            .sqlx_logging(true) // 测试时启用SQL日志记录，便于调试
            .test_before_acquire(true)
            .set_schema_search_path("public")
            .sqlx_logging_level(log::LevelFilter::Debug);

        Database::connect(opt)
            .await
            .expect("Failed to connect to test database")
    }

    // 创建测试用户
    async fn create_test_user(db: &DatabaseConnection) -> users::Model {
        let user_uuid = Uuid::new_v4();
        let user = users::ActiveModel {
            uuid: Set(user_uuid.to_string()),
            user_name: Set(format!("测试用户-{}", user_uuid)),
            email: Set(Some(format!("test-{}@example.com", user_uuid))),
            pass_word: Set("hashed_password".to_string()),
            ..Default::default()
        };

        user.insert(db).await.expect("创建测试用户失败")
    }

    // 创建测试分类
    async fn create_test_category(db: &DatabaseConnection) -> categories::Model {
        let category_uuid = Uuid::new_v4();
        let category = categories::ActiveModel {
            name: Set(format!("测试分类-{}", category_uuid)),
            slug: Set(format!("test-category-{}", category_uuid)),
            description: Set(Some("用于测试的分类".to_string())),
            ..Default::default()
        };

        category.insert(db).await.expect("创建测试分类失败")
    }

    // 创建测试标签
    async fn create_test_tag(db: &DatabaseConnection, name: String) -> tags::Model {
        let tag_uuid = Uuid::new_v4();
        let unique_name = format!("{}-{}", name, tag_uuid);
        let tag = tags::ActiveModel {
            name: Set(unique_name),
            ..Default::default()
        };

        tag.insert(db).await.expect("创建测试标签失败")
    }

    // 清理测试数据
    async fn cleanup_test_data(db: &DatabaseConnection, post_id: i32) {
        // 删除文章标签关联
        post_tags::Entity::delete_many()
            .filter(post_tags::Column::PostId.eq(post_id))
            .exec(db)
            .await
            .expect("删除文章标签关联失败");

        // 删除文章
        posts::Entity::delete_by_id(post_id)
            .exec(db)
            .await
            .expect("删除测试文章失败");
    }
}
