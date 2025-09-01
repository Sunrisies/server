use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};

use crate::{config::AppError, dto::user::ValidationErrorJson};

/// 统一的API响应结构
#[derive(Deserialize, Serialize)]
pub struct ApiResponse<T>
where
    T: Serialize,
{
    /// 状态码
    code: i32,
    /// 消息
    message: String,
    /// 数据
    data: Option<T>,
}

impl<T: Serialize> ApiResponse<T> {
    /// 用数据创建成功响应
    pub fn success(data: T, message: &str) -> Self {
        ApiResponse {
            code: 200,
            message: message.to_owned(),
            data: Some(data),
        }
    }
    /// 创建成功响应，但无数据
    pub fn success_msg(message: &str) -> Self {
        ApiResponse {
            code: 200,
            message: message.to_owned(),
            data: None,
        }
    }

    ///转换为具有适当状态代码的HTTP响应
    pub fn to_http_response(&self) -> HttpResponse {
        let status_code = match self.code {
            200..=299 => actix_web::http::StatusCode::OK,
            400 => actix_web::http::StatusCode::BAD_REQUEST,
            401 => actix_web::http::StatusCode::UNAUTHORIZED,
            403 => actix_web::http::StatusCode::FORBIDDEN,
            404 => actix_web::http::StatusCode::NOT_FOUND,
            409 => actix_web::http::StatusCode::CONFLICT,
            422 => actix_web::http::StatusCode::UNPROCESSABLE_ENTITY,
            429 => actix_web::http::StatusCode::TOO_MANY_REQUESTS,
            500..=599 => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            _ => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        };

        HttpResponse::build(status_code).json(self)
    }
}

// 实现ApiResponse<()>的错误转换
impl From<AppError> for ApiResponse<ValidationErrorJson> {
    fn from(error: AppError) -> Self {
        match error {
            AppError::BadRequest(msg) => ApiResponse {
                code: 400,
                message: msg,
                data: None,
            },
            AppError::NotFound(msg) => ApiResponse {
                code: 404,
                message: msg,
                data: None,
            },
            AppError::Unauthorized(msg) => ApiResponse {
                code: 401,
                message: msg,
                data: None,
            },

            AppError::Conflict(msg) => ApiResponse {
                code: 409,
                message: msg,
                data: None,
            },
            AppError::Forbidden(msg) => ApiResponse {
                code: 403,
                message: msg,
                data: None,
            },
            AppError::InternalServerError(msg) => ApiResponse {
                code: 500,
                message: msg,
                data: None,
            },

            AppError::DatabaseError(msg) => ApiResponse {
                code: 500,
                message: msg,
                data: None,
            },
            AppError::ValidationError(msg) => {
                println!("Validation errors:-- {:?}", msg.errors);
                ApiResponse {
                    code: 422,
                    message: "校验失败".to_string(),
                    data: Some(ValidationErrorJson { ..msg }),
                }
            }
            _ => ApiResponse {
                code: 500,
                message: "Internal Server Error".to_string(),
                data: None,
            },
            // 根据需要添加其他错误变量
        }
    }
}

/// 带数据的API结果的类型别名
pub type ApiResult<T> = Result<ApiResponse<T>, AppError>;

/// HTTP结果的类型别名
pub type HttpResult = Result<HttpResponse, AppError>;

/// 将ApiResult转换为HttpResult的实用函数
pub fn to_http_result<T: Serialize>(result: ApiResult<T>) -> HttpResult {
    result.map(|resp| resp.to_http_response())
}
