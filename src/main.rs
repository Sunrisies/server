use std::time::Duration;

use actix_web::{App, HttpServer};
use sea_orm::{ConnectOptions, Database};
use server::config_routes;

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut opt = ConnectOptions::new("postgres://postgres:server123456@127.0.0.1:5432/database");
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true)
        // .sqlx_logging_level(log::LevelFilter::Info)
        .set_schema_search_path("my_schema"); // Setting default PostgreSQL schema

    let _db = Database::connect(opt).await;

    HttpServer::new(|| App::new().configure(config_routes))
        .bind(("0.0.0.0", 2345))?
        .run()
        .await
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
