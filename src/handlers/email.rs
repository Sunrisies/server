use crate::ApiResponse;
use crate::config::AppError;
use crate::dto::user::ValidationErrorJson;
use crate::services::{EmailService, EmailVerificationManager};
use actix_web::{HttpResponse, web};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

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
    path = "/api/email/send-verification-code",
    tag = "邮件",
    summary = "发送邮箱验证码",
    request_body = SendVerificationCodeRequest,
    responses(
        (status = 200, description = "验证码发送成功", body = SendVerificationCodeResponse),
        (status = 400, description = "请求参数错误", body = ApiResponse<ValidationErrorJson>),
        (status = 500, description = "服务器内部错误", body = ApiResponse<ValidationErrorJson>)
    )
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
