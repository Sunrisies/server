use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt;

use crate::dto::user::ValidationErrorJson;

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

// 字段级错误详情
#[derive(Debug, Serialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
    pub code: String,
}

impl AppError {
    // 便捷构造函数
    pub fn unauthorized(message: &str) -> Self {
        AppError::Unauthorized(message.to_string())
    }

    pub fn not_found(resource: &str) -> Self {
        AppError::NotFound(format!("{} 未找到", resource))
    }

    // pub fn validation(field: &str, message: &str, code: &str) -> Self {
    //     AppError::ValidationError(vec![FieldError {
    //         field: field.to_string(),
    //         message: message.to_string(),
    //         code: code.to_string(),
    //     }])
    // }

    pub fn already_exists(resource: &str) -> Self {
        AppError::AlreadyExists(format!("{} 已存在", resource))
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
        match self {
            // 4xx 错误
            AppError::Unauthorized(_) => HttpResponse::Unauthorized().json(self),
            AppError::Forbidden(_) => HttpResponse::Forbidden().json(self),
            AppError::InvalidCredentials(_) => HttpResponse::Unauthorized().json(self),
            AppError::TokenExpired(_) => HttpResponse::Unauthorized().json(self),
            AppError::TokenInvalid(_) => HttpResponse::Unauthorized().json(self),
            AppError::BadRequest(_) => HttpResponse::BadRequest().json(self),
            AppError::ValidationError(_) => HttpResponse::BadRequest().json(self),
            AppError::UnprocessableEntity(_) => HttpResponse::UnprocessableEntity().json(self),
            AppError::NotFound(_) => HttpResponse::NotFound().json(self),
            AppError::AlreadyExists(_) => HttpResponse::Conflict().json(self),
            AppError::Conflict(_) => HttpResponse::Conflict().json(self),
            AppError::RateLimited(_) => HttpResponse::TooManyRequests().json(self),
            AppError::FileTooLarge(_) => HttpResponse::PayloadTooLarge().json(self),
            AppError::UnsupportedFileType(_) => HttpResponse::UnsupportedMediaType().json(self),
            AppError::UploadFailed(_) => HttpResponse::BadRequest().json(self),

            // 5xx 错误
            AppError::DatabaseError(_)
            | AppError::DatabaseTimeout(_)
            | AppError::DatabaseConnectionError(_)
            | AppError::ExternalServiceError(_)
            | AppError::EmailServiceError(_)
            | AppError::SearchServiceError(_)
            | AppError::StorageServiceError(_)
            | AppError::InternalServerError(_)
            | AppError::ConfigurationError(_)
            | AppError::EncryptionError(_) => HttpResponse::InternalServerError().json(self),

            AppError::NotImplemented(_) => HttpResponse::NotImplemented().json(self),
            AppError::MaintenanceMode(_) => HttpResponse::ServiceUnavailable().json(self),
        }
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

// 错误响应格式
#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Serialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

// 为 AppError 实现到 ErrorResponse 的转换
impl From<AppError> for ErrorResponse {
    fn from(error: AppError) -> Self {
        let (code, message, details) = match error {
            AppError::ValidationError(field_errors) => (
                "VALIDATION_ERROR".to_string(),
                "One or more validation errors occurred".to_string(),
                Some(serde_json::to_value(field_errors).unwrap()),
            ),
            _ => {
                let code = match error {
                    AppError::ValidationError(_) => "VALIDATION_ERROR",
                    AppError::Unauthorized(_) => "UNAUTHORIZED",
                    AppError::Forbidden(_) => "FORBIDDEN",
                    AppError::InvalidCredentials(_) => "INVALID_CREDENTIALS",
                    AppError::TokenExpired(_) => "TOKEN_EXPIRED",
                    AppError::TokenInvalid(_) => "TOKEN_INVALID",
                    AppError::BadRequest(_) => "BAD_REQUEST",
                    AppError::UnprocessableEntity(_) => "UNPROCESSABLE_ENTITY",
                    AppError::NotFound(_) => "NOT_FOUND",
                    AppError::AlreadyExists(_) => "ALREADY_EXISTS",
                    AppError::Conflict(_) => "CONFLICT",
                    AppError::RateLimited(_) => "RATE_LIMITED",
                    AppError::FileTooLarge(_) => "FILE_TOO_LARGE",
                    AppError::UnsupportedFileType(_) => "UNSUPPORTED_FILE_TYPE",
                    AppError::UploadFailed(_) => "UPLOAD_FAILED",
                    AppError::DatabaseError(_) => "DATABASE_ERROR",
                    AppError::DatabaseTimeout(_) => "DATABASE_TIMEOUT",
                    AppError::DatabaseConnectionError(_) => "DATABASE_CONNECTION_ERROR",
                    AppError::ExternalServiceError(_) => "EXTERNAL_SERVICE_ERROR",
                    AppError::EmailServiceError(_) => "EMAIL_SERVICE_ERROR",
                    AppError::SearchServiceError(_) => "SEARCH_SERVICE_ERROR",
                    AppError::StorageServiceError(_) => "STORAGE_SERVICE_ERROR",
                    AppError::InternalServerError(_) => "INTERNAL_SERVER_ERROR",
                    AppError::ConfigurationError(_) => "CONFIGURATION_ERROR",
                    AppError::EncryptionError(_) => "ENCRYPTION_ERROR",
                    AppError::NotImplemented(_) => "NOT_IMPLEMENTED",
                    AppError::MaintenanceMode(_) => "MAINTENANCE_MODE",
                };
                (code.to_string(), error.to_string(), None)
            }
        };

        ErrorResponse {
            error: ErrorDetail {
                code,
                message,
                details,
            },
        }
    }
}
