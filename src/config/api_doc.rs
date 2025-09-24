use crate::handlers::auth;
use crate::handlers::category;
use crate::services::users;
use std::fs::File;
use std::io::Write;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Rust Web API",
        version = "1.0",
        description = "一个简单的Rust web API",
        terms_of_service = "https://www.rust-web-api.com/terms",
        contact(
            name = "Sunrisies",
            email = "3266420686@qq.com",
            url = "https://github.com/Sunrisies/rust-web"
        ),
    ),
    paths(
        // 权限模块的
        auth::register, // 注册
        users::get_users, // 获取用户列表
        users::get_user_uuid, // 获取单个用户
        users::delete_user_uuid, // 删除用户
        category::create_category, // 创建分类
        category::get_categories, // 获取分类列表
        // category::get_category_uuid, // 获取单个分类
    )
)]
pub struct ApiDoc;

// #[cfg(debug_assertions)]
pub fn write_to_file() {
    let openapi_json = ApiDoc::openapi().to_pretty_json().unwrap();
    let mut file = File::create("openapi.json").unwrap();
    writeln!(file, "{}", openapi_json).unwrap();
    log::info!("OpenAPI JSON written to openapi.json");
    // log::info!("{}112112312", openapi_json);
}
