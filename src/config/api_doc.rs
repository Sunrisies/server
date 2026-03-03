use crate::handlers::__path_get_users_handler;
use crate::handlers::auth;
use crate::handlers::category::categories_routes::{
    __path_delete_categories_handler, __path_get_categories_all, __path_get_categories_handler,
};
use crate::handlers::links::external_links_routes::{
    __path_delete_external_links_handler, __path_get_external_links_all,
    __path_get_external_links_handler,
};
use crate::handlers::posts::{
    __path_create_post_handler, __path_delete_post_handler, __path_get_posts_all_handler,
    __path_get_posts_handler, __path_get_prev_next_handler, __path_get_timeline_handler,
    __path_update_post_handler,
};
use crate::handlers::tags::tags_routes::{
    __path_delete_tags_handler, __path_get_tags_all, __path_get_tags_handler,
};
use std::fs::File;
use std::io::Write;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Rust Web API",
        version = "1.0",
        description = "一个简单的Rust web API",
        terms_of_service = "https://www.rust-web-api.com/terms",
        contact(
            name = "Sunrisies",
            email = "3266420686@qq.com",
            url = "https://github.com/Sunrisies/rust-web"
        ),
    ),
    paths(
        auth::register,
        auth::login,
        get_users_handler,
        get_posts_all_handler,
        get_timeline_handler,
        get_posts_handler,
        get_prev_next_handler,
        create_post_handler,
        update_post_handler,
        delete_post_handler,
        get_external_links_handler,
        get_categories_handler,
        get_tags_handler,
        get_external_links_all,
        get_categories_all,
        get_tags_all,
        delete_external_links_handler,
        delete_tags_handler,
        delete_categories_handler
    )
)]
pub struct ApiDoc;

// #[cfg(debug_assertions)]
pub fn write_to_file() {
    let openapi_json = ApiDoc::openapi().to_pretty_json().unwrap();
    let mut file = File::create("openapi.json").unwrap();
    writeln!(file, "{}", openapi_json).unwrap();
    log::info!("OpenAPI JSON written to openapi.json");
    // log::info!("{}112112312", openapi_json);
}
