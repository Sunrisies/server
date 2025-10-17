use crate::config::AppError;
use crate::dto::PaginatedResp;
use crate::dto::PaginationQuery;
use crate::dto::common::Pagination;
use crate::dto::posts::CategoryResponse;
use crate::dto::posts::PostListResponse;
use crate::dto::posts::PostResponse;
use crate::dto::posts::TagResponse;
use crate::models::{categories, post_tags, posts, tags};
use crate::{ApiResponse, HttpResult};
use actix_web::web;
use sea_orm::FromQueryResult;
use sea_orm::Order;
use sea_orm::sea_query::Alias;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, prelude::Expr,
};
use serde::Serialize;
use tokio::try_join;

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
            let author = "朝阳".to_string(); // 您需要从用户表查询真实作者名
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

    let resp = PaginatedResp {
        data,
        pagination: Pagination { total, page, limit },
    };
    Ok(ApiResponse::success(resp, "成功").to_http_response())
}

/// 现在请根据文章的创建时间返回时间轴,
pub async fn get_timeline_handler(db_pool: web::Data<DatabaseConnection>) -> HttpResult {
    #[derive(Debug, FromQueryResult)]
    struct TimelineCount {
        date: chrono::NaiveDate,
        count: i64,
    }
    let timeline_data = posts::Entity::find()
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
        .await
        .unwrap_or_default();

    let resp = timeline_data
        .into_iter()
        .map(|item| (item.date.format("%Y-%m-%d").to_string(), item.count))
        .collect::<Vec<_>>();

    Ok(ApiResponse::success(resp, "成功").to_http_response())
}

pub async fn get_posts_handler(
    db_pool: web::Data<DatabaseConnection>,
    page: web::Path<String>,
) -> HttpResult {
    let uuid = page.into_inner();

    // 1. 查询文章和分类（通过 UUID）
    let post_with_category = posts::Entity::find_by_uuid(&uuid)
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
        .await
        .unwrap_or_default();

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

    let author = "朝阳".to_string(); // 这里需要从用户表查询真实作者名

    let response = PostListResponse {
        id: post.id,
        uuid: post.uuid,
        title: post.title,
        content: post.markdowncontent,
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
/// prevNext 或者文章的上一篇跟下一篇
pub async fn get_prev_next_handler(
    db_pool: web::Data<DatabaseConnection>,
    page: web::Path<String>,
) -> HttpResult {
    #[derive(Debug, FromQueryResult, Serialize)]
    struct PrevNextResponse {
        title: String,
        uuid: String,
    }

    #[derive(Debug, Serialize)]
    struct PrevNextResult {
        #[serde(rename = "prevArticle")]
        prev_article: Option<PrevNextResponse>,
        #[serde(rename = "nextArticle")]
        next_article: Option<PrevNextResponse>,
    }

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
        .one(db_pool.as_ref())
        .await?
        .ok_or_else(|| AppError::NotFound("文章不存在".to_string()))?;

    // 并行查询上一篇和下一篇文章
    let (prev, next) = try_join!(
        // 查询上一篇（创建时间更早的）
        posts::Entity::find()
            .filter(posts::Column::CreatedAt.lt(post.created_at))
            .order_by(posts::Column::CreatedAt, Order::Desc)
            .one(db_pool.as_ref()),
        // 查询下一篇（创建时间更晚的）
        posts::Entity::find()
            .filter(posts::Column::CreatedAt.gt(post.created_at))
            .order_by(posts::Column::CreatedAt, Order::Asc)
            .one(db_pool.as_ref())
    )
    .map_err(|e| {
        log::error!("数据库操作失败: {}", e);
        AppError::DatabaseError("服务器异常，请联系管理员".to_string())
    })?;
    log::info!("prev: {:?}, next: {:?}", prev, next);
    Ok(ApiResponse::success(
        PrevNextResult {
            prev_article: prev.map(to_response),
            next_article: next.map(to_response),
        },
        "成功",
    )
    .to_http_response())
}
