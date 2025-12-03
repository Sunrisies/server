use crate::handlers::posts::{
    create_post_handler, delete_post_handler, get_posts_all_handler, get_posts_handler,
    get_prev_next_handler, get_timeline_handler, update_post_handler,
};
use actix_web::web;

pub fn config_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/v1/posts")
            .route("/prevNext/{uuid:.*}", web::get().to(get_prev_next_handler))
            .route("/uploadTime", web::get().to(get_timeline_handler))
            .route("", web::get().to(get_posts_all_handler))
            .route("", web::post().to(create_post_handler))
            .route("/{uuid:.*}", web::get().to(get_posts_handler))
            .route("/{uuid:.*}", web::put().to(update_post_handler))
            .route("/{uuid:.*}", web::delete().to(delete_post_handler)),
    );
}
