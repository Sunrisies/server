use actix_cors::Cors;
use actix_web::{App, HttpServer, http, web};
use anyhow::{Context, Result};
use tokio::sync::Mutex;
use web_server::{
    SseNotifier,
    config::{init_logger, manager::CONFIG, write_to_file},
    config_routes, create_db_pool, init_route_registry,
    middleware::auth::Auth,
    services::{EmailService, EmailVerificationManager},
    utils::{perm_cache::load_perm_cache, websocket::ChatServer},
};

#[actix_web::main]
async fn main() -> Result<()> {
    init_logger(); // 初始化日志
    // let app_config = AppConfig::default();
    log::info!("app config: {:?}", CONFIG.database);
    // 初始化路由注册表 - 这行很重要！
    init_route_registry();

    // 打印所有注册的路由（调试用）
    // let routes = get_all_routes();
    // log::info!("Registered {} routes:", routes.len());
    // for route in routes {
    //     log::info!("  {} {} -> {}", route.method, route.path, route.permission);
    // }
    let db = create_db_pool()
        .await
        .context("Failed to connect to database")?;
    // 将db添加到应用数据中
    let db_pool = web::Data::new(db);
    load_perm_cache(&db_pool.clone()).await.unwrap();
    // 添加sse
    let notifier = web::Data::new(SseNotifier::new());
    let chat_server = web::Data::new(Mutex::new(ChatServer::new()));

    // 初始化邮件服务
    let email_service = web::Data::new(EmailService::default());

    // // 初始化邮件验证码管理器
    let email_verification_manager = web::Data::new(EmailVerificationManager::default());

    // // 启动邮件验证码清理任务
    email_verification_manager.start_cleanup_task();

    write_to_file(); // api_doc生成文件
    println!(
        "Server running on {}:{}",
        CONFIG.server.host, CONFIG.server.port
    );
    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://127.0.0.1:5502")
            .allowed_origin("https://blog.sunrise1024.top")
            .allowed_origin("http://localhost:12002")
            .allowed_methods(vec![
                http::Method::GET,
                http::Method::POST,
                http::Method::PUT,
                http::Method::DELETE,
                http::Method::OPTIONS,
            ])
            .allowed_headers(vec![
                http::header::AUTHORIZATION,
                http::header::ACCEPT,
                http::header::CONTENT_TYPE,
            ])
            .supports_credentials()
            .max_age(3600);
        App::new()
            .app_data(db_pool.clone())
            .app_data(notifier.clone())
            .app_data(chat_server.clone()) // 共享聊天服务器状态
            .app_data(email_service.clone()) // 添加邮件服务
            .app_data(email_verification_manager.clone()) // 添加邮件验证码管理器
            .configure(config_routes)
            .wrap(actix_web::middleware::Logger::default())
            .wrap(cors)
            .wrap(Auth)
    });

    server
        .bind((CONFIG.server.host.as_str(), CONFIG.server.port))?
        .on_connect(move |conn, _addr| {
            println!("New connection: {:?}", conn);
            println!("Remote address: {:?}", _addr);
        })
        .run()
        .await?;
    Ok(())
}
