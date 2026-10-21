use crate::handlers::{get_users_all_handler, get_users_handler};
use actix_web::web;

pub fn config_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/v1/users")
            .route(
                "/default-avatar.webp",
                web::get().to(crate::services::avatar::handler),
            )
            .route(
                "/default-avatar.svg",
                web::get().to(crate::services::avatar::legacy_handler),
            )
            .route("", web::get().to(get_users_all_handler))
            .route("/{uuid:.*}", web::get().to(get_users_handler)), // .route("/{uuid:.*}", web::put().to(put_demo))
                                                                    // .route("/{uuid:.*}", web::delete().to(delete_user_uuid)),
    );
}
