use actix_web::web;
use chrono::Utc;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, DatabaseConnection};
use uuid::Uuid;
use validator::Validate;

use crate::RegisterResponse;
use crate::config::AppError;
use crate::dto::user::ValidationErrorJson;
use crate::models::users::ActiveModel;
use crate::{ApiResponse, HttpResult, SseNotifier};

// 注册
#[utoipa::path(
    post,
    summary = "注册",
    path = "/api/v1/auth/register",
    request_body( content = RegisterResponse),
    responses(
        (status = 200, description = "添加成功", body = RegisterResponse),
        (status = 422,description = "校验失败", body = ApiResponse<ValidationErrorJson> )
    ),
)]
pub async fn register(
    db_pool: web::Data<DatabaseConnection>,
    user_data: web::Json<RegisterResponse>,
    notifier: web::Data<SseNotifier>,
) -> HttpResult {
    // 校验
    if let Err(errors) = user_data.validate() {
        let msg = ValidationErrorJson::from_validation_errors(&errors);
        return Ok(ApiResponse::from(AppError::ValidationError(msg)).to_http_response());
    }
    let RegisterResponse {
        user_name,
        pass_word,
    } = user_data.into_inner();
    println!("Validated user data: {:?}", user_name);
    println!("Validated user data: {:?}", pass_word);
    let new_user = ActiveModel {
        user_name: Set(user_name.to_string()),
        pass_word: Set(pass_word.to_string()),
        permissions: Set(Some("33333".to_string())), // 设置默认权限
        uuid: Set(Uuid::new_v4().to_string()),       // 生成唯一的UUID
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        ..Default::default()
    };
    match new_user.insert(db_pool.as_ref()).await {
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
        }
        Err(e) => println!("Error creating user: {}", e),
    }
    println!("Database connected");

    Ok(ApiResponse::success("添加成功", "添加成功").to_http_response())
}
