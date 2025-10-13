use crate::config::AppError;
use crate::dto::PaginatedResp;
use crate::dto::PaginationQuery;
use crate::dto::common::Pagination;
use crate::dto::posts::CategoryResponse;
use crate::dto::posts::PostListResponse;
use crate::dto::posts::TagResponse;
use crate::models::{categories, post_tags, posts, tags};
use crate::{ApiResponse, HttpResult, RouteInfo};
use actix_web::web;
use route_macros::crud_entity;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
};

crud_entity!({
    entity : posts,
    route_prefix:"/api/v1/posts",
    permission_prefix: "posts",
    id_type:"uuid",
    operations: ["read"],
    create_request_type: CreateTagRequest
});

pub async fn get_posts_all_handler(
    db_pool: web::Data<DatabaseConnection>,
    query: web::Query<PaginationQuery>,
) -> HttpResult {
    let PaginationQuery { page, limit } = query.into_inner();
    // 1. 首先查询文章和分类（一对多关系）
    let paginator = posts::Entity::find()
        .find_also_related(categories::Entity)
        .order_by_desc(posts::Column::CreatedAt)
        .paginate(db_pool.as_ref(), limit);
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
    log::info!("total:{}", total);
    let posts_with_categories = match paginator.fetch_page(page).await {
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
            .await
            .unwrap_or_default();

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
    // 4. 构建最终的响应数据
    let data: Vec<PostListResponse> = posts_with_categories
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
            let author = "朝阳".to_string(); // 您需要从用户表查询真实作者名
            // 构建响应对象
            PostListResponse {
                id: post.id,
                uuid: post.uuid,
                title: post.title,
                content: post.content,
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
    let resp = PaginatedResp {
        data,
        pagination: Pagination { total, page, limit },
    };
    Ok(ApiResponse::success(resp, "成功").to_http_response())
}
