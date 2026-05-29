use crate::dto::category::CreateCategoryRequest;
use crate::dto::user::ValidationErrorJson;
use crate::dto::{PaginatedResp, Pagination, PaginationQuery};
use crate::models::categories;
use crate::utils::db_err_map;
use crate::{ApiResponse, EmptyResponse, HttpResult, RouteInfo};
use actix_web::{HttpResponse, web};
use route_macros::crud_entity;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, ModelTrait, PaginatorTrait};
use validator::Validate;

crud_entity!({
    entity : categories,
    route_prefix:"/api/v1/categories",
    permission_prefix: "categories",
    id_type:"id",
    operations: ["create","list","delete","read"],
    create_request_type: CreateCategoryRequest,
    unique_field: Name,
    openapi_summary: "分类",
});
