use actix_web::web;
use sea_orm::DatabaseConnection;

use crate::dto::user::ValidationErrorJson;
use crate::{ApiResponse, HttpResult, SseNotifier};
use crate::{AuthService, RegisterResponse};

// 注册
#[utoipa::path(
    post,
    summary = "注册",
    path = "/api/v1/auth/register",
    request_body( content = RegisterResponse),
    responses(
        (status = 200, description = "添加成功", body = RegisterResponse),
        (status = 422,description = "校验失败", body = ApiResponse<ValidationErrorJson> )
    ),
)]
pub async fn register(
    db_pool: web::Data<DatabaseConnection>,
    user_data: web::Json<RegisterResponse>,
    notifier: web::Data<SseNotifier>,
) -> HttpResult {
    match AuthService::register(db_pool, user_data, notifier).await {
        Ok(user) => Ok(ApiResponse::success(user, "添加用户成功").to_http_response()),
        Err(e) => Ok(ApiResponse::from(e).to_http_response()),
    }
}
