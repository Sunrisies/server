use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use actix_web::{HttpResponse, web};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::ApiResponse;
use crate::RouteInfo;
use crate::config::AppError;
use crate::dto::user::ValidationErrorJson;
use crate::route_permission;
use crate::services::{EmailService, EmailVerificationManager};

lazy_static! {
    static ref RATE_LIMITER: Mutex<HashMap<String, (u32, Instant)>> = Mutex::new(HashMap::new());
}

const MAX_CODES_PER_WINDOW: u32 = 3;
const RATE_WINDOW_SECS: u64 = 300; // 5 分钟

fn check_rate_limit(key: &str) -> Result<(), AppError> {
    let now = Instant::now();
    let mut map = RATE_LIMITER
        .lock()
        .map_err(|_| AppError::InternalServerError("速率限制器异常".to_string()))?;

    let entry = map.get(key).copied();
    if let Some((count, window_start)) = entry {
        if window_start.elapsed().as_secs() < RATE_WINDOW_SECS {
            if count >= MAX_CODES_PER_WINDOW {
                return Err(AppError::RateLimited(format!(
                    "请求过于频繁，请 {} 秒后重试",
                    RATE_WINDOW_SECS - window_start.elapsed().as_secs()
                )));
            }
            map.insert(key.to_string(), (count + 1, window_start));
        } else {
            map.insert(key.to_string(), (1, now));
        }
    } else {
        map.insert(key.to_string(), (1, now));
    }
    Ok(())
}

/// 发送验证码请求
#[derive(Debug, Validate, Deserialize, ToSchema)]
pub struct SendVerificationCodeRequest {
    /// 邮箱地址
    #[validate(email)]
    pub email: String,
}

/// 发送验证码响应
#[derive(Debug, Serialize, ToSchema)]
pub struct SendVerificationCodeResponse {
    /// 是否成功
    pub success: bool,
    /// 消息
    pub message: String,
}

/// 发送邮箱验证码
#[utoipa::path(
    post,
    path = "/api/v1/email/send-verification-code",
    tag = "邮件",
    summary = "发送邮箱验证码",
    request_body = SendVerificationCodeRequest,
    responses(
        (status = 200, description = "验证码发送成功", body = SendVerificationCodeResponse),
        (status = 400, description = "请求参数错误", body = ApiResponse<ValidationErrorJson>),
        (status = 500, description = "服务器内部错误", body = ApiResponse<ValidationErrorJson>)
    )
)]
#[route_permission(
    path = "/api/v1/email/send-verification-code",
    method = "post",
    permission = "email:send_code"
)]
pub async fn send_verification_code(
    request: web::Json<SendVerificationCodeRequest>,
    email_service: web::Data<EmailService>,
    email_verification_manager: web::Data<EmailVerificationManager>,
) -> Result<HttpResponse, AppError> {
    // 校验请求参数
    if let Err(errors) = request.validate() {
        return Err(AppError::ValidationError(
            crate::dto::user::ValidationErrorJson::from_validation_errors(&errors),
        ));
    }

    // 速率限制：每个邮箱 5 分钟内最多 3 次
    check_rate_limit(&format!("email:{}", request.email))?;

    // 生成并发送验证码
    match email_verification_manager
        .generate_and_send_code(&email_service, &request.email)
        .await
    {
        Ok(_) => Ok(HttpResponse::Ok().json(SendVerificationCodeResponse {
            success: true,
            message: "验证码已发送，请查收邮件".to_string(),
        })),
        Err(e) => Err(AppError::InternalServerError(format!(
            "发送验证码失败: {}",
            e
        ))),
    }
}
