use crate::handlers::upload::upload_file_handler;
use actix_web::web;

pub fn config_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/v1/upload").route("", web::post().to(upload_file_handler)));
}
