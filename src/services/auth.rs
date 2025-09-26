use crate::config::AppError;
use crate::dto::user::{EmailLogin, OAuthLogin, PasswordLogin, PhoneLogin, ValidationErrorJson};
use crate::models::users::ActiveModel;
use crate::models::{roles, user_roles};
use crate::utils::crypto_pwd::{hash, verify};
use crate::utils::jwt::generate_jwt;
use crate::{ApiResponse, SseNotifier};
use crate::{HttpResult, RegisterResponse};
use actix_web::cookie::{Cookie, SameSite, time::Duration as ActixDuration};
use actix_web::{HttpResponse, web};
use chrono::Utc;
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, TransactionTrait,
};
use serde_json::json;
use uuid::Uuid;
use validator::Validate;
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
        // —— 1. 开事务 ——
        let txn = db_pool
            .begin()
            .await
            .map_err(|_| AppError::DatabaseError(String::from("Begin transaction failed")))?;

        // —— 2. 插用户 ——
        let user_am = ActiveModel {
            user_name: Set(user_name.clone()),
            pass_word: Set(password_hash),
            uuid: Set(Uuid::new_v4().to_string()),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        };
        let user = user_am
            .insert(&txn)
            .await
            .map_err(|_| AppError::DatabaseError(String::from("Insert user failed")))?;

        // —— 3. 查 VIEWER 角色 id ——
        let viewer_role = roles::Entity::find()
            .filter(roles::Column::Code.eq("SUPER_ADMIN"))
            .one(&txn)
            .await
            .map_err(|_| AppError::DatabaseError("VIEWER role not found".into()))?
            .ok_or_else(|| AppError::DatabaseError("VIEWER role not found".into()))?;

        // —— 4. 绑角色（is_primary = true） ——
        let user_role_am = user_roles::ActiveModel {
            user_id: Set(Some(user.id)),
            role_id: Set(Some(viewer_role.id)),
            is_primary: Set(Some(true)),
            created_at: Set(Utc::now()),
            ..Default::default()
        };
        user_role_am
            .insert(&txn)
            .await
            .map_err(|_| AppError::DatabaseError("Insert user role failed".into()))?;

        // —— 5. 提交事务 ——
        txn.commit()
            .await
            .map_err(|_| AppError::DatabaseError("Commit transaction failed".into()))?;
        // —— 6. 原通知逻辑保持不动 ——
        let notification = serde_json::json!({
            "event": "user_updated",
            "data": {
                "user_id": user.id,
                "updated_fields": { "username": &user.user_name }
            }
        });
        notifier.notify(&notification.to_string());
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
                let token = generate_jwt(
                    &db_pool,
                    &user,
                    "uZr0aHV8Z2dRa1NmYnJ0aXN0aGViZXN0a2V5",
                    3600,
                )
                .await?;
                log::info!("login_by_pwd token: {:?}", token);
                // 2. 构造 Cookie
                let cookie = Cookie::build("access_token", token)
                    .http_only(true) // 防 XSS
                    .same_site(SameSite::Strict) // 防 CSRF
                    .secure(true) // 生产必须 true，本地可 false
                    .max_age(ActixDuration::hours(1))
                    .path("/") // 全局可用
                    .finish();
                Ok(HttpResponse::Ok()
                    .cookie(cookie) // ← 关键：把 Cookie 塞进响应
                    .json(json!({
                        "code": 200,
                        "message": "密码登录成功",
                        "data":user
                    })))
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
