use crate::handlers::tags::{
    get_posts_by_tag_handler, get_tags_with_count_handler,
    tags_routes::{create_tags_handler, get_tags_all_handler},
};
use actix_web::web;

pub fn config_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/v1/tags")
            .route(
                "/{id}",
                web::put().to(crate::handlers::management::update_tag),
            )
            .route("", web::post().to(create_tags_handler)) // .route("/{id}", web::get().to(get_category_by_id))
            .route("", web::get().to(get_tags_all_handler))
            .route("/count", web::get().to(get_tags_with_count_handler)) // 新增这行
            .route("/{id}/posts", web::get().to(get_posts_by_tag_handler))
            // .route("/{id}", web::put().to(handlers::category::update_category))
            .route(
                "/{id}",
                web::delete().to(crate::handlers::management::delete_tag),
            ),
    );
}
