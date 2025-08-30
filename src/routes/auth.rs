use actix_web::{Responder, web};
use chrono::Utc;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, DatabaseConnection};
use uuid::Uuid;
use validator::Validate;

use crate::dto::user::ValidationErrorMsg;
use crate::{
    RegisterResponse,
    user::{self},
};

// 新增
pub async fn post_demo(
    db_pool: web::Data<DatabaseConnection>,
    user_data: web::Json<RegisterResponse>,
) -> impl Responder {
    // 校验
    if let Err(errors) = user_data.validate() {
        let msg = ValidationErrorMsg(&errors);
        println!("Validation errors:-- {}", msg);
        return format!("Validation errors: {}", msg);
    }
    let RegisterResponse {
        user_name,
        pass_word,
    } = user_data.into_inner();
    println!("Validated user data: {:?}", user_name);
    println!("Validated user data: {:?}", pass_word);
    let new_user = user::ActiveModel {
        user_name: Set(user_name.to_string()),
        pass_word: Set(pass_word.to_string()),
        permissions: Set(Some("33333".to_string())), // 设置默认权限
        uuid: Set(Uuid::new_v4().to_string()),       // 生成唯一的UUID
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        ..Default::default()
    };
    match new_user.insert(db_pool.as_ref()).await {
        Ok(_) => println!("User created successfully"),
        Err(e) => println!("Error creating user: {}", e),
    }
    println!("Database connected");
    "添加成功".to_string()
}
