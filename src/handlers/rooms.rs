use crate::{HttpResult, config::AppError};
use actix_web::{HttpResponse, Result, web};
use chrono::Utc;
use sea_orm::{ActiveValue::Set, EntityTrait, prelude::*};
use serde::{Deserialize, Serialize};

use crate::{ApiResponse, models::rooms};

#[derive(Deserialize)]
pub struct CreateRoomRequest {
    pub name: String,
    pub description: Option<String>,
    pub max_users: Option<i32>,
}

#[derive(Serialize)]
pub struct RoomResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub max_users: i32,
    pub user_count: i32,
}

pub async fn create_room_handler(
    db: web::Data<DatabaseConnection>,
    room_data: web::Json<CreateRoomRequest>,
) -> HttpResult {
    // 先检查房间是否已存在
    let existing_room = rooms::Entity::find()
        .filter(rooms::Column::Name.eq(room_data.name.clone()))
        .one(db.get_ref())
        .await
        .map_err(|_| AppError::DatabaseError(String::from("检查房间是否存在失败")))?;
    if existing_room.is_some() {
        return Ok(ApiResponse::<()>::success_msg("房间已存在").to_http_response());
    }
    let new_room = rooms::ActiveModel {
        uuid: Set(Uuid::new_v4().to_string()),
        name: Set(room_data.name.clone()),
        description: Set(room_data.description.clone()),
        max_users: Set(Some(room_data.max_users.unwrap_or(100))),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        ..Default::default()
    };
    let room = new_room
        .insert(db.get_ref())
        .await
        .map_err(|_| AppError::DatabaseError(String::from("插入房间失败")))?;

    Ok(ApiResponse::success(room, "创建房间成功").to_http_response())
}

pub async fn get_room_handler(
    db_pool: web::Data<DatabaseConnection>,
    room_name: web::Path<String>,
) -> Result<HttpResponse> {
    let existing_room = rooms::Entity::find()
        .filter(rooms::Column::Name.eq(room_name.clone()))
        .one(db_pool.get_ref())
        .await
        .map_err(|_| AppError::DatabaseError(String::from("检查房间是否存在失败")))?;
    Ok(ApiResponse::success(existing_room, "查询成功").to_http_response())
    // Ok(HttpResponse::Ok().json("Room created successfully"))
}
