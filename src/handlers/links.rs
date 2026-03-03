use crate::config::AppError;
use crate::dto::PaginationQuery;
use crate::dto::link::CreateLinkRequest;
use crate::dto::user::ValidationErrorJson;
use crate::dto::{PaginatedResp, Pagination};

use crate::models::external_links;
use crate::{ApiResponse, EmptyResponse, HttpResult, RouteInfo, utils::db_err_map};
use actix_web::{HttpResponse, web};
use route_macros::crud_entity;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, ModelTrait, PaginatorTrait};
crud_entity!({
    entity : external_links,
    route_prefix:"/api/v1/links",
    permission_prefix: "links",
    id_type:"id",
    operations: ["create","list","delete","read"],
    create_request_type: CreateLinkRequest,
    openapi_summary:"链接管理",
    openapi_read: {
        summary: "获取链接详情",
        description: "根据ID获取单个外部链接的详细信息"
    }
});
