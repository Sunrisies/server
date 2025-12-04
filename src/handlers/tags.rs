use crate::config::AppError;
use crate::dto::PaginationQuery;
use crate::dto::posts::PostResponse;
use crate::dto::tag::{CreateTagRequest, TagCloudItem};
use crate::dto::{PaginatedResp, Pagination};
use crate::models::{post_tags, posts, tags};
use crate::{ApiResponse, HttpResult, RouteInfo, utils::db_err_map};
use actix_web::{HttpResponse, web};
use route_macros::crud_entity;
use sea_orm::prelude::Expr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, RelationTrait,
};

crud_entity!({
    entity : tags,
    route_prefix:"/api/tags",
    permission_prefix: "tags",
    id_type:"id",
    operations: ["create","list","delete","read"],
    create_request_type: CreateTagRequest
});

pub async fn get_tags_with_count_handler(db_pool: web::Data<DatabaseConnection>) -> HttpResult {
    // 构建查询：统计每个标签的使用次数（只统计已发布的文章）
    let tag_counts = tags::Entity::find()
        .column_as(tags::Column::Id, "id")
        .column_as(tags::Column::Name, "name")
        .column_as(
            Expr::col((post_tags::Entity, post_tags::Column::PostId)).count_distinct(),
            "count",
        )
        .join(
            sea_orm::JoinType::LeftJoin,
            post_tags::Relation::Tags.def().rev(), // 使用关系的反向
        )
        .join(
            sea_orm::JoinType::LeftJoin,
            posts::Relation::PostTags.def().rev(), // 使用关系的反向
        )
        .group_by(tags::Column::Id)
        .group_by(tags::Column::Name)
        .order_by_desc(Expr::cust("count"))
        .into_model::<TagCloudItem>()
        .all(db_pool.as_ref())
        .await?;
    Ok(ApiResponse::success(tag_counts, "成功").to_http_response())
}
/// 通过tag获取文章列表
pub async fn get_posts_by_tag_handler(
    db_pool: web::Data<DatabaseConnection>,
    path: web::Path<i32>, // 标签ID
    query: web::Query<PaginationQuery>,
) -> HttpResult {
    let tag_id = path.into_inner();
    let PaginationQuery { page, limit } = query.into_inner();

    // 1. 查询标签是否存在
    let tag = tags::Entity::find_by_id(tag_id)
        .one(db_pool.as_ref())
        .await
        .map_err(|e| {
            log::error!("查询标签失败: {}", e);
            AppError::DatabaseConnectionError("查询失败".to_string())
        })?;

    let _tag = match tag {
        Some(tag) => tag,
        None => {
            return Err(AppError::NotFound("标签不存在".to_string()));
        }
    };

    // 2. 查询该标签下的所有文章
    let paginator = post_tags::Entity::find()
        .filter(post_tags::Column::TagId.eq(tag_id))
        .find_also_related(posts::Entity)
        // .filter(posts::Column::Status.eq(1)) // 只查询已发布的文章
        .order_by_desc(posts::Column::PublishedAt)
        // .into_tuple::<(post_tags::Model, Option<posts::Model>)>()
        .paginate(db_pool.as_ref(), limit);

    let total = paginator.num_items().await?;
    let posts_with_relations = paginator.fetch_page(page - 1).await?;
    log::info!(
        "total: {}, page: {}, limit: {}, posts_with_relations: {:?}",
        total,
        page,
        limit,
        posts_with_relations
    );
    // 3. 构建响应数据
    let posts: Vec<PostResponse> = posts_with_relations
        .into_iter()
        .filter_map(|(_post_tag, post_option)| post_option) // 只保留存在的文章
        .map(|post| {
            let category = None; // 这里可以根据需要查询分类
            let tags = vec![]; // 这里可以根据需要查询标签
            let author = "朝阳".to_string(); // 这里需要从用户表查询真实作者名

            PostResponse {
                id: post.id,
                uuid: post.uuid,
                title: post.title,
                // content: post.markdowncontent,
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
            }
        })
        .collect();

    let response = PaginatedResp {
        data: posts,
        pagination: Pagination { total, page, limit },
    };
    // 返回一个错误的
    Ok(ApiResponse::success(response, "成功").to_http_response())
}
