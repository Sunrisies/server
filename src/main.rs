use actix_web::{App, HttpServer};
use anyhow::Result;
use chrono::Utc;
use sea_orm::ActiveModelTrait;
use sea_orm::ActiveValue::Set;
use server::models::user::{self};
use server::{config_routes, create_db_pool};

#[actix_web::main]
async fn main() -> Result<()> {
    let db = create_db_pool().await?;
    let new_user = user::ActiveModel {
        user_name: Set("1111".to_string()),
        pass_word: Set("2222".to_string()),
        permissions: Set(Some("3333".to_string())), // 设置默认权限
        uuid: Set("4444".to_string()),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        ..Default::default()
    };
    match new_user.insert(&db).await {
        Ok(_) => println!("User created successfully"),
        Err(e) => println!("Error creating user: {}", e),
    }
    println!("Database connected");
    let _ = HttpServer::new(|| App::new().configure(config_routes))
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
