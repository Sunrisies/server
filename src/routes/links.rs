use crate::handlers::links::external_links_routes::{
    create_external_links_handler, delete_external_links_handler, get_external_links_all_handler,
    get_external_links_handler,
};
use actix_web::web;

pub fn config_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        // 管理接口（需要认证）
        web::scope("/v1/links")
            .route("", web::post().to(create_external_links_handler))
            .route("", web::get().to(get_external_links_all_handler))
            .route("/{id}", web::get().to(get_external_links_handler))
            .route("/{id}", web::delete().to(delete_external_links_handler)),
    );
}
