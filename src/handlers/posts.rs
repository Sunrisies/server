use actix_web::{HttpMessage, HttpRequest, web};

use crate::EmptyResponse;
use crate::config::AppError;
use crate::dto::PaginatedResp;
use crate::dto::PaginationQuery;
use crate::dto::common::Pagination;
use crate::dto::posts::PrevNextResponse;
use crate::dto::posts::PrevNextResult;
use crate::dto::posts::TimelineResponse;
use crate::dto::posts::UpdatePostRequest;
use crate::dto::posts::{
    CategoryResponse, CreatePostRequest, PostListResponse, PostResponse, TagResponse,
};
use crate::dto::user::ValidationErrorJson;
use crate::models::{categories, post_tags, posts, tags};
use crate::utils::jwt::TokenClaims;
use crate::{ApiResponse, HttpResult};

/// 从请求中获取当前用户ID。如果未登录（游客访问），返回 None。
async fn current_user_id(req: &HttpRequest, db: &DatabaseConnection) -> Result<i32, AppError> {
    let uuid = req
        .extensions()
        .get::<TokenClaims>()
        .map(|claims| claims.user_uuid.clone())
        .ok_or_else(|| AppError::Unauthorized("请先登录".into()))?;
    crate::models::users::Entity::find_by_uuid(&uuid)
        .one(db)
        .await
        .map_err(|_| AppError::DatabaseError("用户查询失败".into()))?
        .map(|user| user.id)
        .ok_or_else(|| AppError::Unauthorized("用户不存在".into()))
}
use sea_orm::RelationTrait;
use sea_orm::{
    ColumnTrait,
    DatabaseConnection,
    EntityTrait,
    FromQueryResult,
    Order,
    PaginatorTrait,
    QueryFilter,
    QueryOrder,
    QuerySelect,
    //   ActiveModelTrait,Set, TransactionTrait,
    prelude::Expr,
    sea_query::Alias,
};
use tokio::try_join;
use validator::Validate;

/// 获取所有文章列表
///
/// 返回分页的文章列表，支持按分类和标签过滤
#[utoipa::path(
    summary = "获取文章列表",
    tag="文章",
    description = "获取文章列表接口",
    get,
    path = "/api/v1/posts",
    params(
        ("page" = u64, Query, description = "页码",example = 1),
        ("limit" = u64, Query, description = "每页数量",example = 10),
        ("category" = Option<i32>, Query, description = "分类ID",nullable  = false),
        ("tag" = Option<i32>, Query, description = "标签ID",nullable=false)
    ),
    responses(
        (status = 200, description = "成功获取文章列表", body = PaginatedResp<PostResponse>),
        (status = 500, description = "服务器内部错误", body = ApiResponse<ValidationErrorJson>)
    ),
)]
pub async fn get_posts_all_handler(
    req: HttpRequest,
    db_pool: web::Data<DatabaseConnection>,
    query: web::Query<PaginationQuery>,
) -> HttpResult {
    query
        .validate()
        .map_err(|e| AppError::ValidationError(ValidationErrorJson::from_validation_errors(&e)))?;
    let PaginationQuery {
        page,
        limit,
        category,
        tag,
    } = query.into_inner();
    // 1. 构建基础查询（文章 + 分类）
    let mut query_builder = posts::Entity::find().find_also_related(categories::Entity);

    if req
        .extensions()
        .get::<crate::middleware::auth::AdminAccess>()
        .is_none()
    {
        query_builder = query_builder.filter(posts::Column::Status.eq(1));
    }
    // 新增：如果传入了分类ID，则添加过滤条件
    if let Some(cat_id) = category {
        // 假设 category 是 i32 类型（分类ID），且 posts 表有 category_id 字段
        query_builder = query_builder.filter(posts::Column::CategoryId.eq(cat_id));
    }
    // 按标签过滤（通过中间表 post_tags）
    if let Some(tag_id) = tag {
        // 使用 inner join 关联 post_tags 表，并过滤 tag_id
        query_builder = query_builder
            .join(
                sea_orm::JoinType::InnerJoin,
                posts::Relation::PostTags.def(),
            )
            .filter(post_tags::Column::TagId.eq(tag_id))
            .distinct(); // 避免因一篇文章有多个标签而产生重复
    }
    // 按创建时间倒序
    let query_builder = query_builder
        .order_by_desc(posts::Column::CreatedAt)
        .order_by_desc(posts::Column::Id);
    // 分页查询（总数会自动应用过滤条件，无需额外处理）
    let paginator = query_builder.paginate(db_pool.as_ref(), limit);
    let total = match paginator.num_items().await {
        Ok(t) => t,
        Err(e) => {
            log::error!("查询文章总数失败: {}", e);
            return Ok(ApiResponse::from(AppError::DatabaseConnectionError(
                "获取失败".to_string(),
            ))
            .to_http_response());
        }
    };
    let posts_with_categories = match paginator.fetch_page(page - 1).await {
        Ok(list) => list,
        Err(e) => {
            log::error!("查询文章列表失败: {}", e);
            return Ok(ApiResponse::from(AppError::DatabaseConnectionError(
                "获取列表失败".to_string(),
            ))
            .to_http_response());
        }
    };

    // 2. 收集文章ID用于批量查询标签
    let post_ids: Vec<i32> = posts_with_categories
        .iter()
        .map(|(post, _)| post.id)
        .collect();

    // 3. 批量查询所有文章的标签（避免N+1查询）
    let post_tags_map = if post_ids.is_empty() {
        std::collections::HashMap::new()
    } else {
        // 通过中间表查询标签
        let tag_relations = post_tags::Entity::find()
            .filter(post_tags::Column::PostId.is_in(post_ids.clone()))
            .find_also_related(tags::Entity)
            .all(db_pool.as_ref())
            .await?;

        let mut map = std::collections::HashMap::new();
        for (post_tag, tag_option) in tag_relations {
            if let Some(tag) = tag_option {
                map.entry(post_tag.post_id)
                    .or_insert_with(Vec::new)
                    .push(TagResponse {
                        id: tag.id,
                        name: tag.name,
                    });
            }
        }
        map
    };
    let author_ids: Vec<i32> = posts_with_categories
        .iter()
        .map(|(post, _)| post.author_id)
        .collect();
    let authors: std::collections::HashMap<i32, String> = crate::models::users::Entity::find()
        .filter(crate::models::users::Column::Id.is_in(author_ids))
        .all(db_pool.get_ref())
        .await?
        .into_iter()
        .map(|user| (user.id, user.user_name))
        .collect();
    // 4. 构建最终的响应数据
    let data: Vec<PostResponse> = posts_with_categories
        .into_iter()
        .map(|(post, category_option)| {
            // 处理分类信息
            let category = category_option.map(|category| CategoryResponse {
                id: category.id,
                name: category.name,
            });

            // 获取标签信息
            let tags = post_tags_map.get(&post.id).cloned().unwrap_or_default();

            // 构建作者信息（这里需要根据您的用户表结构调整）
            let author = authors.get(&post.author_id).cloned().unwrap_or_default();
            // 构建响应对象
            PostResponse {
                id: post.id,
                uuid: post.uuid,
                title: post.title,
                cover: post.cover_image.unwrap_or_default(),
                author,
                publish_time: post.published_at,
                update_time: post.updated_at,
                views: post.view_count,
                is_top: post.featured,
                is_publish: post.status == 1,
                is_hide: post.status == 2,
                description: post.summary.unwrap_or_default(),
                size: post.size, // 可以根据内容长度计算
                category,
                tags,
            }
        })
        .collect();

    let resp: PaginatedResp<PostResponse> = PaginatedResp {
        data,
        pagination: Pagination { total, page, limit },
    };
    Ok(ApiResponse::success(resp, "成功").to_http_response())
}

/// 为时间轴处理函数添加OpenAPI文档
#[utoipa::path(
    summary = "获取文章时间轴",
    tag="文章",
    description = "根据文章的创建时间返回时间轴",
    get,
    path = "/api/v1/posts/uploadTime",
    responses(
        (status = 200, description = "成功获取文章时间轴", body = ApiResponse<Vec<TimelineResponse>>),
        (status = 500, description = "服务器内部错误", body = ApiResponse<ValidationErrorJson>)
    ),
)]
pub async fn get_timeline_handler(db_pool: web::Data<DatabaseConnection>) -> HttpResult {
    #[derive(Debug, FromQueryResult)]
    struct TimelineCount {
        date: chrono::NaiveDate,
        count: i64,
    }
    let timeline_data = posts::Entity::find()
        .filter(posts::Column::Status.eq(1))
        .select_only()
        .column_as(
            Expr::col(posts::Column::CreatedAt).cast_as(sea_orm::sea_query::Alias::new("date")),
            "date",
        )
        .column_as(Expr::col(posts::Column::Id).count(), "count")
        .group_by(Expr::col(posts::Column::CreatedAt).cast_as(Alias::new("date")))
        .order_by_desc(Expr::col(posts::Column::CreatedAt).cast_as(Alias::new("date")))
        .into_model::<TimelineCount>()
        .all(db_pool.as_ref())
        .await?;

    let resp = timeline_data
        .into_iter()
        .map(|item| (item.date.format("%Y-%m-%d").to_string(), item.count))
        .collect::<Vec<_>>();

    Ok(ApiResponse::success(resp, "成功").to_http_response())
}

// 为获取文章详情的处理函数添加OpenAPI文档
#[utoipa::path(
    summary = "获取文章详情",
    tag="文章",
    description = "根据UUID获取文章详情",
    get,
    path = "/api/v1/posts/{uuid}",
    params(
        ("uuid" = String, Path, description = "文章UUID")
    ),
    responses(
        (status = 200, description = "成功获取文章详情", body = ApiResponse<PostListResponse>),
        (status = 404, description = "文章不存在", body = ApiResponse<ValidationErrorJson>),
        (status = 500, description = "服务器内部错误", body = ApiResponse<ValidationErrorJson>)
    ),
)]
pub async fn get_posts_handler(
    req: HttpRequest,
    db_pool: web::Data<DatabaseConnection>,
    page: web::Path<String>,
) -> HttpResult {
    let uuid = page.into_inner();

    // 1. 查询文章和分类（通过 UUID）
    let mut query = posts::Entity::find_by_uuid(&uuid);
    if req
        .extensions()
        .get::<crate::middleware::auth::AdminAccess>()
        .is_none()
    {
        query = query.filter(posts::Column::Status.eq(1));
    }
    let post_with_category = query
        .find_also_related(categories::Entity)
        .one(db_pool.as_ref())
        .await
        .map_err(|e| {
            log::error!("查询文章详情失败: {}", e);
            AppError::DatabaseConnectionError("查询失败".to_string())
        })?;

    // 2. 检查文章是否存在
    let (post, category_option) = match post_with_category {
        Some(data) => data,
        None => {
            return Err(AppError::NotFound(String::from("文章不存在")));
        }
    };

    // 3. 查询该文章的标签
    let tag_relations = post_tags::Entity::find()
        .filter(post_tags::Column::PostId.eq(post.id))
        .find_also_related(tags::Entity)
        .all(db_pool.as_ref())
        .await?;

    let tags: Vec<TagResponse> = tag_relations
        .into_iter()
        .filter_map(|(_, tag_option)| tag_option)
        .map(|tag| TagResponse {
            id: tag.id,
            name: tag.name,
        })
        .collect();

    // 4. 构建响应数据
    let category = category_option.map(|category| CategoryResponse {
        id: category.id,
        name: category.name,
    });

    let author = crate::models::users::Entity::find_by_id(post.author_id)
        .one(db_pool.get_ref())
        .await?
        .map(|user| user.user_name)
        .unwrap_or_default();

    let response = PostListResponse {
        id: post.id,
        uuid: post.uuid,
        title: post.title,
        content: post.markdowncontent.clone(),
        htmlcontent: req
            .extensions()
            .get::<crate::middleware::auth::AdminAccess>()
            .map(|_| post.content),
        markdowncontent: req
            .extensions()
            .get::<crate::middleware::auth::AdminAccess>()
            .map(|_| post.markdowncontent),
        cover: post.cover_image.unwrap_or_default(),
        author,
        publish_time: post.published_at,
        update_time: post.updated_at,
        views: post.view_count,
        is_top: post.featured,
        is_publish: post.status == 1,
        is_hide: post.status == 2,
        description: post.summary.unwrap_or_default(),
        size: post.size,
        category,
        tags,
    };

    Ok(ApiResponse::success(response, "成功").to_http_response())
}

/// `为获取上一篇和下一篇文章的处理函数添加OpenAPI文档`
#[utoipa::path(
    summary = "获取上一篇和下一篇文章",
    tag="文章",
    description = "根据当前文章的UUID获取上一篇和下一篇文章的信息",
    get,
    path = "/api/v1/posts/{uuid}/prevnext",
    params(
        ("uuid" = String, Path, description = "当前文章UUID")
    ),
    responses(
        (status = 200, description = "成功获取上一篇和下一篇文章", body = ApiResponse<PrevNextResult>),
        (status = 404, description = "文章不存在", body = ApiResponse<ValidationErrorJson>),
        (status = 500, description = "服务器内部错误", body = ApiResponse<ValidationErrorJson>)
    ),
)]
pub async fn get_prev_next_handler(
    db_pool: web::Data<DatabaseConnection>,
    page: web::Path<String>,
) -> HttpResult {
    // 将转换函数提取出来，避免重复
    fn to_response(post: posts::Model) -> PrevNextResponse {
        PrevNextResponse {
            uuid: post.uuid,
            title: post.title,
        }
    }

    let uuid = page.into_inner();
    // 查询当前文章，使用更简洁的错误处理
    let post = posts::Entity::find_by_uuid(&uuid)
        .filter(posts::Column::Status.eq(1))
        .one(db_pool.as_ref())
        .await?
        .ok_or_else(|| AppError::NotFound("文章不存在".to_string()))?;

    // 并行查询上一篇和下一篇文章
    let (prev, next) = try_join!(
        // 查询上一篇（创建时间更早的）
        posts::Entity::find()
            .filter(posts::Column::Status.eq(1))
            .filter(
                sea_orm::Condition::any()
                    .add(posts::Column::CreatedAt.lt(post.created_at))
                    .add(
                        sea_orm::Condition::all()
                            .add(posts::Column::CreatedAt.eq(post.created_at))
                            .add(posts::Column::Id.lt(post.id))
                    )
            )
            .order_by(posts::Column::CreatedAt, Order::Desc)
            .order_by(posts::Column::Id, Order::Desc)
            .one(db_pool.as_ref()),
        // 查询下一篇（创建时间更晚的）
        posts::Entity::find()
            .filter(posts::Column::Status.eq(1))
            .filter(
                sea_orm::Condition::any()
                    .add(posts::Column::CreatedAt.gt(post.created_at))
                    .add(
                        sea_orm::Condition::all()
                            .add(posts::Column::CreatedAt.eq(post.created_at))
                            .add(posts::Column::Id.gt(post.id))
                    )
            )
            .order_by(posts::Column::CreatedAt, Order::Asc)
            .order_by(posts::Column::Id, Order::Asc)
            .one(db_pool.as_ref())
    )
    .map_err(|e| {
        log::error!("数据库操作失败: {e}");
        AppError::DatabaseError("服务器异常，请联系管理员".to_string())
    })?;
    Ok(ApiResponse::success(
        PrevNextResult {
            prev_article: prev.map(to_response),
            next_article: next.map(to_response),
        },
        "成功",
    )
    .to_http_response())
}

/// `为创建文章的处理函数添加OpenAPI文档`
#[utoipa::path(
    summary = "创建文章",
    tag="文章",
    description = "创建新文章",
    post,
    path = "/api/v1/posts",
    request_body = CreatePostRequest,
    responses(
        (status = 200, description = "文章创建成功", body = ApiResponse<PostResponse>),
        (status = 400, description = "请求参数验证失败", body = ApiResponse<ValidationErrorJson>),
        (status = 500, description = "服务器内部错误", body = ApiResponse<ValidationErrorJson>)
    ),
)]
pub async fn create_post_handler(
    req: HttpRequest,
    db_pool: web::Data<DatabaseConnection>,
    post_data: web::Json<CreatePostRequest>,
) -> HttpResult {
    log::info!("create_post_handler: {post_data:?}");
    let user_id = current_user_id(&req, db_pool.get_ref()).await?;
    // 验证输入
    if let Err(validation_errors) = post_data.validate() {
        let msg = ValidationErrorJson::from_validation_errors(&validation_errors);
        return Ok(ApiResponse::from(AppError::ValidationError(msg)).to_http_response());
    }

    // 调用服务层创建文章
    match crate::services::posts::PostService::create_post(
        db_pool.as_ref(),
        user_id,
        post_data.into_inner(),
    )
    .await
    {
        Ok(post) => Ok(ApiResponse::success(post, "文章创建成功").to_http_response()),
        Err(e) => Ok(ApiResponse::from(e).to_http_response()),
    }
}

// 为更新文章的处理函数添加OpenAPI文档
#[utoipa::path(
    summary = "更新文章",
    tag="文章",
    description = "根据UUID更新现有文章",
    put,
    path = "/api/v1/posts/{uuid}",
    params(
        ("uuid" = String, Path, description = "文章UUID")
    ),
    request_body = UpdatePostRequest,
    responses(
        (status = 200, description = "文章更新成功", body = ApiResponse<PostResponse>),
        (status = 400, description = "请求参数验证失败", body = ApiResponse<ValidationErrorJson>),
        (status = 404, description = "文章不存在", body = ApiResponse<ValidationErrorJson>),
        (status = 500, description = "服务器内部错误", body = ApiResponse<ValidationErrorJson>)
    ),
)]
pub async fn update_post_handler(
    req: HttpRequest,
    db_pool: web::Data<DatabaseConnection>,
    path: web::Path<String>,
    post_data: web::Json<UpdatePostRequest>,
) -> HttpResult {
    let user_id = current_user_id(&req, db_pool.get_ref()).await?;
    // 验证输入
    if let Err(validation_errors) = post_data.validate() {
        let msg = ValidationErrorJson::from_validation_errors(&validation_errors);
        return Ok(ApiResponse::from(AppError::ValidationError(msg)).to_http_response());
    }

    let uuid = path.into_inner();

    // 调用服务层更新文章
    match crate::services::posts::PostService::update_post(
        db_pool.as_ref(),
        user_id,
        &uuid,
        post_data.into_inner(),
    )
    .await
    {
        Ok(post) => Ok(ApiResponse::success(post, "文章更新成功").to_http_response()),
        Err(e) => Ok(ApiResponse::from(e).to_http_response()),
    }
}

// 为删除文章的处理函数添加OpenAPI文档
#[utoipa::path(
    summary = "删除文章",
    tag="文章",
    description = "根据UUID删除文章",
    delete,
    path = "/api/v1/posts/{uuid}",
    params(
        ("uuid" = String, Path, description = "文章UUID")
    ),
    responses(
        (status = 200, description = "文章删除成功", body = ApiResponse<EmptyResponse>),
        (status = 404, description = "文章不存在", body = ApiResponse<ValidationErrorJson>),
        (status = 500, description = "服务器内部错误", body = ApiResponse<ValidationErrorJson>)
    ),
)]
pub async fn delete_post_handler(
    req: HttpRequest,
    db_pool: web::Data<DatabaseConnection>,
    path: web::Path<String>,
) -> HttpResult {
    let user_id = current_user_id(&req, db_pool.get_ref()).await?;
    let uuid = path.into_inner();

    // 调用服务层删除文章
    match crate::services::posts::PostService::delete_post(db_pool.as_ref(), user_id, &uuid).await {
        Ok(()) => Ok(
            ApiResponse::<EmptyResponse>::success(EmptyResponse, "文章删除成功").to_http_response(),
        ),
        Err(e) => Ok(ApiResponse::from(e).to_http_response()),
    }
}

/// 增加文章阅读数
#[utoipa::path(
    post,
    path = "/api/v1/posts/{uuid}/view",
    tag = "文章",
    summary = "增加文章阅读数",
    description = "阅读文章时调用，view_count +1（前端需自行去重）",
    params(("uuid" = String, Path, description = "文章 UUID")),
    responses(
        (status = 200, description = "更新成功", body = crate::ApiResponse<crate::EmptyResponse>),
        (status = 404, description = "文章不存在"),
        (status = 500, description = "服务器内部错误"),
    ),
)]
pub async fn view_post_handler(
    req: HttpRequest,
    db_pool: web::Data<DatabaseConnection>,
    path: web::Path<String>,
) -> HttpResult {
    let uuid = path.into_inner();
    let ip = crate::utils::client_ip::client_ip(&req);
    match crate::services::posts::PostService::increment_view(db_pool.as_ref(), &uuid, &ip).await {
        Ok(_) => {
            Ok(ApiResponse::<EmptyResponse>::success(EmptyResponse, "成功").to_http_response())
        }
        Err(e) => Ok(ApiResponse::from(e).to_http_response()),
    }
}

#[cfg(test)]
mod auth_author_tests {
    use super::*;
    use actix_web::test;
    #[actix_web::test]
    async fn author_id_is_resolved_from_uuid_instead_of_constant() {
        let db = crate::services::auth_test::tests::setup_test_db().await;
        let notifier = web::Data::new(crate::SseNotifier::new());
        let mut last = None;
        for name in ["firstuser", "seconduser"] {
            last = Some(
                crate::services::auth::AuthService::register(
                    web::Data::new(db.clone()),
                    web::Json(crate::dto::user::RegisterResponse {
                        user_name: name.into(),
                        pass_word: "password123".into(),
                    }),
                    notifier.clone(),
                )
                .await
                .unwrap(),
            );
        }
        let user = last.unwrap();
        let req = test::TestRequest::default().to_http_request();
        req.extensions_mut().insert(TokenClaims {
            user_uuid: user.uuid,
            user_name: user.user_name,
            exp: i64::MAX,
            role_id: 1,
            jti: None,
        });
        assert_ne!(user.id, 1);
        assert_eq!(current_user_id(&req, &db).await.unwrap(), user.id);
        assert!(
            current_user_id(&test::TestRequest::default().to_http_request(), &db)
                .await
                .is_err()
        );
    }
    #[actix_web::test]
    async fn auth_public_posts_hide_drafts_and_admin_detail_preserves_sources() {
        use sea_orm::{ActiveModelTrait, ConnectionTrait, Schema, Set};
        let db = crate::services::auth_test::tests::setup_test_db().await;
        let schema = Schema::new(db.get_database_backend());
        for table in [
            schema.create_table_from_entity(categories::Entity),
            schema.create_table_from_entity(posts::Entity),
            schema.create_table_from_entity(tags::Entity),
            schema.create_table_from_entity(post_tags::Entity),
        ] {
            db.execute(db.get_database_backend().build(&table))
                .await
                .unwrap();
        }
        let now = chrono::Utc::now();
        let category = categories::ActiveModel {
            name: Set("Test".into()),
            slug: Set("test".into()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();
        let user = crate::services::auth::AuthService::register(
            web::Data::new(db.clone()),
            web::Json(crate::dto::user::RegisterResponse {
                user_name: "authorname".into(),
                pass_word: "password123".into(),
            }),
            web::Data::new(crate::SseNotifier::new()),
        )
        .await
        .unwrap();
        let tag = tags::ActiveModel {
            name: Set("Privacy tag".into()),
            created_at: Set(now),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();
        for (uuid, status) in [("published", 1), ("draft", 0), ("hidden", 2)] {
            let post = posts::ActiveModel {
                uuid: Set(uuid.into()),
                author_id: Set(user.id),
                category_id: Set(category.id),
                title: Set(uuid.into()),
                content: Set("<h1>HTML</h1>".into()),
                markdowncontent: Set("# Markdown".into()),
                status: Set(status),
                featured: Set(false),
                view_count: Set(0),
                size: Set(10),
                created_at: Set(now + chrono::Duration::seconds(i64::from(status))),
                updated_at: Set(now),
                published_at: Set(now),
                ..Default::default()
            }
            .insert(&db)
            .await
            .unwrap();
            post_tags::ActiveModel {
                post_id: Set(post.id),
                tag_id: Set(tag.id),
                ..Default::default()
            }
            .insert(&db)
            .await
            .unwrap();
        }
        assert!(matches!(
            crate::services::posts::PostService::increment_view(&db, "draft", "127.0.0.1").await,
            Err(AppError::NotFound(_))
        ));
        assert!(matches!(
            crate::handlers::management::delete_category(
                web::Data::new(db.clone()),
                web::Path::from(category.id)
            )
            .await,
            Err(AppError::Conflict(_))
        ));
        let tagged = crate::handlers::tags::get_posts_by_tag_handler(
            web::Data::new(db.clone()),
            web::Path::from(tag.id),
            web::Query(crate::dto::PaginationQuery {
                page: 1,
                limit: 100,
                category: None,
                tag: None,
            }),
        )
        .await
        .unwrap();
        let tagged: serde_json::Value =
            serde_json::from_slice(&actix_web::body::to_bytes(tagged.into_body()).await.unwrap())
                .unwrap();
        assert_eq!(tagged["data"]["pagination"]["total"], 1);
        assert_eq!(tagged["data"]["data"][0]["author"], "authorname");
        let counts = crate::handlers::tags::get_tags_with_count_handler(web::Data::new(db.clone()))
            .await
            .unwrap();
        let counts: serde_json::Value =
            serde_json::from_slice(&actix_web::body::to_bytes(counts.into_body()).await.unwrap())
                .unwrap();
        assert_eq!(counts["data"][0]["count"], 1);
        let app = actix_web::test::init_service(
            actix_web::App::new()
                .app_data(web::Data::new(db.clone()))
                .configure(crate::routes::posts::config_routes),
        )
        .await;
        let response = actix_web::test::call_service(
            &app,
            actix_web::test::TestRequest::get()
                .uri("/v1/posts?page=1&limit=100")
                .to_request(),
        )
        .await;
        let body: serde_json::Value = actix_web::test::read_body_json(response).await;
        assert_eq!(body["data"]["pagination"]["total"], 1);
        assert_eq!(body["data"]["data"][0]["author"], "authorname");
        let invalid = actix_web::test::call_service(
            &app,
            actix_web::test::TestRequest::get()
                .uri("/v1/posts?page=0&limit=10")
                .to_request(),
        )
        .await;
        assert_eq!(invalid.status(), 422);
        let neighbors = actix_web::test::call_service(
            &app,
            actix_web::test::TestRequest::get()
                .uri("/v1/posts/published/prevnext")
                .to_request(),
        )
        .await;
        assert_eq!(neighbors.status(), 200);
        let neighbors: serde_json::Value = actix_web::test::read_body_json(neighbors).await;
        assert!(neighbors["data"]["prev"].is_null());
        assert!(neighbors["data"]["next"].is_null());
        let draft = actix_web::test::call_service(
            &app,
            actix_web::test::TestRequest::get()
                .uri("/v1/posts/draft")
                .to_request(),
        )
        .await;
        assert_eq!(draft.status(), 404);
        let req = actix_web::test::TestRequest::default().to_http_request();
        req.extensions_mut()
            .insert(crate::middleware::auth::AdminAccess);
        let response = get_posts_handler(
            req,
            web::Data::new(db.clone()),
            web::Path::from("draft".to_string()),
        )
        .await
        .unwrap();
        let bytes = actix_web::body::to_bytes(response.into_body())
            .await
            .unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["data"]["htmlcontent"], "<h1>HTML</h1>");
        assert_eq!(body["data"]["markdowncontent"], "# Markdown");
        assert!(
            crate::handlers::category::categories_routes::get_categories_all(&db, 1, 0)
                .await
                .is_err()
        );
        let update = serde_json::from_value(serde_json::json!({"title":"changed"})).unwrap();
        assert!(matches!(
            crate::services::posts::PostService::update_post(&db, user.id + 1, "published", update)
                .await,
            Err(AppError::Forbidden(_))
        ));
        assert!(matches!(
            crate::services::posts::PostService::delete_post(&db, user.id + 1, "published").await,
            Err(AppError::Forbidden(_))
        ));
        crate::handlers::management::delete_tag(
            web::Data::new(db.clone()),
            web::Path::from(tag.id),
        )
        .await
        .unwrap();
        assert_eq!(post_tags::Entity::find().count(&db).await.unwrap(), 0);
        assert_eq!(posts::Entity::find().count(&db).await.unwrap(), 3);
        let req = actix_web::test::TestRequest::default().to_http_request();
        let response = get_posts_handler(
            req,
            web::Data::new(db),
            web::Path::from("published".to_string()),
        )
        .await
        .unwrap();
        let body: serde_json::Value = serde_json::from_slice(
            &actix_web::body::to_bytes(response.into_body())
                .await
                .unwrap(),
        )
        .unwrap();
        assert!(body["data"].get("htmlcontent").is_none());
    }
}
