use actix_web::{Responder, web};
use serde::{Deserialize, Serialize};
use validator::Validate;
#[derive(Validate, Debug, Clone, Serialize, Deserialize)]
pub struct PaginationQuery {
    #[validate(range(min = 1, message = "页码必须大于1"))]
    pub page: Option<u64>,
    // 每页数量不能超过100
    #[validate(range(max = 10, message = "每页数量不能超过100"))]
    pub limit: Option<u64>,
}
#[derive(Validate, Debug, Default, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    #[validate(length(min = 5, max = 100, message = "用户名长度必须在5到100之间"))]
    pub user_name: String,
    #[validate(length(min = 6, max = 100, message = "密码长度必须在6到100之间"))]
    pub pass_word: String,
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
async fn post_demo(user_data: web::Json<LoginRequest>) -> impl Responder {
    format!(
        "添加 user_name{:?} pass_word{:?}!",
        user_data.user_name, user_data.pass_word
    )
}

// 修改
async fn put_demo(uuid: web::Path<String>, user_data: web::Json<LoginRequest>) -> impl Responder {
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
