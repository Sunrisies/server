use actix_web::web;

use crate::{
    handlers::{
        room_messages::get_room_messages_handler,
        rooms::{create_room_handler, get_room_handler},
    },
    ws::chat_route,
};

pub fn config_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/v1/rooms")
            .route("/ws/{room_id}/{user_id}", web::get().to(chat_route))
            .route("", web::post().to(create_room_handler))
            .route("/{room_id}", web::get().to(get_room_handler))
            .route(
                "{room_id}/messages",
                web::get().to(get_room_messages_handler),
            ),
    );
}
