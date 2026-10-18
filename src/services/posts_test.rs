#[cfg(test)]
mod tests {
    use crate::dto::posts::{CreatePostRequest, UpdatePostRequest};
    use crate::models::{categories, post_tags, posts, tags, users};
    use crate::services::posts::PostService;
    use sea_orm::{
        ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel,
        QueryFilter, Set,
    };
    use uuid::Uuid;

    #[tokio::test]
    async fn markdown_only_draft_can_be_published_with_current_timestamp() {
        let db = setup_test_db().await;
        let user = create_test_user(&db).await;
        let category = create_test_category(&db).await;
        let tag = create_test_tag(&db, "写作流程".into()).await;
        let markdown = "# 仅写 Markdown\n\n正文";
        let created = PostService::create_post(
            &db,
            user.id,
            CreatePostRequest {
                title: "草稿到发布".into(),
                summary: None,
                content: String::new(),
                markdowncontent: markdown.into(),
                cover_image: None,
                category_id: category.id,
                tag_ids: vec![tag.id],
                status: 0,
                featured: false,
            },
        )
        .await
        .unwrap();
        assert_eq!(created.size as usize, markdown.len());
        let mut draft = posts::Entity::find_by_uuid(&created.uuid)
            .one(&db)
            .await
            .unwrap()
            .unwrap()
            .into_active_model();
        let old_date = chrono::Utc::now() - chrono::Duration::days(1);
        draft.published_at = Set(old_date);
        draft.update(&db).await.unwrap();
        let update: UpdatePostRequest =
            serde_json::from_value(serde_json::json!({"status":1,"markdowncontent":"# 新正文"}))
                .unwrap();
        let published = PostService::update_post(&db, user.id, &created.uuid, update)
            .await
            .unwrap();
        assert!(published.is_publish);
        assert!(published.publish_time > old_date);
        assert_eq!(published.size as usize, "# 新正文".len());
    }

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
        crate::test_helpers::setup_test_db().await
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

    // 测试修改文章
    #[tokio::test]
    async fn test_update_post() {
        // 初始化测试数据库连接
        let db = setup_test_db().await;

        // 创建测试用户
        let user = create_test_user(&db).await;

        // 创建测试分类
        let category = create_test_category(&db).await;

        // 创建测试标签
        let tag1 = create_test_tag(&db, "Rust".to_string()).await;
        let tag2 = create_test_tag(&db, "Web开发".to_string()).await;

        // 先创建一篇文章
        let create_post_request = CreatePostRequest {
            title: "原始文章标题".to_string(),
            summary: Some("这是原始文章的摘要".to_string()),
            content: "这是原始文章的内容。".to_string(),
            markdowncontent: "# 原始文章\n\n这是原始文章的内容。".to_string(),
            cover_image: Some("https://example.com/original.jpg".to_string()),
            category_id: category.id,
            tag_ids: vec![tag1.id],
            status: 1, // 发布状态
            featured: false,
        };

        // 调用创建文章的服务方法
        let result = PostService::create_post(&db, user.id, create_post_request).await;
        assert!(result.is_ok(), "创建文章应该成功");
        let original_post = result.unwrap();

        // 创建新的标签用于更新
        let tag3 = create_test_tag(&db, "更新".to_string()).await;
        let new_category = create_test_category(&db).await;

        // 构建更新文章的请求
        let update_post_request = UpdatePostRequest {
            title: Some("更新后的文章标题".to_string()),
            summary: Some("这是更新后的文章摘要".to_string()),
            content: Some("这是更新后的文章内容，比原来更长更详细。".to_string()),
            markdowncontent: Some(
                "# 更新后的文章\n\n这是更新后的文章内容，比原来更长更详细。".to_string(),
            ),
            cover_image: Some("https://example.com/updated.jpg".to_string()),
            category_id: Some(new_category.id),
            tag_ids: Some(vec![tag2.id, tag3.id]),
            status: Some(1),      // 保持发布状态
            featured: Some(true), // 设置为置顶
        };

        // 调用更新文章的服务方法
        let update_result =
            PostService::update_post(&db, user.id, &original_post.uuid, update_post_request).await;
        assert!(update_result.is_ok(), "更新文章应该成功");
        let updated_post = update_result.unwrap();

        // 验证文章ID和UUID保持不变
        assert_eq!(updated_post.id, original_post.id);
        assert_eq!(updated_post.uuid, original_post.uuid);

        // 验证文章基本信息已更新
        assert_eq!(updated_post.title, "更新后的文章标题");
        assert_eq!(updated_post.description, "这是更新后的文章摘要");
        assert_eq!(updated_post.cover, "https://example.com/updated.jpg");
        assert!(updated_post.is_top); // 现在是置顶文章
        assert!(updated_post.is_publish); // 仍然是发布状态

        // 验证更新时间已改变
        assert!(updated_post.update_time > original_post.update_time);

        // 验证分类已更新
        assert!(updated_post.category.is_some());
        let category_name = &updated_post.category.as_ref().unwrap().name;
        assert!(category_name.starts_with("测试分类-"));
        assert_ne!(
            category_name,
            &original_post.category.as_ref().unwrap().name
        );

        // 验证标签已更新
        assert_eq!(updated_post.tags.len(), 2);
        let tag_names: Vec<String> = updated_post.tags.iter().map(|t| t.name.clone()).collect();
        assert!(tag_names.iter().any(|name| name.starts_with("Web开发-")));
        assert!(tag_names.iter().any(|name| name.starts_with("更新-")));

        // 清理测试数据
        cleanup_test_data(&db, updated_post.id).await;
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
    async fn view_fixture(db: &DatabaseConnection) -> String {
        let user = create_test_user(db).await;
        let category = create_test_category(db).await;
        PostService::create_post(
            db,
            user.id,
            CreatePostRequest {
                title: "Concurrent view fixture".into(),
                summary: None,
                content: "<p>Test</p>".into(),
                markdowncontent: "## Test".into(),
                cover_image: None,
                category_id: category.id,
                tag_ids: vec![],
                status: 1,
                featured: false,
            },
        )
        .await
        .unwrap()
        .uuid
    }

    #[tokio::test]
    async fn test_concurrent_views_are_successful_and_count_once() {
        use sea_orm::ConnectionTrait;
        let db = setup_test_db().await;
        let uuid = view_fixture(&db).await;
        let mut tasks = Vec::new();
        for _ in 0..40 {
            let db = db.clone();
            let uuid = uuid.clone();
            tasks.push(tokio::spawn(async move {
                PostService::increment_view(&db, &uuid, "198.51.100.1").await
            }));
        }
        for task in tasks {
            task.await.unwrap().unwrap();
        }
        let post = posts::Entity::find_by_uuid(&uuid)
            .one(&db)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(post.view_count, 1);
        let row = db
            .query_one(sea_orm::Statement::from_string(
                db.get_database_backend(),
                "SELECT COUNT(*) AS total FROM post_views".to_owned(),
            ))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(row.try_get::<i64>("", "total").unwrap(), 1);
        PostService::delete_post(&db, post.author_id, &uuid)
            .await
            .unwrap();
        assert!(
            posts::Entity::find_by_uuid(&uuid)
                .one(&db)
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn test_failed_counter_update_rolls_back_visit_and_can_retry() {
        use sea_orm::ConnectionTrait;
        let db = setup_test_db().await;
        let uuid = view_fixture(&db).await;
        for sql in [
            "CREATE OR REPLACE FUNCTION test_reject_view() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'isolated test failure'; END $$",
            "CREATE TRIGGER test_reject_view BEFORE UPDATE OF view_count ON posts FOR EACH ROW EXECUTE FUNCTION test_reject_view()",
        ] {
            db.execute(sea_orm::Statement::from_string(
                db.get_database_backend(),
                sql.to_owned(),
            ))
            .await
            .unwrap();
        }
        assert!(
            PostService::increment_view(&db, &uuid, "198.51.100.1")
                .await
                .is_err()
        );
        let row = db
            .query_one(sea_orm::Statement::from_string(
                db.get_database_backend(),
                "SELECT COUNT(*) AS total FROM post_views".to_owned(),
            ))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(row.try_get::<i64>("", "total").unwrap(), 0);
        db.execute(sea_orm::Statement::from_string(
            db.get_database_backend(),
            "DROP TRIGGER test_reject_view ON posts".to_owned(),
        ))
        .await
        .unwrap();
        PostService::increment_view(&db, &uuid, "198.51.100.1")
            .await
            .unwrap();
        let post = posts::Entity::find_by_uuid(&uuid)
            .one(&db)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(post.view_count, 1);
    }
    #[tokio::test]
    async fn test_equal_timestamps_have_stable_pagination() {
        use actix_web::{App, test, web};
        let db = setup_test_db().await;
        let uuid = view_fixture(&db).await;
        let base = posts::Entity::find_by_uuid(&uuid)
            .one(&db)
            .await
            .unwrap()
            .unwrap();
        let records = (0..210).map(|index| posts::ActiveModel {
            uuid: Set(Uuid::new_v4().to_string()),
            author_id: Set(base.author_id),
            category_id: Set(base.category_id),
            title: Set(format!("Tie {index}")),
            content: Set("HTML".into()),
            markdowncontent: Set("Markdown".into()),
            status: Set(1),
            featured: Set(false),
            view_count: Set(0),
            size: Set(4),
            created_at: Set(base.created_at),
            updated_at: Set(base.updated_at),
            published_at: Set(base.published_at),
            ..Default::default()
        });
        posts::Entity::insert_many(records).exec(&db).await.unwrap();
        let app = test::init_service(App::new().app_data(web::Data::new(db)).route(
            "/posts",
            web::get().to(crate::handlers::posts::get_posts_all_handler),
        ))
        .await;
        let mut ids = Vec::new();
        for page in 1..=3 {
            let response = test::call_service(
                &app,
                test::TestRequest::get()
                    .uri(&format!("/posts?page={page}&limit=100"))
                    .to_request(),
            )
            .await;
            assert!(response.status().is_success());
            let data: serde_json::Value = test::read_body_json(response).await;
            ids.extend(
                data["data"]["data"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|post| post["id"].as_i64().unwrap()),
            );
        }
        assert_eq!(ids.len(), 211);
        assert_eq!(
            ids.iter().collect::<std::collections::HashSet<_>>().len(),
            211
        );
        assert!(ids.windows(2).all(|pair| pair[0] > pair[1]));
    }
}
