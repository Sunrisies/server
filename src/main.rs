use tokio::sync::Mutex;

use actix_cors::Cors;
use actix_web::{App, HttpServer, web};
use anyhow::{Context, Result};
use server::{
    SseNotifier,
    config::{init_logger, write_to_file},
    config_routes, create_db_pool, get_all_routes, init_route_registry,
    middleware::auth::Auth,
    utils::{perm_cache::load_perm_cache, websocket::ChatServer},
};

#[actix_web::main]
async fn main() -> Result<()> {
    init_logger(); // 初始化日志
    // 初始化路由注册表 - 这行很重要！
    init_route_registry();

    // 打印所有注册的路由（调试用）
    let routes = get_all_routes();
    log::info!("Registered {} routes:", routes.len());
    for route in routes {
        log::info!("  {} {} -> {}", route.method, route.path, route.permission);
    }
    let db = create_db_pool()
        .await
        .context("Failed to connect to database")?;
    // 将db添加到应用数据中
    let db_pool = web::Data::new(db);
    load_perm_cache(&db_pool.clone()).await.unwrap();
    // 添加sse
    let notifier = web::Data::new(SseNotifier::new());
    let chat_server = web::Data::new(Mutex::new(ChatServer::new()));

    write_to_file(); // api_doc生成文件
    println!("Server running on http://127.0.0.1:2345");
    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://127.0.0.1:5502")
            .allowed_origin("http://127.0.0.1:12002")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec!["Content-Type", "Authorization", "ACCEPT"])
            .supports_credentials()
            .max_age(3600);
        App::new()
            .app_data(db_pool.clone())
            .app_data(notifier.clone())
            .app_data(chat_server.clone()) // 共享聊天服务器状态
            .configure(config_routes)
            .wrap(cors)
            .wrap(actix_web::middleware::Logger::default())
            .wrap(Auth)
    });

    server
        .bind(("0.0.0.0", 2345))?
        .on_connect(move |conn, _addr| {
            println!("New connection: {:?}", conn);
            println!("Remote address: {:?}", _addr);
        })
        .run()
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
