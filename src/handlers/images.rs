use actix_multipart::Multipart;
use actix_web::web;
use sea_orm::DatabaseConnection;

use crate::{HttpResult, images::ImageService};

/// 处理图片上传
pub async fn upload_image_handler(
    db_pool: web::Data<DatabaseConnection>,
    payload: Multipart,
) -> HttpResult {
    let result = ImageService::handle_upload(&db_pool, payload).await?;
    Ok(result)
}

/// 获取图片列表
pub async fn get_images_handler(
    db_pool: web::Data<DatabaseConnection>,

    query: web::Query<crate::dto::common::PaginationQuery>,
) -> HttpResult {
    let result = ImageService::get_images(&db_pool, query.into_inner()).await?;
    Ok(result)
}

/// 根据ID获取图片
pub async fn get_image_by_id_handler(
    db_pool: web::Data<DatabaseConnection>,

    path: web::Path<i32>,
) -> HttpResult {
    let image_id = path.into_inner();
    let result = ImageService::get_image_by_id(&db_pool, image_id).await?;
    Ok(result)
}

/// 删除图片
pub async fn delete_image_handler(
    db_pool: web::Data<DatabaseConnection>,
    path: web::Path<i32>,
) -> HttpResult {
    let image_id = path.into_inner();
    let result = ImageService::delete_image(&db_pool, image_id).await?;
    Ok(result)
}
