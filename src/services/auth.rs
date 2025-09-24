use actix_web::web;
use chrono::Utc;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, DatabaseConnection};
use uuid::Uuid;
use validator::Validate;

use crate::RegisterResponse;
use crate::SseNotifier;
use crate::config::AppError;
use crate::dto::user::ValidationErrorJson;
use crate::models::users::ActiveModel;
use crate::utils::hash;
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
}
