use actix_web::web;

use crate::{
    auth::login,
    echo,
    handlers::{
        category::categories_routes::{
            create_categories_handler, get_categories_all_handler, get_categories_handler,
        },
        posts::{get_posts_all_handler, posts_routes::get_posts_handler},
        register,
        tags::tags_routes::{create_tags_handler, delete_tags_handler, get_tags_all_handler},
        users::users_routes::{get_users_all_handler, get_users_handler},
    },
    sse_stream,
};
pub fn config_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(
                web::scope("/v1/auth")
                    .route("/register", web::post().to(register))
                    .route("/login", web::post().to(login)),
            )
            .service(
                web::scope("/v1/users")
                    .route("", web::get().to(get_users_all_handler))
                    .route("/{uuid:.*}", web::get().to(get_users_handler)), // .route("/{uuid:.*}", web::put().to(put_demo))
                                                                            // .route("/{uuid:.*}", web::delete().to(delete_user_uuid)),
            )
            .service(web::scope("/v1/ws").route("", web::get().to(echo)))
            .service(web::scope("/v1/sse").route("/stream", web::get().to(sse_stream)))
            .service(
                web::scope("/v1/categories")
                    .route("", web::post().to(create_categories_handler))
                    .route("/{id}", web::get().to(get_categories_handler))
                    .route("", web::get().to(get_categories_all_handler)), // .route("/{id}", web::put().to(handlers::category::update_category))
                                                                           // .route("/{id}", web::delete().to(delete_category)),
            )
            .service(
                web::scope("/v1/tags")
                    .route("", web::post().to(create_tags_handler)) // .route("/{id}", web::get().to(get_category_by_id))
                    .route("", web::get().to(get_tags_all_handler))
                    // .route("/{id}", web::put().to(handlers::category::update_category))
                    .route("/{uuid:.*}", web::delete().to(delete_tags_handler)),
            )
            .service(
                web::scope("/v1/posts")
                    .route("", web::get().to(get_posts_all_handler))
                    .route("/{uuid:.*}", web::get().to(get_posts_handler)),
            ),
    );
}
