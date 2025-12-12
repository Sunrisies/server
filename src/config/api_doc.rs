use crate::handlers::auth;
// use crate::handlers::category;
// use crate::handlers::users;
use crate::handlers::__path_get_users_handler;
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
        // crate::handlers::links::list_links_handler,
        // crate::handlers::links::click_link_handler,
        // crate::handlers::links::redirect_link_handler,
        // crate::handlers::links::update_link_handler,
        // crate::handlers::links::external_links_routes::create_external_links_handler,
        // crate::handlers::links::external_links_routes::delete_external_links_handler,
        // crate::handlers::links::external_links_routes::get_external_links_all_handler,
        // crate::handlers::links::external_links_routes::get_external_links_handler
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
