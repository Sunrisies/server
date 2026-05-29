use crate::dto::{PaginatedResp, Pagination, PaginationQuery};
use crate::models::users;
use crate::{ApiResponse, HttpResult, RouteInfo};
use actix_web::{HttpResponse, web};
use route_macros::crud_entity;
use sea_orm::{DatabaseConnection, EntityTrait, PaginatorTrait};

crud_entity!({
    entity : users,
    route_prefix:"/api/v1/users",
    permission_prefix: "users",
    id_type:"uuid",
    openapi_summary: "用户",
    operations: ["list","read"],
});
