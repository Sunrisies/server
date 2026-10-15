use crate::config::AppError;
use crate::dto::user::{LoginRequest, ValidationErrorJson};
use crate::{ApiResponse, HttpResult, SseNotifier};
use crate::{AuthService, RegisterResponse};
use crate::{EmailVerificationManager, RouteInfo};
use actix_web::web;
use route_macros::route_permission;
use sea_orm::DatabaseConnection;

// 注册
#[utoipa::path(
    post,
    summary = "注册",
    path = "/api/v1/auth/register",
    tag="鉴权模块",
    description = "注册接口，用户名长度5-100，密码长度6-100",
    request_body(
        content = RegisterResponse,
        examples(
            ("注册请求" = (value = json!({"user_name": "chao yang", "pass_word": "123456"})))
        )
    ),
    responses(
        (status = 200, description = "注册成功", body = ApiResponse<crate::models::users::Model>,
            example = json!({
                "code": 200,
                "message": "添加用户成功",
                "data": {
                    "id": 1,
                    "uuid": "550e8400-e29b-41d4-a716-446655440000",
                    "user_name": "chao yang",
                    "email": null,
                    "image": null,
                    "phone": null,
                    "binding": null,
                    "created_at": "2026-05-25 14:00:00",
                    "updated_at": "2026-05-25 14:00:00"
                }
            })),
        (status = 422, description = "校验失败",
            example = json!({
                "code": 422,
                "message": "请求参数校验失败",
                "data": {
                    "errors": [{"name": "user_name", "error": "用户名长度必须在5到100之间"}]
                }
            })),
    ),
)]
#[route_permission(path = "/api/register", method = "post", permission = "auth:register")]
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
    description = "支持密码与邮箱验证码登录；手机号、第三方登录暂未开放",
    request_body(
        content = LoginRequest,
        examples(
            ("密码登录" = (value = json!({"login_type": "password", "account": "admin", "password": "123456"}))),
            ("邮箱登录" = (value = json!({"login_type": "email", "email": "user@example.com", "code": "123456"})))
        )
    ),
    responses(
        (status = 200, description = "登录成功（密码登录/邮箱登录通过 Cookie 返回令牌）",
            example = json!({
                "code": 200,
                "message": "密码登录成功",
                "data": {
                    "id": 1,
                    "uuid": "550e8400-e29b-41d4-a716-446655440000",
                    "user_name": "admin",
                    "email": "admin@example.com",
                    "image": null,
                    "phone": null,
                    "created_at": "2026-05-25 14:00:00",
                    "updated_at": "2026-05-25 14:00:00"
                }
            })),
        (status = 401, description = "密码错误",
            example = json!({
                "code": 401,
                "message": "密码错误",
                "data": null
            })),
        (status = 404, description = "用户不存在",
            example = json!({
                "code": 404,
                "message": "未找到: 用户不存在",
                "data": null
            })),
        (status = 422, description = "校验失败",
            example = json!({
                "code": 422,
                "message": "请求参数校验失败",
                "data": {
                    "errors": [{"name": "password", "error": "密码长度必须在6到100之间"}]
                }
            })),
    ),
)]
#[route_permission(path = "/api/login", method = "post", permission = "auth:login")]
pub async fn login(
    req: actix_web::HttpRequest,
    db_pool: web::Data<DatabaseConnection>,
    login: web::Json<LoginRequest>,
    email_manager: web::Data<EmailVerificationManager>, // 添加这行
) -> HttpResult {
    if let Err(e) = login.validate() {
        let msg = ValidationErrorJson::from_validation_errors(&e);
        return Ok(ApiResponse::from(AppError::ValidationError(msg)).to_http_response());
    }
    match login.0 {
        LoginRequest::Password(p) => {
            crate::utils::login_limit::check(
                db_pool.get_ref(),
                &crate::utils::client_ip::client_ip(&req),
                &p.account,
            )
            .await?;
            AuthService::login_by_pwd(db_pool, p).await
        }
        LoginRequest::Email(e) => {
            crate::utils::login_limit::check(
                db_pool.get_ref(),
                &crate::utils::client_ip::client_ip(&req),
                &format!("email:{}", e.email),
            )
            .await?;
            AuthService::login_by_email(db_pool, e, email_manager).await
        }
        LoginRequest::Phone(p) => AuthService::login_by_phone(db_pool, p),
        LoginRequest::OAuth(o) => AuthService::login_by_oauth(db_pool, o),
    }
}

/// 注销当前 JWT，数据库撤销记录在重启和多实例间保持有效。
pub async fn logout(req: actix_web::HttpRequest, db: web::Data<DatabaseConnection>) -> HttpResult {
    if let Some(cookie) = req.cookie("access_token") {
        if let Ok(claims) = crate::utils::jwt::decode_jwt(cookie.value()) {
            crate::utils::token_revocation::revoke(db.get_ref(), cookie.value(), claims.exp)
                .await?;
        }
    }
    let cookie = actix_web::cookie::Cookie::build("access_token", "")
        .path("/")
        .http_only(true)
        .secure(true)
        .same_site(actix_web::cookie::SameSite::Strict)
        .max_age(actix_web::cookie::time::Duration::ZERO)
        .finish();
    Ok(actix_web::HttpResponse::Ok()
        .cookie(cookie)
        .json(serde_json::json!({"code":200,"data":{"revoked":true}})))
}
