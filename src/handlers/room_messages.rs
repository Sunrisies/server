use crate::{HttpResult, config::AppError, models::room_messages};
use actix_web::web;
use sea_orm::{EntityTrait, prelude::*};

use crate::ApiResponse;

pub async fn get_room_messages_handler(
    db: web::Data<DatabaseConnection>,
    room_id: web::Path<String>,
) -> HttpResult {
    // 房间id由字符串转成i32
    let room_id = room_id
        .parse::<i32>()
        .map_err(|_| AppError::BadRequest("房间ID格式错误".to_string()))?;
    // 查询指定房间ID的消息
    let messages = room_messages::Entity::find()
        .filter(room_messages::Column::RoomId.eq(room_id))
        .all(db.as_ref())
        .await?;
    log::info!("messages: {:?}", messages);
    Ok(ApiResponse::success(messages, "获取房间信息成功").to_http_response())
}
