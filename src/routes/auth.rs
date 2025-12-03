use crate::handlers::auth::{login, register};
use actix_web::web;

pub fn config_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/v1/auth")
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login)),
    );
}
