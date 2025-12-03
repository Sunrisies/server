use actix_web::web;

use crate::routes::{auth, categories, email, posts, rooms, tags, upload, users};

pub fn config_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
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
            // 上传路由
            .configure(upload::config_routes)
            // 邮件路由
            .configure(email::config_routes)
            // 房间路由
            .configure(rooms::config_routes),
    );
}
