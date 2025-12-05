use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use serde::Serialize;
use std::fmt;

use crate::{ApiResponse, dto::user::ValidationErrorJson};

#[derive(Debug, Serialize)]
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
}

impl AppError {
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
        ApiResponse {
            code: self.code(),
            message: self.to_string(),
            data: self.details(),
        }
    }
}

// 实现 Display trait 用于错误消息显示
impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Unauthorized(msg) => write!(f, "未授权: {}", msg),
            AppError::Forbidden(msg) => write!(f, "禁止访问: {}", msg),
            AppError::InvalidCredentials(msg) => write!(f, "无效的凭据: {}", msg),
            AppError::TokenExpired(msg) => write!(f, "令牌已过期: {}", msg),
            AppError::TokenInvalid(msg) => write!(f, "无效的令牌: {}", msg),
            AppError::BadRequest(msg) => write!(f, "错误的请求: {}", msg),

            AppError::ValidationError(error) => write!(f, "验证错误: {:?}", error),
            AppError::UnprocessableEntity(msg) => write!(f, "无法处理的实体: {}", msg),
            AppError::NotFound(msg) => write!(f, "未找到: {}", msg),
            AppError::AlreadyExists(msg) => write!(f, "已存在: {}", msg),
            AppError::Conflict(msg) => write!(f, "冲突: {}", msg),
            AppError::RateLimited(msg) => write!(f, "请求过于频繁: {}", msg),
            AppError::FileTooLarge(msg) => write!(f, "文件太大: {}", msg),
            AppError::UnsupportedFileType(msg) => write!(f, "不支持的文件类型: {}", msg),
            AppError::UploadFailed(msg) => write!(f, "上传失败: {}", msg),
            AppError::DatabaseError(msg) => write!(f, "数据库错误: {}", msg),
            AppError::DatabaseTimeout(msg) => write!(f, "数据库超时: {}", msg),
            AppError::DatabaseConnectionError(msg) => {
                write!(f, "数据库连接错误: {}", msg)
            }
            AppError::ExternalServiceError(msg) => write!(f, "外部服务错误: {}", msg),
            AppError::EmailServiceError(msg) => write!(f, "邮箱服务错误: {}", msg),
            AppError::SearchServiceError(msg) => write!(f, "搜索服务错误: {}", msg),
            AppError::StorageServiceError(msg) => write!(f, "存储服务错误: {}", msg),
            AppError::InternalServerError(msg) => write!(f, "服务器内部错误: {}", msg),
            AppError::ConfigurationError(msg) => write!(f, "配置错误: {}", msg),
            AppError::EncryptionError(msg) => write!(f, "加密错误: {}", msg),
            AppError::NotImplemented(msg) => write!(f, "未实现: {}", msg),
            AppError::MaintenanceMode(msg) => write!(f, "维护模式: {}", msg),
        }
    }
}

// 实现 ResponseError trait 用于 Actix-web 错误处理
impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(self.to_response())
    }
}

// 从 Diesel 错误转换
impl From<sea_orm::DbErr> for AppError {
    fn from(err: sea_orm::DbErr) -> Self {
        AppError::DatabaseError(err.to_string())
    }
}

// 从其他常见错误类型转换
impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        AppError::InternalServerError(format!("IO error: {}", error))
    }
}
/// actix_multipart
impl From<actix_multipart::MultipartError> for AppError {
    fn from(error: actix_multipart::MultipartError) -> Self {
        AppError::UploadFailed(format!("Multipart error: {}", error))
    }
}

impl From<argon2::password_hash::Error> for AppError {
    fn from(error: argon2::password_hash::Error) -> Self {
        AppError::EncryptionError(format!("Password hashing error: {}", error))
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
