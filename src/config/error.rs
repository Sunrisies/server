use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use serde::Serialize;
use std::fmt;
use utoipa::ToSchema;

use crate::{ApiResponse, dto::user::ValidationErrorJson};

#[derive(Debug, Serialize, ToSchema)]
pub enum AppError {
    // 认证相关错误 (4xx)
    Unauthorized(String),
    Forbidden(String),
    InvalidCredentials(String),
    TokenExpired(String),
    TokenInvalid(String),

    // 输入验证错误 (4xx)
    BadRequest(String),
    ValidationError(ValidationErrorJson),
    UnprocessableEntity(String),

    // 资源操作错误 (4xx)
    NotFound(String),
    AlreadyExists(String),
    Conflict(String),
    RateLimited(String),

    // 文件操作错误 (4xx)
    FileTooLarge(String),
    UnsupportedFileType(String),
    UploadFailed(String),

    // 数据库错误 (5xx)
    DatabaseError(String),
    DatabaseTimeout(String),
    DatabaseConnectionError(String),

    // 外部服务错误 (5xx)
    ExternalServiceError(String),
    EmailServiceError(String),
    SearchServiceError(String),
    StorageServiceError(String),

    // 服务器内部错误 (5xx)
    InternalServerError(String),
    ConfigurationError(String),
    EncryptionError(String),

    // 其他错误
    NotImplemented(String),
    MaintenanceMode(String),
    MultipartError(String), // 新增
}

impl AppError {
    /// 返回给前端的用户友好错误消息（5xx 统一隐藏技术细节）
    pub fn user_message(&self) -> String {
        match self {
            // 4xx — 由 handler 构造，通常是用户友好消息，直接返回
            AppError::Unauthorized(msg)
            | AppError::Forbidden(msg)
            | AppError::InvalidCredentials(msg)
            | AppError::TokenExpired(msg)
            | AppError::TokenInvalid(msg)
            | AppError::BadRequest(msg)
            | AppError::UnprocessableEntity(msg)
            | AppError::NotFound(msg)
            | AppError::AlreadyExists(msg)
            | AppError::Conflict(msg)
            | AppError::RateLimited(msg)
            | AppError::FileTooLarge(msg)
            | AppError::UnsupportedFileType(msg)
            | AppError::UploadFailed(msg)
            | AppError::NotImplemented(msg)
            | AppError::MaintenanceMode(msg)
            | AppError::MultipartError(msg) => msg.clone(),

            // 5xx — 统一隐藏技术细节
            AppError::DatabaseError(_)
            | AppError::DatabaseTimeout(_)
            | AppError::DatabaseConnectionError(_)
            | AppError::ExternalServiceError(_)
            | AppError::EmailServiceError(_)
            | AppError::SearchServiceError(_)
            | AppError::StorageServiceError(_)
            | AppError::InternalServerError(_)
            | AppError::ConfigurationError(_)
            | AppError::EncryptionError(_) => "服务器内部错误，请稍后再试".to_string(),

            // ValidationError 特殊处理
            AppError::ValidationError(_) => "请求参数校验失败".to_string(),
        }
    }
    // 获取错误码
    pub fn code(&self) -> i32 {
        self.status_code().as_u16() as i32
    }
    // 获取 HTTP 状态码
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::Unauthorized(_)
            | AppError::InvalidCredentials(_)
            | AppError::TokenExpired(_)
            | AppError::TokenInvalid(_) => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::BadRequest(_) | AppError::ValidationError(_) | AppError::UploadFailed(_) => {
                StatusCode::BAD_REQUEST
            }
            AppError::UnprocessableEntity(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::AlreadyExists(_) | AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::RateLimited(_) => StatusCode::TOO_MANY_REQUESTS,
            AppError::FileTooLarge(_) => StatusCode::PAYLOAD_TOO_LARGE,
            AppError::UnsupportedFileType(_) => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            AppError::DatabaseError(_)
            | AppError::DatabaseTimeout(_)
            | AppError::DatabaseConnectionError(_)
            | AppError::ExternalServiceError(_)
            | AppError::EmailServiceError(_)
            | AppError::SearchServiceError(_)
            | AppError::StorageServiceError(_)
            | AppError::InternalServerError(_)
            | AppError::ConfigurationError(_)
            | AppError::EncryptionError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::NotImplemented(_) => StatusCode::NOT_IMPLEMENTED,
            AppError::MaintenanceMode(_) => StatusCode::SERVICE_UNAVAILABLE,
            AppError::MultipartError(_) => StatusCode::BAD_REQUEST, // 新增
        }
    }

    // 获取错误详情
    pub fn details(&self) -> Option<serde_json::Value> {
        match self {
            AppError::ValidationError(field_errors) => {
                Some(serde_json::to_value(field_errors).unwrap())
            }
            _ => None,
        }
    }
    // 转换为 ApiResponse
    pub fn to_response(&self) -> ApiResponse<serde_json::Value> {
        // 5xx 错误自动记录详细日志供运维排查
        if self.status_code().as_u16() >= 500 {
            log::error!("服务器错误: {:?}", self);
        }
        ApiResponse {
            code: self.code(),
            message: self.user_message(),
            data: self.details(),
        }
    }
}

// Display 用于日志，直接输出 Debug 信息
impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// 实现 ResponseError trait 用于 Actix-web 错误处理
impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(self.to_response())
    }
}

// 从 SeaORM 数据库错误转换
impl From<sea_orm::DbErr> for AppError {
    fn from(err: sea_orm::DbErr) -> Self {
        log::error!("数据库错误(已隐藏): {:?}", err);
        AppError::DatabaseError(String::new())
    }
}

// 从 IO 错误转换
impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        log::error!("IO错误(已隐藏): {:?}", error);
        AppError::InternalServerError(String::new())
    }
}

/// actix_multipart 错误转换
impl From<actix_multipart::MultipartError> for AppError {
    fn from(error: actix_multipart::MultipartError) -> Self {
        log::error!("Multipart错误(已隐藏): {:?}", error);
        AppError::UploadFailed("文件上传失败，请重试".to_string())
    }
}

impl From<argon2::password_hash::Error> for AppError {
    fn from(error: argon2::password_hash::Error) -> Self {
        log::error!("密码加密错误(已隐藏): {:?}", error);
        AppError::InternalServerError(String::new())
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(error: jsonwebtoken::errors::Error) -> Self {
        match error.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                AppError::TokenExpired("Token has expired".to_string())
            }
            _ => AppError::TokenInvalid("Invalid token".to_string()),
        }
    }
}
