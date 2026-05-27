use crate::handlers::clipboard::{
    auth_channel_handler, create_channel_handler, create_file_handler, create_text_handler,
    delete_handler, list_handler,
};
use actix_web::web;

pub fn config_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/v1/clipboard")
            // 频道管理（无需 token）
            .route("/channel", web::post().to(create_channel_handler))
            .route("/channel/auth", web::post().to(auth_channel_handler))
            // 内容操作（需要 token）
            .route("/text", web::post().to(create_text_handler))
            .route("/file", web::post().to(create_file_handler))
            .route("", web::get().to(list_handler))
            .route("/{uuid}", web::delete().to(delete_handler)),
    );
}
