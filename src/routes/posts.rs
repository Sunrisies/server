use crate::handlers::posts::{
    create_post_handler, delete_post_handler, get_post_versions_handler, get_posts_all_handler,
    get_posts_handler, get_prev_next_handler, get_timeline_handler, update_post_handler,
    view_post_handler,
};
use actix_web::web;

pub fn config_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/v1/posts")
            .route(
                "/category-cover/{id}.svg",
                web::get().to(crate::services::category_cover::handler),
            )
            .route(
                "/content-version",
                web::get().to(crate::services::content_revision::handler),
            )
            .route("/{uuid}/prevnext", web::get().to(get_prev_next_handler))
            .route("/prevNext/{uuid:.*}", web::get().to(get_prev_next_handler))
            .route("/uploadTime", web::get().to(get_timeline_handler))
            .route("/{uuid}/view", web::post().to(view_post_handler))
            .route("/{uuid}/versions", web::get().to(get_post_versions_handler))
            .route("", web::get().to(get_posts_all_handler))
            .route("", web::post().to(create_post_handler))
            .route("/{uuid:.*}", web::get().to(get_posts_handler))
            .route("/{uuid:.*}", web::put().to(update_post_handler))
            .route("/{uuid:.*}", web::delete().to(delete_post_handler)),
    );
}
