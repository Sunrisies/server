use crate::handlers::images::{
    delete_image_handler, get_image_by_id_handler, get_images_handler, upload_image_handler,
};
use actix_web::web;

pub fn config_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/v1/images")
            .route("/upload", web::post().to(upload_image_handler))
            .route("", web::get().to(get_images_handler))
            .route("/{id}", web::get().to(get_image_by_id_handler))
            .route("/{id}", web::delete().to(delete_image_handler)),
    );
}
