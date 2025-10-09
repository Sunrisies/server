use crate::config::AppError;
use crate::dto::PaginatedResp;
use crate::dto::PaginationQuery;
use crate::dto::tag::CreateTagRequest;
use crate::models::tags;
use crate::{ApiResponse, HttpResult, RouteInfo, utils::db_err_map};
use actix_web::{HttpResponse, web};
use route_macros::crud_entity;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, ModelTrait, PaginatorTrait};

crud_entity!({
    entity : tags,
    route_prefix:"/api/tags",
    permission_prefix: "tags",
    id_type:"id",
    operations: ["create","list","delete","read"],
    create_request_type: CreateTagRequest
});
