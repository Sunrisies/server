use crate::config::AppError;
use crate::dto::PaginationQuery;
use crate::dto::posts::{CategoryResponse, PostResponse};
use crate::dto::tag::{CreateTagRequest, TagCloudItem};
use crate::dto::user::ValidationErrorJson;
use crate::dto::{PaginatedResp, Pagination};
use crate::models::tags::PostWithCategory;
use crate::models::{categories, post_tags, posts, tags};
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
    let tag_counts = tags::Entity::find()
        .column_as(tags::Column::Id, "id")
        .column_as(tags::Column::Name, "name")
        .column_as(
            Expr::col(post_tags::Column::PostId).count_distinct(),
            "count",
        )
        .join(sea_orm::JoinType::InnerJoin, tags::Relation::PostTags.def())
        .join(
            sea_orm::JoinType::InnerJoin,
            post_tags::Relation::Posts.def(),
        )
        .group_by(tags::Column::Id)
        .group_by(tags::Column::Name)
        .having(Expr::expr(Expr::col(post_tags::Column::PostId).count_distinct()).gt(0))
        .order_by_desc(Expr::expr(
            Expr::col(post_tags::Column::PostId).count_distinct(),
        ))
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
    let PaginationQuery { page, limit,.. } = query.into_inner();

    // 1. 查询标签是否存在
    if tags::Entity::find_by_id(tag_id)
        .one(db_pool.as_ref())
        .await
        .map_err(|e| {
            log::error!("查询标签失败: {}", e);
            AppError::DatabaseConnectionError("查询失败".to_string())
        })?
        .is_none()
    {
        return Err(AppError::NotFound("标签不存在".to_string()));
    }

    // 2. 查询该标签下的所有文章
    let posts = posts::Entity::find()
        .select_only()
        .column(posts::Column::Id)
        .column(posts::Column::Uuid)
        .column(posts::Column::Title)
        .column(posts::Column::CoverImage)
        .column(posts::Column::PublishedAt)
        .column(posts::Column::UpdatedAt)
        .column(posts::Column::ViewCount)
        .column(posts::Column::Featured)
        .column(posts::Column::Status)
        .column(posts::Column::Summary)
        .column(posts::Column::Size)
        .column_as(categories::Column::Id, "category_id")
        .column_as(categories::Column::Name, "category_name")
        .join(
            sea_orm::JoinType::InnerJoin,
            posts::Relation::PostTags.def(),
        )
        .filter(post_tags::Column::TagId.eq(tag_id))
        .join(
            sea_orm::JoinType::LeftJoin,
            posts::Relation::Categories.def(),
        )
        // .filter(posts::Column::Status.eq(1))
        .order_by_desc(posts::Column::PublishedAt)
        .into_model::<PostWithCategory>()
        .paginate(db_pool.as_ref(), limit);

    let total = posts.num_items().await?;
    log::info!("查询标签下的文章: {}", total);
    let posts_with_relations = posts.fetch_page(page - 1).await?;
    // 3. 构建响应数据
    let posts: Vec<PostResponse> = posts_with_relations
        .into_iter()
        .map(|post| PostResponse {
            id: post.id,
            uuid: post.uuid,
            title: post.title,
            cover: post.cover_image.unwrap_or_default(),
            author: "朝阳".to_string(),
            publish_time: post.published_at,
            update_time: post.updated_at,
            views: post.view_count,
            is_top: post.featured,
            is_publish: post.status == 1,
            is_hide: post.status == 2,
            description: post.summary.unwrap_or_default(),
            size: post.size,
            // category,
            category: post.category_id.map(|id| CategoryResponse {
                id,
                name: post.category_name.unwrap_or_default(),
            }),
            tags: vec![],
        })
        .collect();

    let response = PaginatedResp {
        data: posts,
        pagination: Pagination { total, page, limit },
    };
    // 返回一个错误的
    Ok(ApiResponse::success(response, "成功").to_http_response())
}
