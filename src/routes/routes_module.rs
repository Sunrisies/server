use actix_web::web;

use crate::routes::user::{delete_demo, get_demo, get_demo_uuid};
use crate::routes::{echo, post_demo};

pub fn config_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(
                web::scope("/v1")
                    .route("/", web::get().to(get_demo))
                    .route("/{uuid:.*}", web::get().to(get_demo_uuid))
                    .route("/", web::post().to(post_demo))
                    // .route("/{uuid:.*}", web::put().to(put_demo))
                    .route("/{uuid:.*}", web::delete().to(delete_demo)),
            )
            .service(web::scope("/ws").route("", web::get().to(echo))),
    );
}
