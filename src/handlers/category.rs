use crate::config::AppError;
use crate::dto::PaginationQuery;
use crate::models::categories::Model as CategoryModel;
use crate::services::CategoryService;
use crate::{ApiResponse, HttpResult};
use actix_web::web;
use sea_orm::DatabaseConnection;
use serde::Deserialize;
// 创建分类的请求体
#[derive(Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
}

// 创建分类的处理器
pub async fn create_category(
    db_pool: web::Data<DatabaseConnection>,
    form: web::Json<CreateCategoryRequest>,
) -> HttpResult {
    // 将请求数据转换为实体模型
    let category_model = CategoryModel {
        id: 0, // ID 将由数据库自动生成
        name: form.name.clone(),
        slug: form.slug.clone(),
        description: form.description.clone(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // 调用服务创建分类
    match CategoryService::create(&db_pool, category_model).await {
        Ok(category) => Ok(ApiResponse::success(category, "添加成功").to_http_response()),
        Err(AppError::DatabaseConnectionError(msg)) => {
            // 统一包装：HTTP 200，业务码 200，message 提示不存在
            Ok(ApiResponse::<()>::success_msg(&msg).to_http_response())
        }
        Err(e) => {
            // 其他错误（数据库等）按原样返回 500/400 等
            Ok(ApiResponse::from(e).to_http_response())
        }
    }
    // 返回成功响应
    // macth/
    // Ok(ApiResponse::success(category, "添加成功").to_http_response())
}

// 获取所有分类的处理器
pub async fn get_categories(
    db_pool: web::Data<DatabaseConnection>,
    query: web::Query<PaginationQuery>,
) -> HttpResult {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(10);
    let categories = CategoryService::find_all(&db_pool, page, limit).await?;
    Ok(categories)
}

// 根据ID获取分类的处理器
pub async fn get_category_by_id(
    db_pool: web::Data<DatabaseConnection>,
    id: web::Path<i32>,
) -> HttpResult {
    match CategoryService::find_by_id(&db_pool, *id).await {
        Ok(category) => Ok(ApiResponse::success(category, "获取成功").to_http_response()),
        Err(AppError::NotFound(msg)) => {
            // 统一包装：HTTP 200，业务码 200，message 提示不存在
            Ok(ApiResponse::<()>::success_msg(&msg).to_http_response())
        }
        Err(e) => {
            // 其他错误（数据库等）按原样返回 500/400 等
            Ok(ApiResponse::from(e).to_http_response())
        }
    }
}

// // 更新分类的处理器
// pub async fn update_category(
//     state: web::Data<AppState>,
//     id: web::Path<i32>,
//     form: web::Json<CreateCategoryRequest>,
// ) -> Result<HttpResponse, crate::AppError> {
//     // 检查分类是否存在
//     let _ = CategoryService::find_by_id(&state.db, *id)
//         .await?
//         .ok_or_else(|| crate::AppError::NotFound("分类不存在".to_string()))?;

//     // 准备更新数据
//     let update_data = crate::entities::category::Model {
//         id: *id,
//         name: form.name.clone(),
//         slug: form.slug.clone(),
//         description: form.description.clone(),
//         created_at: chrono::Utc::now().naive_utc(), // 这个字段不会被更新
//         updated_at: chrono::Utc::now().naive_utc(),
//     };

//     // 调用服务更新分类
//     let updated_category = CategoryService::update(&state.db, *id, update_data).await?;

//     Ok(HttpResponse::Ok().json(updated_category))
// }

// 删除分类的处理器
pub async fn delete_category(
    db_pool: web::Data<DatabaseConnection>,
    id: web::Path<i32>,
) -> HttpResult {
    // 检查分类是否存在
    let category = match CategoryService::find_by_id(&db_pool, *id).await {
        Ok(c) => c,
        Err(e) => return Ok(ApiResponse::from(e).to_http_response()),
    };

    let res = CategoryService::delete(&category, &db_pool).await?;
    // 调用服务删除分类
    Ok(res)
}
