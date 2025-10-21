use actix_web::{Error, HttpRequest, HttpResponse, rt, web};
use actix_ws::AggregatedMessage;
use chrono::Utc;

use actix_ws::{Message, Session};
use futures_util::StreamExt;
use sea_orm::{ActiveModelTrait as _, ActiveValue::Set, DatabaseConnection};
use serde_json;
use tokio::sync::Mutex;

use crate::{
    models::room_messages,
    utils::websocket::{BroadcastMessage, ChatServer, ClientMessage},
};
pub async fn echo(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let (res, mut session, stream) = actix_ws::handle(&req, stream)?;

    let mut stream = stream
        .aggregate_continuations()
        .max_continuation_size(2_usize.pow(20));

    rt::spawn(async move {
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(AggregatedMessage::Text(text)) => {
                    println!("Received text message: {}", text);
                    session.text(text).await.unwrap();
                }

                Ok(AggregatedMessage::Binary(bin)) => {
                    println!("Received binary message: {:x?}", bin);
                    session.binary(bin).await.unwrap();
                }

                Ok(AggregatedMessage::Ping(msg)) => {
                    println!(
                        "Received ping message: {:x?}, responding with pong message",
                        msg
                    );
                    session.pong(&msg).await.unwrap();
                }

                _ => {}
            }
        }
    });

    Ok(res)
}

// WebSocket路由处理函数
pub async fn chat_route(
    req: HttpRequest,
    stream: web::Payload,
    chat_server: web::Data<Mutex<ChatServer>>,
    db_pool: web::Data<DatabaseConnection>,
    path: web::Path<(String, String)>, // (room_id, user_id)
) -> Result<HttpResponse, Error> {
    println!(
        "New WebSocket connection - Room: {}, User: {}",
        path.0, path.1
    );

    let (room_name, user_name) = path.into_inner();

    // 建立WebSocket连接
    let (response, session, msg_stream) = actix_ws::handle(&req, stream)?;

    // 添加到聊天服务器
    let session_id = {
        let mut server = chat_server.lock().await;
        server.add_session(room_name.clone(), session.clone())
    };

    // 广播用户加入消息
    {
        let mut server = chat_server.lock().await;
        let user_count = server.get_room_user_count(&room_name);
        server
            .broadcast_system_message(
                &room_name,
                &format!("用户已加入房间。在线用户: {}", user_count),
            )
            .await;
    }
    let db_pool_clone = db_pool.clone();
    // 处理消息流
    actix_rt::spawn(handle_ws_messages(
        session,
        msg_stream,
        chat_server,
        user_name,
        session_id,
        db_pool_clone,
    ));

    Ok(response)
}

// 处理WebSocket消息
async fn handle_ws_messages(
    mut session: Session,
    mut msg_stream: actix_ws::MessageStream,
    chat_server: web::Data<Mutex<ChatServer>>,
    room_name: String,
    session_id: String,
    db_pool: web::Data<DatabaseConnection>,
) {
    // 从user_id中提取昵称

    while let Some(Ok(msg)) = msg_stream.next().await {
        match msg {
            Message::Text(text) => {
                println!(
                    "Received text message from  in room {}: {}",
                    room_name, text
                );

                // 解析客户端消息
                match serde_json::from_str::<ClientMessage>(&text) {
                    Ok(client_msg) => {
                        // 创建广播消息
                        let broadcast_msg = BroadcastMessage {
                            user_nickname: Some(
                                client_msg.user_nickname.unwrap_or("系统回复".to_string()),
                            ),
                            room_id: client_msg.room_id,
                            room_name: room_name.clone(),
                            message_type: client_msg.message_type,
                            content: client_msg.content,
                            file_url: client_msg.file_url,
                            file_name: client_msg.file_name,
                            file_size: client_msg.file_size,
                            retention_hours: client_msg.retention_hours.unwrap_or(24),
                            timestamp: chrono::Utc::now(),
                        };
                        let broadcast_msg_clone = broadcast_msg.clone();
                        // 广播消息到房间内的所有客户端
                        let mut server = chat_server.lock().await;
                        server.broadcast_user_message(broadcast_msg).await;

                        // 这里可以添加数据库保存逻辑
                        if let Err(e) =
                            save_message_to_db(db_pool.get_ref(), &broadcast_msg_clone).await
                        {
                            eprintln!("Failed to save message to database: {}", e);
                        }
                    }
                    Err(e) => {
                        println!("Failed to parse message: {}", e);

                        // 发送错误消息回客户端
                        let error_msg = serde_json::json!({
                            "error": "Invalid message format",
                            "details": e.to_string()
                        });

                        if let Ok(error_text) = serde_json::to_string(&error_msg) {
                            let _ = session.text(error_text).await;
                        }
                    }
                }
            }
            Message::Binary(bin) => {
                println!(
                    "Received binary message from in room {}: {} bytes",
                    room_name,
                    bin.len()
                );

                // 可以处理二进制消息，如图片/文件
                // 这里简单回显
                let _ = session.binary(bin).await;
            }
            Message::Ping(bytes) => {
                println!("Received ping from in room {}", room_name);
                let _ = session.pong(&bytes).await;
            }
            Message::Pong(_) => {
                // 忽略pong消息
            }
            Message::Close(reason) => {
                println!("WebSocket closed by  in room {}: {:?}", room_name, reason);
                break;
            }
            Message::Continuation(_) => {
                // 处理continuation帧
                println!("Received continuation frame from in room {}", room_name);
            }
            Message::Nop => {
                // 无操作
            }
        }
    }

    // 连接断开，从聊天服务器移除
    println!("WebSocket connection closed for  in room {}", room_name);

    {
        let mut server = chat_server.lock().await;
        server.remove_session(&room_name, &session_id);
        let user_count = server.get_room_user_count(&room_name);
        // 广播用户离开消息
        server
            .broadcast_system_message(
                &room_name,
                &format!("一位用户离开了房间。在线用户: {}", user_count),
            )
            .await;
    }
}

async fn save_message_to_db(
    db: &DatabaseConnection,
    msg: &BroadcastMessage,
) -> Result<(), sea_orm::DbErr> {
    // 根据房间号去查房间id
    log::info!("msg: {:?}", msg);
    let new_message = room_messages::ActiveModel {
        room_id: Set(Some(msg.room_id)),
        message_type: Set(msg.message_type.clone()),
        content: Set(msg.content.clone()),
        file_url: Set(msg.file_url.clone()),
        file_name: Set(msg.file_name.clone()),
        file_size: Set(msg.file_size),
        retention_hours: Set(Some(msg.retention_hours)),
        created_at: Set(Utc::now()),
        expires_at: Set(Utc::now()),
        ..Default::default()
    };

    new_message.insert(db).await?;
    Ok(())
}
