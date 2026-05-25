use actix_multipart::Multipart;
use actix_web::web;
use sea_orm::DatabaseConnection;

use crate::{HttpResult, images::ImageService};

#[utoipa::path(
    post,
    path = "/api/v1/images/upload",
    tag = "图片管理",
    summary = "上传图片",
    description = "上传图片并保存记录到数据库，返回图片信息",
    request_body(content = String, description = "图片文件（multipart/form-data）"),
    responses(
        (status = 200, description = "上传成功", body = crate::ApiResponse<crate::dto::image::ImageUploadResponse>),
        (status = 400, description = "请求参数错误", body = crate::ApiResponse<crate::EmptyResponse>),
        (status = 500, description = "服务器内部错误", body = crate::ApiResponse<crate::EmptyResponse>)
    )
)]
pub async fn upload_image_handler(
    db_pool: web::Data<DatabaseConnection>,
    payload: Multipart,
) -> HttpResult {
    let result = ImageService::handle_upload(&db_pool, payload).await?;
    Ok(result)
}

#[utoipa::path(
    get,
    path = "/api/v1/images",
    tag = "图片管理",
    summary = "获取图片列表",
    description = "分页获取已上传的图片列表",
    params(
        ("page" = Option<u64>, Query, description = "页码，默认1"),
        ("limit" = Option<u64>, Query, description = "每页数量，默认10")
    ),
    responses(
        (status = 200, description = "获取成功", body = crate::ApiResponse<Vec<crate::models::images::Model>>),
        (status = 500, description = "服务器内部错误", body = crate::ApiResponse<crate::EmptyResponse>)
    )
)]
pub async fn get_images_handler(
    db_pool: web::Data<DatabaseConnection>,

    query: web::Query<crate::dto::common::PaginationQuery>,
) -> HttpResult {
    let result = ImageService::get_images(&db_pool, query.into_inner()).await?;
    Ok(result)
}

#[utoipa::path(
    get,
    path = "/api/v1/images/{id}",
    tag = "图片管理",
    summary = "获取图片详情",
    description = "根据ID获取单张图片的详细信息",
    params(
        ("id" = i32, Path, description = "图片ID")
    ),
    responses(
        (status = 200, description = "获取成功", body = crate::ApiResponse<crate::models::images::Model>),
        (status = 404, description = "图片不存在", body = crate::ApiResponse<crate::EmptyResponse>),
        (status = 500, description = "服务器内部错误", body = crate::ApiResponse<crate::EmptyResponse>)
    )
)]
pub async fn get_image_by_id_handler(
    db_pool: web::Data<DatabaseConnection>,

    path: web::Path<i32>,
) -> HttpResult {
    let image_id = path.into_inner();
    let result = ImageService::get_image_by_id(&db_pool, image_id).await?;
    Ok(result)
}

#[utoipa::path(
    delete,
    path = "/api/v1/images/{id}",
    tag = "图片管理",
    summary = "删除图片",
    description = "根据ID删除已上传的图片",
    params(
        ("id" = i32, Path, description = "图片ID")
    ),
    responses(
        (status = 200, description = "删除成功", body = crate::ApiResponse<crate::EmptyResponse>),
        (status = 404, description = "图片不存在", body = crate::ApiResponse<crate::EmptyResponse>),
        (status = 500, description = "服务器内部错误", body = crate::ApiResponse<crate::EmptyResponse>)
    )
)]
pub async fn delete_image_handler(
    db_pool: web::Data<DatabaseConnection>,
    path: web::Path<i32>,
) -> HttpResult {
    let image_id = path.into_inner();
    let result = ImageService::delete_image(&db_pool, image_id).await?;
    Ok(result)
}
