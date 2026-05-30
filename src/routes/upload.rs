use crate::handlers::upload::{delete_upload_handler, list_uploads_handler, upload_file_handler};
use actix_web::web;

pub fn config_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/v1/upload").route("", web::post().to(upload_file_handler)));
    cfg.service(
        web::scope("/v1/uploads")
            .route("", web::get().to(list_uploads_handler))
            .route("/{uuid}", web::delete().to(delete_upload_handler)),
    );
}
