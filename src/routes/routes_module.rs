use actix_web::{Responder, web};
use chrono::Utc;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, DatabaseConnection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::dto::user::ValidationErrorMsg;
use crate::{RegisterResponse, user};

#[derive(Validate, Debug, Clone, Serialize, Deserialize)]
pub struct PaginationQuery {
    #[validate(range(min = 1, message = "页码必须大于1"))]
    pub page: Option<u64>,
    // 每页数量不能超过100
    #[validate(range(max = 10, message = "每页数量不能超过100"))]
    pub limit: Option<u64>,
}

// 获取
async fn get_demo(query: web::Query<PaginationQuery>) -> impl Responder {
    format!(
        "获取 page{:?} limit{:?}!",
        query.page.unwrap_or(1),
        query.limit.unwrap_or(10)
    )
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
async fn put_demo(
    uuid: web::Path<String>,
    user_data: web::Json<RegisterResponse>,
) -> impl Responder {
    format!(
        "修改 uuid:{:?} user_name{:?} pass_word{:?}!",
        uuid, user_data.user_name, user_data.pass_word
    )
}

// 删除
async fn delete_demo(uuid: web::Path<String>) -> impl Responder {
    format!("删除 uuid:{:?}!", uuid)
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
                .route("/{uuid:.*}", web::put().to(put_demo))
                .route("/{uuid:.*}", web::delete().to(delete_demo)),
        ),
    );
}
