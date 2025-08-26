use actix_web::{App, HttpServer};
use server::config_routes;

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
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
