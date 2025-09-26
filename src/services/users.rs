use actix_web::HttpResponse;
use sea_orm::{DatabaseConnection, EntityTrait, QuerySelect};

// pub struct CategyService;
pub struct UserService;
use crate::{config::AppError, models::users::Entity as UserEntity};
impl UserService {
    pub async fn get_users(
        db_pool: &DatabaseConnection,
        page: u64,
        limit: u64,
    ) -> Result<HttpResponse, AppError> {
        match UserEntity::find()
            .limit(limit)
            .offset((page - 1) * limit)
            .all(db_pool)
            .await
        {
            Ok(data) => Ok(HttpResponse::Ok().json(data)),
            Err(e) => {
                println!("Database query error: {}", e);
                Err(AppError::DatabaseConnectionError(e.to_string()))
            }
        }
    }
}
