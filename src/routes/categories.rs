use crate::handlers::category::categories_routes::{
    create_categories_handler, get_categories_all_handler, get_categories_handler,
};
use actix_web::web;

pub fn config_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/v1/categories")
            .route(
                "/{id}",
                web::put().to(crate::handlers::management::update_category),
            )
            .route("", web::post().to(create_categories_handler))
            .route("/{id}", web::get().to(get_categories_handler))
            .route("", web::get().to(get_categories_all_handler))
            .route(
                "/{id}",
                web::delete().to(crate::handlers::management::delete_category),
            ),
    );
}
