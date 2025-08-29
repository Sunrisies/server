use actix_web::{App, HttpServer, web};
use anyhow::Result;
use server::{config_routes, create_db_pool};

#[actix_web::main]
async fn main() -> Result<()> {
    let db = create_db_pool().await?;
    // 将db添加到应用数据中
    let db_pool = web::Data::new(db);

    let _ = HttpServer::new(move || {
        App::new()
            .app_data(db_pool.clone())
            .configure(config_routes)
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
