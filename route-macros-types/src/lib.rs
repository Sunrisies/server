//! route-macros-types — `route-macros` crate 所需的共享类型和工具函数

use actix_web::HttpResponse;
use sea_orm::DbErr;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// ── RouteInfo ──

/// 路由信息（由 `route_permission` 宏收集）
#[derive(Debug, Clone, Serialize)]
pub struct RouteInfo {
    pub path: &'static str,
    pub method: &'static str,
    pub permission: &'static str,
}

// ── API 响应 ──

/// 统一的 API 响应结构
#[derive(Deserialize, Serialize, ToSchema)]
pub struct ApiResponse<T: Serialize> {
    pub code: i32,
    pub message: String,
    pub data: Option<T>,
}

impl<T: Serialize> ApiResponse<T> {
    /// 创建成功响应
    pub fn success(data: T, message: &str) -> Self {
        ApiResponse {
            code: 200,
            message: message.to_owned(),
            data: Some(data),
        }
    }

    /// 转换为 HTTP 响应
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

// ── 空响应 ──

/// 空响应数据（用于删除等无返回数据的操作）
#[derive(Serialize, ToSchema)]
pub struct EmptyResponse;

impl ApiResponse<EmptyResponse> {
    /// 创建成功响应，无数据
    pub fn success_msg(message: &str) -> Self {
        ApiResponse {
            code: 200,
            message: message.to_owned(),
            data: None,
        }
    }
}

// ── 分页 ──

/// 分页信息
#[derive(Serialize, ToSchema)]
pub struct Pagination {
    pub page: u64,
    pub limit: u64,
    pub total: u64,
}

/// 统一分页响应
#[derive(Serialize, ToSchema)]
pub struct PaginatedResp<T: Serialize> {
    pub data: Vec<T>,
    pub pagination: Pagination,
}

/// 分页查询参数
#[derive(Deserialize, Serialize, utoipa::IntoParams)]
#[into_params(style = Form, parameter_in = Query)]
pub struct PaginationQuery {
    /// 页码
    #[param(example = 1)]
    #[serde(default = "default_page")]
    pub page: u64,
    /// 每页数量
    #[param(example = 10)]
    #[serde(default = "default_limit")]
    pub limit: u64,
}

fn default_page() -> u64 {
    1
}
fn default_limit() -> u64 {
    10
}

// ── 验证错误 ──

/// 验证错误项
#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ValidationErrorItem {
    #[schema(example = "user_name")]
    pub name: String,
    #[schema(example = "用户名长度必须在5到100之间")]
    pub error: String,
}

/// 验证错误列表
#[derive(Serialize, Debug, ToSchema)]
pub struct ValidationErrorJson {
    pub errors: Vec<ValidationErrorItem>,
}

impl ValidationErrorJson {
    pub fn from_validation_errors(errs: &validator::ValidationErrors) -> Self {
        let mut list = Vec::new();
        for (field, field_errs) in errs.field_errors() {
            for err in field_errs {
                let msg = err
                    .message
                    .as_ref()
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "invalid value".into());
                list.push(ValidationErrorItem {
                    name: field.to_string(),
                    error: msg,
                });
            }
        }
        ValidationErrorJson { errors: list }
    }
}

// ── 数据库错误映射 ──

/// 把 Sea-ORM 底层错误转成用户能看懂的 &str
pub fn db_err_map(e: DbErr) -> &'static str {
    let detail = e.to_string();
    if detail.contains("duplicate key value") {
        if detail.contains("users_email_key") {
            "邮箱已被注册"
        } else if detail.contains("users_username_key") {
            "用户名已被使用"
        } else {
            "数据重复，请检查唯一字段"
        }
    } else if detail.contains("foreign key constraint") {
        "关联数据不存在，无法操作"
    } else if detail.contains("violates not-null constraint") {
        "必填字段不能为空"
    } else if detail.contains("value too long") {
        "字段长度超出限制"
    } else {
        "数据库操作失败，请稍后再试"
    }
}

// ── 默认错误类型 ──

/// 宏生成的代码默认使用的错误类型（可通过 `error_type` 覆盖）
#[derive(Debug)]
pub enum AppError {
    DatabaseError(String),
    DatabaseConnectionError(String),
    NotFound(String),
    ValidationError(ValidationErrorJson),
    BadRequest(String),
}

impl AppError {
    pub fn status_code(&self) -> u16 {
        match self {
            AppError::NotFound(_) => 404,
            AppError::ValidationError(_) | AppError::BadRequest(_) => 400,
            _ => 500,
        }
    }
}

/// HTTP 结果类型（用于宏生成的 handler 返回值）
pub type HttpResult<E = AppError> = Result<HttpResponse, E>;

// ── 日志输出 ──

/// 编译时输出宏日志
#[macro_export]
macro_rules! log_macro_info {
    ($($arg:tt)*) => {
        eprintln!("[route-macros] {}", format_args!($($arg)*));
    };
}

// ── SeaORM 实体工具宏 ──

/// 为实体添加 `find_by_col` 和 `check_unique` 方法
#[macro_export]
macro_rules! impl_entity_unique_check {
    ($entity:ident, $model:ident) => {
        impl $entity {
            pub fn find_by_col<C, V>(col: C, val: V) -> sea_orm::Select<Self>
            where
                C: sea_orm::ColumnTrait,
                V: Into<sea_orm::Value>,
            {
                Self::find().filter(col.eq(val))
            }
            pub async fn check_unique(
                db: &sea_orm::DatabaseConnection,
                col: impl sea_orm::ColumnTrait,
                value: impl Into<sea_orm::Value>,
            ) -> Result<Option<$model>, sea_orm::DbErr> {
                Self::find().filter(col.eq(value)).one(db).await
            }
        }
    };
}

/// 从请求结构体转换为 ActiveModel
#[macro_export]
macro_rules! impl_from_request {
    ($request:ty => $model:ty { $($field:ident),* $(,)? }) => {
        impl From<$request> for $model {
            fn from(request: $request) -> Self {
                let mut model = <$model>::default();
                $(
                    model.$field = sea_orm::Set(request.$field);
                )*
                model
            }
        }
    };
}

/// 带转换逻辑的 From 请求宏
#[macro_export]
macro_rules! impl_from_request_with_default {
    ($request:ty => $model:ty {
        fields: { $($field:ident: $transform:expr),* $(,)? },
        defaults: { $($default_field:ident: $default_value:expr),* $(,)? }
    }) => {
        impl $model {
            pub fn from_request(request: $request) -> Self {
                let mut model = <$model>::default();
                $(
                    model.$field = sea_orm::Set($transform);
                )*
                $(
                    model.$default_field = sea_orm::Set($default_value);
                )*
                model
            }
        }
    };
}
