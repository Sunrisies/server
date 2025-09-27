use actix_web::web;

use crate::{
    auth::login,
    echo,
    handlers::{
        create_category, delete_category, get_categories, get_category_by_id, register,
        users::users_routes::get_users_handler,
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
                web::scope("/v1/users") // .route("", web::get().to(get_api_users_handler))
                    .route("/{uuid:.*}", web::get().to(get_users_handler)), // .route("/{uuid:.*}", web::put().to(put_demo))
                                                                            // .route("/{uuid:.*}", web::delete().to(delete_user_uuid)),
            )
            .service(web::scope("/v1/ws").route("", web::get().to(echo)))
            .service(web::scope("/v1/sse").route("/stream", web::get().to(sse_stream)))
            .service(
                web::scope("/v1/categories")
                    .route("", web::post().to(create_category))
                    .route("/{id}", web::get().to(get_category_by_id))
                    .route("", web::get().to(get_categories)) // .route("/{id}", web::put().to(handlers::category::update_category))
                    .route("/{id}", web::delete().to(delete_category)),
            ),
    );
}
