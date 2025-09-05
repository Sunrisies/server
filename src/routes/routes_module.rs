use actix_web::web;

use crate::{
    echo,
    handlers::category::{create_category, delete_category, get_categories, get_category_by_id},
    post_demo, sse_stream,
    users::{delete_demo, get_demo, get_demo_uuid},
};
pub fn config_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(web::scope("/v1/auth").route("/", web::post().to(post_demo)))
            .service(
                web::scope("/v1/users")
                    .route("/", web::get().to(get_demo))
                    .route("/{uuid:.*}", web::get().to(get_demo_uuid))
                    // .route("/{uuid:.*}", web::put().to(put_demo))
                    .route("/{uuid:.*}", web::delete().to(delete_demo)),
            )
            .service(web::scope("/ws").route("", web::get().to(echo)))
            .service(web::scope("/sse").route("/stream", web::get().to(sse_stream)))
            .service(
                web::scope("/v1/categories")
                    .route("", web::post().to(create_category))
                    .route("/{id}", web::get().to(get_category_by_id))
                    .route("", web::get().to(get_categories)) // .route("/{id}", web::put().to(handlers::category::update_category))
                    .route("/{id}", web::delete().to(delete_category)),
            ),
    );
}
