pub mod auth;
pub mod categories;
pub mod email;
pub mod links;
pub mod posts;
pub mod rooms;
pub mod tags;
pub mod upload;
pub mod users;
mod version;
use actix_web::web;

use crate::routes::version::get_version;

pub fn config_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            // 认证路由
            .configure(auth::config_routes)
            // 用户路由
            .configure(users::config_routes)
            // 文章路由
            .configure(posts::config_routes)
            // 分类路由
            .configure(categories::config_routes)
            // 标签路由
            .configure(tags::config_routes)
            // 链接路由
            .configure(links::config_routes)
            // 上传路由
            .configure(upload::config_routes)
            // 邮件路由
            .configure(email::config_routes)
            // 房间路由
            .configure(rooms::config_routes)
            // 获取当前版本信息
            .route("/v1/version", web::get().to(get_version)),
    );
}
