use crate::handlers::send_verification_code;
use actix_web::web;

pub fn config_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/v1/email").route(
        "/send-verification-code",
        web::get().to(send_verification_code),
    ));
}
