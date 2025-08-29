use actix_web::{HttpResponse, Responder, web};
use chrono::Utc;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, ModelTrait, QuerySelect};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::dto::user::{UpdateUserRequest, ValidationErrorMsg};
use crate::{
    RegisterResponse,
    user::{self, Entity as UserEntity},
};

#[derive(Validate, Debug, Clone, Serialize, Deserialize)]
pub struct PaginationQuery {
    #[validate(range(min = 1, message = "页码必须大于1"))]
    pub page: Option<u64>,
    // 每页数量不能超过100
    #[validate(range(max = 10, message = "每页数量不能超过100"))]
    pub limit: Option<u64>,
}

// 获取
async fn get_demo(
    db_pool: web::Data<DatabaseConnection>,
    query: web::Query<PaginationQuery>,
) -> impl Responder {
    let PaginationQuery { page, limit } = query.into_inner();

    match user::Entity::find()
        .limit(limit.unwrap_or(10).min(100))
        .offset((page.unwrap_or(1) - 1) * limit.unwrap_or(10).min(100))
        .all(db_pool.as_ref())
        .await
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(e) => {
            println!("Database query error: {}", e);
            HttpResponse::InternalServerError().json("Database query error")
        }
    }
}
// 新增
async fn post_demo(
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

// 修改
async fn _put_demo(
    db: web::Data<DatabaseConnection>,
    uuid: web::Path<String>,
    user_data: web::Json<UpdateUserRequest>,
) -> Result<impl Responder, HttpResponse> {
    // 验证UUID格式
    let uuid_result = Uuid::parse_str(&uuid);
    let uuid = match uuid_result {
        Ok(u) => u,
        Err(_) => return Err(HttpResponse::InternalServerError().json("Database query error")),
    };

    // 先找到用户
    let existing_user = match UserEntity::find_by_uuid(&uuid.to_string())
        .one(db.as_ref())
        .await
    {
        Ok(u) => u,
        Err(_e) => return Err(HttpResponse::InternalServerError().json("Database query error")),
    };
    let existing_user = existing_user
        .ok_or_else(|| HttpResponse::NotFound().json(format!("ID为{}的用户不存在", uuid)))?;
    let mut user_active: user::ActiveModel = existing_user.into();
    user_active.user_name = Set(user_data.user_name.to_string());
    user_active.image = Set(user_data.image.clone());
    user_active.updated_at = Set(Utc::now());
    let updated_user = user_active
        .update(db.as_ref())
        .await
        .map_err(|e| HttpResponse::InternalServerError().json(format!("更新失败: {}", e)))?;
    Ok(HttpResponse::Ok().json(updated_user))
    // "修改 uuid:{:?}!".to_string()
}

// 删除
async fn delete_demo(db: web::Data<DatabaseConnection>, uuid: web::Path<String>) -> impl Responder {
    // 验证UUID格式
    let uuid_result = Uuid::parse_str(&uuid);
    let uuid = match uuid_result {
        Ok(u) => u,
        Err(_) => return HttpResponse::InternalServerError().json("Database query error"),
    };

    // 先找到用户
    let existing_user = match UserEntity::find_by_uuid(&uuid.to_string())
        .one(db.as_ref())
        .await
    {
        Ok(u) => u,
        Err(_e) => return HttpResponse::InternalServerError().json("Database query error"),
    };

    match existing_user {
        Some(user) => {
            // 删除用户
            match user.delete(db.as_ref()).await {
                Ok(_) => HttpResponse::Ok().json("删除成功"),
                Err(_e) => HttpResponse::InternalServerError().json("Database query error"),
            }
        }
        None => HttpResponse::NotFound().json("User not found"),
    }
}

// 获取单个
async fn get_demo_uuid(uuid: web::Path<String>) -> impl Responder {
    format!("获取 uuid:{:?}!", uuid)
}
pub fn config_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api").service(
            web::scope("/v1")
                .route("/", web::get().to(get_demo))
                .route("/{uuid:.*}", web::get().to(get_demo_uuid))
                .route("/", web::post().to(post_demo))
                // .route("/{uuid:.*}", web::put().to(put_demo))
                .route("/{uuid:.*}", web::delete().to(delete_demo)),
        ),
    );
}
