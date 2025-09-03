use actix_cors::Cors;
use actix_web::{App, HttpServer, web};
use anyhow::{Context, Result};
use server::{SseNotifier, config_routes, create_db_pool};
#[actix_web::main]
async fn main() -> Result<()> {
    let db = create_db_pool()
        .await
        .context("Failed to connect to database")?;
    // 将db添加到应用数据中
    let db_pool = web::Data::new(db);
    // 添加sse
    let notifier = web::Data::new(SseNotifier::new());
    println!("Server running on http://127.0.0.1:2345");
    let _ = HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://127.0.0.1:5502")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec!["Content-Type", "Authorization", "ACCEPT"])
            .supports_credentials()
            .max_age(3600);
        App::new()
            .app_data(db_pool.clone())
            .app_data(notifier.clone())
            .configure(config_routes)
            .wrap(cors)
    })
    .bind(("0.0.0.0", 2345))?
    .on_connect(move |conn, _addr| {
        println!("New connection: {:?}", conn);
        println!("Remote address: {:?}", _addr);
    })
    .run()
    .await;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
