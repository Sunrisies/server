use crate::config::AppError;
use crate::dto::PaginationQuery;
use crate::dto::category::CreateCategoryRequest;
use crate::dto::tag::CreateTagRequest;
use crate::models::users;
use crate::models::{categories, tags};
use crate::{ApiResponse, HttpResult, RouteInfo, utils::db_err_map};
use actix_web::{HttpResponse, web};
use route_macros::crud_entity;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, ModelTrait, QuerySelect};
crud_entity!({
    entity : users,
    route_prefix:"/api/users",
    permission_prefix: "users",
    id_type:"uuid",
    operations: ["read","list"],
});

crud_entity!({
    entity : categories,
    route_prefix:"/api/categories",
    permission_prefix: "categories",
    id_type:"id",
    operations: ["create"],
    create_request_type: CreateCategoryRequest

});
crud_entity!({
    entity : tags,
    route_prefix:"/api/tags",
    permission_prefix: "tags",
    id_type:"id",
    operations: ["create","list","delete"],
    create_request_type: CreateTagRequest
});
// / 获取用户列表
// #[utoipa::path(
//     get,
//     path = "/api/v1/users",
//     params(PaginationQuery),
//     responses(
//         (status = 200, description = "成功", body = String),
//     )
// )]

// pub async fn get_users(
//     db_pool: web::Data<DatabaseConnection>,
//     query: web::Query<PaginationQuery>,
// ) -> HttpResult {
//     let PaginationQuery { page, limit } = query.into_inner();
//     let result = UserService::get_users(db_pool.as_ref(), page, limit).await?;
//     Ok(result)
// }

// // 修改
// pub async fn _put_demo(
//     db: web::Data<DatabaseConnection>,
//     uuid: web::Path<String>,
//     user_data: web::Json<UpdateUserRequest>,
// ) -> Result<impl Responder, HttpResponse> {
//     // 验证UUID格式
//     let uuid_result = Uuid::parse_str(&uuid);
//     let uuid = match uuid_result {
//         Ok(u) => u,
//         Err(_) => return Err(HttpResponse::InternalServerError().json("Database query error")),
//     };

//     // 先找到用户
//     let existing_user = match UserEntity::find_by_uuid(&uuid.to_string())
//         .one(db.as_ref())
//         .await
//     {
//         Ok(u) => u,
//         Err(_e) => return Err(HttpResponse::InternalServerError().json("Database query error")),
//     };
//     let existing_user = existing_user
//         .ok_or_else(|| HttpResponse::NotFound().json(format!("ID为{}的用户不存在", uuid)))?;
//     let mut user_active: users::ActiveModel = existing_user.into();
//     user_active.user_name = Set(user_data.user_name.to_string());
//     user_active.image = Set(user_data.image.clone());
//     user_active.updated_at = Set(Utc::now());
//     let updated_user = user_active
//         .update(db.as_ref())
//         .await
//         .map_err(|e| HttpResponse::InternalServerError().json(format!("更新失败: {}", e)))?;
//     Ok(HttpResponse::Ok().json(updated_user))
//     // "修改 uuid:{:?}!".to_string()
// }

// // 删除
// #[utoipa::path(
//     post,
//     summary = "删除指定用户",
//     path = "/api/v1/users/{uuid}",
//     responses(
//         (status = 200, description = "删除成功", body = String),
//     ),
// )]
// pub async fn delete_user_uuid(
//     db: web::Data<DatabaseConnection>,
//     uuid: web::Path<String>,
// ) -> impl Responder {
//     // 验证UUID格式
//     let uuid_result = Uuid::parse_str(&uuid);
//     let uuid = match uuid_result {
//         Ok(u) => u,
//         Err(_) => return HttpResponse::InternalServerError().json("Database query error"),
//     };

//     // 先找到用户
//     let existing_user = match UserEntity::find_by_uuid(&uuid.to_string())
//         .one(db.as_ref())
//         .await
//     {
//         Ok(u) => u,
//         Err(_e) => return HttpResponse::InternalServerError().json("Database query error"),
//     };

//     match existing_user {
//         Some(user) => {
//             // 删除用户
//             match user.delete(db.as_ref()).await {
//                 Ok(_) => HttpResponse::Ok().json("删除成功"),
//                 Err(_e) => HttpResponse::InternalServerError().json("Database query error"),
//             }
//         }
//         None => HttpResponse::NotFound().json("User not found"),
//     }
// }

// // 获取单个
// #[utoipa::path(
//     post,
//     summary = "获取单个用户信息",
//     path = "/api/v1/users/{uuid}",
//     responses(
//         (status = 200, description = "获取成功", body = String),
//     ),
// )]
// pub async fn get_user_uuid(uuid: web::Path<String>) -> impl Responder {
//     format!("获取 uuid:{:?}!", uuid)
// }
