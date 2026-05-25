use crate::{HttpResult, config::AppError, models::room_messages};
use actix_web::web;
use sea_orm::{EntityTrait, prelude::*};

use crate::ApiResponse;

#[utoipa::path(
    get,
    path = "/api/v1/rooms/{room_id}/messages",
    tag = "聊天室",
    summary = "获取聊天室消息",
    description = "根据房间ID获取该聊天室的所有消息",
    params(
        ("room_id" = i32, Path, description = "房间ID")
    ),
    responses(
        (status = 200, description = "获取成功", body = crate::ApiResponse<Vec<crate::models::room_messages::Model>>),
        (status = 400, description = "房间ID格式错误", body = crate::ApiResponse<crate::EmptyResponse>),
        (status = 500, description = "服务器内部错误", body = crate::ApiResponse<crate::EmptyResponse>)
    )
)]
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
    log::info!("messages: {messages:?}");
    Ok(ApiResponse::success(messages, "获取房间信息成功").to_http_response())
}
