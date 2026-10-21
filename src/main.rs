use actix_cors::Cors;
use actix_web::{
    App, HttpServer,
    http::{self, header},
    web,
};
use anyhow::{Context, Result};
use tokio::sync::Mutex;
use url::Url;
use web_server::utils::json_err_map;
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
    web_server::observability::init_pool_timing().context("Failed to initialize pool timing")?;
    // Force secret validation before starting; never log configuration secrets.
    let _ = &CONFIG.jwt;
    // 初始化路由注册表 - 这行很重要！
    let _ = init_route_registry();

    let db = create_db_pool()
        .await
        .context("Failed to connect to database")?;
    // 将db添加到应用数据中
    web_server::utils::token_revocation::initialize(&db).await?;
    web_server::utils::login_limit::initialize(&db).await?;
    web_server::services::content_revision::initialize(&db).await?;
    web_server::services::post_versions::initialize(&db).await?;
    web_server::services::site_features::initialize(&db).await?;
    web_server::services::upload_jobs::initialize(&db).await?;
    web_server::dropbox::initialize(&db).await?;
    web_server::dropbox::start_cleanup_task(db.clone());
    web_server::services::images::ImageService::backfill_legacy(&db).await?;
    web_server::services::upload_jobs::start_recovery(db.clone());
    web_server::observability::sample_pool(&db);
    let db_pool = web::Data::new(db);
    if let Ok(bind) = std::env::var("METRICS_BIND") {
        let addr = web_server::observability::metrics_bind(&bind).map_err(anyhow::Error::msg)?;
        let metrics = web_server::observability::serve_metrics(addr).await?;
        actix_web::rt::spawn(metrics);
        log::info!("Metrics listening on {addr}/metrics");
    }
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
    log::info!(
        "Server running on {}:{}",
        CONFIG.server.host,
        CONFIG.server.port
    );
    let server = HttpServer::new(move || {
        web_server::observability::start_worker_probe();
        let cors = Cors::default()
            .allowed_methods(vec![
                http::Method::GET,
                http::Method::POST,
                http::Method::PUT,
                http::Method::DELETE,
                http::Method::OPTIONS,
            ])
            .allowed_headers(vec![
                header::AUTHORIZATION,
                header::ACCEPT,
                header::CONTENT_TYPE,
            ])
            .supports_credentials()
            .max_age(3600)
            .allowed_origin_fn(|origin, _req| {
                let Ok(origin_str) = origin.to_str() else {
                    return false;
                };

                // 解析 Origin 得到主机名（host）
                if let Ok(url) = Url::parse(origin_str)
                    && let Some(host) = url.host_str()
                {
                    // 1. 允许本地开发来源（127.0.0.1 或 localhost，任意端口）
                    if host == "127.0.0.1" || host == "localhost" {
                        return true;
                    }
                    // 2. 允许线上域名：sunrise1024.top 及其所有子域名
                    if host == "sunrise1024.top" || host.ends_with(".sunrise1024.top") {
                        return true;
                    }
                }
                false
            });
        App::new()
            .wrap(Auth::new(db_pool.clone()))
            .app_data(web::JsonConfig::default().error_handler(|err, _| json_err_map(err)))
            .app_data(db_pool.clone())
            .app_data(notifier.clone())
            .app_data(chat_server.clone()) // 共享聊天服务器状态
            .app_data(email_service.clone()) // 添加邮件服务
            .app_data(email_verification_manager.clone()) // 添加邮件验证码管理器
            .configure(config_routes)
            .wrap(actix_web::middleware::Logger::default().log_target("http_access"))
            .wrap(cors)
            .wrap(web_server::observability::RequestMetrics)
    });

    server
        .bind((CONFIG.server.host.as_str(), CONFIG.server.port))?
        .on_connect(move |conn, addr| {
            log::info!(target: "connection", "New connection: {conn:?}");
            log::info!(target: "connection", "Remote address: {addr:?}");
        })
        .run()
        .await?;
    Ok(())
}
