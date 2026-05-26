use crate::handlers::__path_get_users_handler;
use crate::handlers::auth;
use crate::handlers::category::categories_routes::{
    __path_create_categories_handler, __path_delete_categories_handler, __path_get_categories_all,
    __path_get_categories_handler,
};
use crate::handlers::clipboard::{
    __path_create_file_handler, __path_create_text_handler, __path_delete_handler,
    __path_list_handler,
};
use crate::handlers::email::__path_send_verification_code;
use crate::handlers::images::{
    __path_delete_image_handler, __path_get_image_by_id_handler, __path_get_images_handler,
    __path_upload_image_handler,
};
use crate::handlers::links::external_links_routes::{
    __path_create_external_links_handler, __path_delete_external_links_handler,
    __path_get_external_links_all, __path_get_external_links_handler,
};
use crate::handlers::posts::{
    __path_create_post_handler, __path_delete_post_handler, __path_get_posts_all_handler,
    __path_get_posts_handler, __path_get_prev_next_handler, __path_get_timeline_handler,
    __path_update_post_handler,
};
use crate::handlers::room_messages::__path_get_room_messages_handler;
use crate::handlers::rooms::{__path_create_room_handler, __path_get_room_handler};
use crate::handlers::tags::tags_routes::__path_get_tags_handler;
use crate::handlers::tags::{
    __path_get_posts_by_tag_handler, __path_get_tags_with_count_handler,
    tags_routes::{__path_create_tags_handler, __path_delete_tags_handler, __path_get_tags_all},
};
use crate::handlers::upload::__path_upload_file_handler;
use crate::routes::version::__path_get_version;
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
        // 用户
        get_users_handler,
        // 文章
        get_posts_all_handler,
        get_timeline_handler,
        get_posts_handler,
        get_prev_next_handler,
        create_post_handler,
        update_post_handler,
        delete_post_handler,
        // 分类
        get_categories_all,
        get_categories_handler,
        create_categories_handler,
        delete_categories_handler,
        // 标签
        get_tags_all,
        get_tags_handler,
        create_tags_handler,
        delete_tags_handler,
        get_tags_with_count_handler,
        get_posts_by_tag_handler,
        // 外部链接
        get_external_links_all,
        get_external_links_handler,
        create_external_links_handler,
        delete_external_links_handler,
        // 云剪贴板
        create_text_handler,
        create_file_handler,
        list_handler,
        delete_handler,
        // 文件上传
        upload_file_handler,
        // 图片管理
        upload_image_handler,
        get_images_handler,
        get_image_by_id_handler,
        delete_image_handler,
        // 邮件
        send_verification_code,
        // 聊天室
        create_room_handler,
        get_room_handler,
        get_room_messages_handler,
        // 系统信息
        get_version,
    )
)]
pub struct ApiDoc;

// #[cfg(debug_assertions)]
pub fn write_to_file() {
    let openapi_json = ApiDoc::openapi().to_pretty_json().unwrap();
    let mut file = File::create("openapi.json").unwrap();
    writeln!(file, "{}", openapi_json).unwrap();
    log::info!("OpenAPI JSON written to openapi.json");
}
