use crate::dto::link::CreateLinkRequest;
use crate::dto::user::ValidationErrorJson;
use crate::dto::{PaginatedResp, Pagination, PaginationQuery};
use crate::models::external_links;
use crate::utils::db_err_map;
use crate::{ApiResponse, EmptyResponse, HttpResult, RouteInfo};
use actix_web::{HttpResponse, web};
use route_macros::crud_entity;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, ModelTrait, PaginatorTrait};
use validator::Validate;

crud_entity!({
    entity : external_links,
    route_prefix:"/api/v1/links",
    permission_prefix: "links",
    id_type:"id",
    operations: ["create","list","delete","read"],
    create_request_type: CreateLinkRequest,
    unique_field: Name,
    openapi_summary:"链接管理",
    openapi_read: {
        summary: "获取链接详情",
        description: "根据ID获取单个外部链接的详细信息"
    }
});
