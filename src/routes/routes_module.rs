use actix_web::web;

use crate::{
    auth::login,
    echo,
    handlers::{
        category::categories_routes::{
            create_categories_handler, get_categories_all_handler, get_categories_handler,
        },
        posts::{
            create_post_handler, delete_post_handler, get_posts_all_handler, get_posts_handler,
            get_prev_next_handler, get_timeline_handler, update_post_handler,
        },
        register,
        room_messages::get_room_messages_handler,
        rooms::{create_room_handler, get_room_handler},
        tags::{
            get_posts_by_tag_handler, get_tags_with_count_handler,
            tags_routes::{create_tags_handler, delete_tags_handler, get_tags_all_handler},
        },
        upload::upload_file_handler,
        users::users_routes::{get_users_all_handler, get_users_handler},
    },
    sse_stream,
    ws::chat_route,
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
                    .route("/count", web::get().to(get_tags_with_count_handler)) // 新增这行
                    .route("/{id}/posts", web::get().to(get_posts_by_tag_handler))
                    // .route("/{id}", web::put().to(handlers::category::update_category))
                    .route("/{uuid:.*}", web::delete().to(delete_tags_handler)),
            )
            .service(
                web::scope("/v1/posts")
                    .route("/prevNext/{uuid:.*}", web::get().to(get_prev_next_handler))
                    .route("/uploadTime", web::get().to(get_timeline_handler))
                    .route("", web::get().to(get_posts_all_handler))
                    .route("", web::post().to(create_post_handler))
                    .route("/{uuid:.*}", web::get().to(get_posts_handler))
                    .route("/{uuid:.*}", web::put().to(update_post_handler))
                    .route("/{uuid:.*}", web::delete().to(delete_post_handler)),
            )
            .service(
                web::scope("/v1/rooms")
                    .route("/ws/{room_id}/{user_id}", web::get().to(chat_route))
                    .route("", web::post().to(create_room_handler))
                    .route("/{room_id}", web::get().to(get_room_handler))
                    .route(
                        "{room_id}/messages",
                        web::get().to(get_room_messages_handler),
                    ),
            )
            .service(web::scope("/v1/upload").route("", web::post().to(upload_file_handler))),
    );
}
