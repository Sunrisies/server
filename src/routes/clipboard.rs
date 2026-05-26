use crate::handlers::clipboard::{
    create_file_handler, create_text_handler, delete_handler, list_handler,
};
use actix_web::web;

pub fn config_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/v1/clipboard")
            .route("/text", web::post().to(create_text_handler))
            .route("/file", web::post().to(create_file_handler))
            .route("", web::get().to(list_handler))
            .route("/{uuid}", web::delete().to(delete_handler)),
    );
}
