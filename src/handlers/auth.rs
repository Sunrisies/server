use actix_web::web;
use sea_orm::DatabaseConnection;

use crate::config::AppError;
use crate::dto::user::{LoginRequest, ValidationErrorJson};
use crate::{ApiResponse, HttpResult, SseNotifier};
use crate::{AuthService, RegisterResponse};

// 注册
#[utoipa::path(
    post,
    summary = "注册",
    path = "/api/v1/auth/register",
    tag="鉴权模块",
    description = "注册接口",
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

///登录
#[utoipa::path(
    post,
    summary = "登录",
    path = "/api/v1/auth/login",
    tag="鉴权模块",
    description = "登录接口",
    request_body( content = LoginRequest),
    responses(
        (status = 200, description = "登录成功", body = RegisterResponse),
        (status = 422,description = "校验失败", body = ApiResponse<ValidationErrorJson> )
        ),

)]
pub async fn login(
    db_pool: web::Data<DatabaseConnection>,
    login: web::Json<LoginRequest>,
) -> HttpResult {
    if let Err(e) = login.validate() {
        let msg = ValidationErrorJson::from_validation_errors(&e);
        return Ok(ApiResponse::from(AppError::ValidationError(msg)).to_http_response());
    }
    match login.0 {
        LoginRequest::Password(p) => AuthService::login_by_pwd(db_pool, p).await,
        LoginRequest::Email(e) => AuthService::login_by_email(db_pool, e).await,
        LoginRequest::Phone(p) => AuthService::login_by_phone(db_pool, p).await,
        LoginRequest::OAuth(o) => AuthService::login_by_oauth(db_pool, o).await,
    }
    // match AuthService::login(db_pool, user_data).await {
    //     Ok(user) => Ok(ApiResponse::success(user, "登录成功").to_http_response()),
    //     Err(e) => Ok(ApiResponse::from(e).to_http_response()),
    // }
}
