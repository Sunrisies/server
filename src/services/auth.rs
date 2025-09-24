use actix_web::web;
use chrono::Utc;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, DatabaseConnection};
use uuid::Uuid;
use validator::Validate;

use crate::config::AppError;
use crate::dto::user::{EmailLogin, OAuthLogin, PasswordLogin, PhoneLogin, ValidationErrorJson};
use crate::models::users::ActiveModel;
use crate::utils::{hash, verify};
use crate::{ApiResponse, SseNotifier};
use crate::{HttpResult, RegisterResponse};
pub struct AuthService;

impl AuthService {
    pub async fn register(
        db_pool: web::Data<DatabaseConnection>,
        user_data: web::Json<RegisterResponse>,
        notifier: web::Data<SseNotifier>,
    ) -> Result<crate::models::users::Model, AppError> {
        // 校验
        if let Err(errors) = user_data.validate() {
            let msg = ValidationErrorJson::from_validation_errors(&errors);
            return Err(AppError::ValidationError(msg));
        }
        let RegisterResponse {
            user_name,
            pass_word,
        } = user_data.into_inner();
        let password_hash = hash(&pass_word)?;
        let new_user = ActiveModel {
            user_name: Set(user_name.to_string()),
            pass_word: Set(password_hash),
            permissions: Set(Some("33333".to_string())), // 设置默认权限
            uuid: Set(Uuid::new_v4().to_string()),       // 生成唯一的UUID
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        };
        let user = match new_user.insert(db_pool.as_ref()).await {
            Ok(user) => {
                println!("User created successfully: {:?}", user);
                let notification = serde_json::json!({
                    "event": "user_updated",
                    "data": {
                        "user_id": user.id,
                        "updated_fields": {
                            "username": &user.user_name,
                        }
                    }
                });
                notifier.notify(&notification.to_string());
                user
            }
            Err(e) => return Err(AppError::DatabaseError(e.to_string())),
        };
        Ok(user)
    }

    pub async fn login_by_pwd(
        db_pool: web::Data<DatabaseConnection>,
        login: PasswordLogin,
    ) -> HttpResult {
        // 查询用户是否存在
        let user = match crate::models::users::Entity::find_by_name(&login.account)
            .one(db_pool.as_ref())
            .await
        {
            Ok(user) => user,
            Err(e) => return Err(AppError::DatabaseError(e.to_string())),
        };
        if user.is_none() {
            return Err(AppError::NotFound("用户不存在".to_string()));
        }
        let user = user.unwrap();
        match verify(&login.password, user.pass_word.as_str()) {
            Ok(true) => {
                // 登录成功
                Ok(ApiResponse::success("user", "密码登录").to_http_response())
            }
            Ok(false) => {
                // 登录失败
                Err(AppError::Unauthorized("密码错误".to_string()))
            }
            Err(e) => {
                // 登录失败
                Err(AppError::Unauthorized(e.to_string()))
            }
        }

        // log::info!("login_by_pwd{:?},{:?}", login, user);
        // Ok(ApiResponse::success("user", "一般登录").to_http_response())
    }
    pub async fn login_by_email(
        _db_pool: web::Data<DatabaseConnection>,
        email: EmailLogin,
    ) -> HttpResult {
        log::info!("login_by_email{:?}", email);

        Ok(ApiResponse::success("user", "邮箱").to_http_response())
    }
    pub async fn login_by_phone(
        _db_pool: web::Data<DatabaseConnection>,
        phone: PhoneLogin,
    ) -> HttpResult {
        log::info!("login_by_phone{:?}", phone);

        Ok(ApiResponse::success("user", "手机号").to_http_response())
    }
    pub async fn login_by_oauth(
        _db_pool: web::Data<DatabaseConnection>,
        oauth: OAuthLogin,
    ) -> HttpResult {
        log::info!("login_by_oauth{:?}", oauth);

        Ok(ApiResponse::success("user", "第三方").to_http_response())
    }
}
