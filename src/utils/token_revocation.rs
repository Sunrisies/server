use crate::{config::AppError, models::revoked_tokens};
use chrono::{TimeZone, Utc};
use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter, Schema, Set,
};
use sha2::{Digest, Sha256};

pub async fn initialize(db: &DatabaseConnection) -> Result<(), AppError> {
    // Pre-created tables allow runtime DB users to operate without CREATE privilege.
    if revoked_tokens::Entity::find().one(db).await.is_ok() {
        return Ok(());
    }
    let mut table =
        Schema::new(db.get_database_backend()).create_table_from_entity(revoked_tokens::Entity);
    table.if_not_exists();
    db.execute(db.get_database_backend().build(&table))
        .await
        .map_err(|_| AppError::DatabaseError("令牌撤销存储初始化失败".into()))?;
    Ok(())
}
fn fingerprint(token: &str) -> String {
    format!("{:x}", Sha256::digest(token.as_bytes()))
}

pub async fn is_revoked(db: &DatabaseConnection, token: &str) -> Result<bool, AppError> {
    Ok(revoked_tokens::Entity::find_by_id(fingerprint(token))
        .one(db)
        .await
        .map_err(|_| AppError::DatabaseError("会话验证暂不可用".into()))?
        .is_some())
}

pub async fn revoke(db: &DatabaseConnection, token: &str, expiry: i64) -> Result<(), AppError> {
    let expires_at = Utc
        .timestamp_opt(expiry, 0)
        .single()
        .ok_or_else(|| AppError::BadRequest("令牌有效期无效".into()))?;
    revoked_tokens::Entity::delete_many()
        .filter(revoked_tokens::Column::ExpiresAt.lt(Utc::now()))
        .exec(db)
        .await
        .map_err(|_| AppError::DatabaseError("会话注销暂不可用".into()))?;
    revoked_tokens::Entity::insert(revoked_tokens::ActiveModel {
        token_hash: Set(fingerprint(token)),
        expires_at: Set(expires_at),
    })
    .on_conflict(
        sea_orm::sea_query::OnConflict::column(revoked_tokens::Column::TokenHash)
            .do_nothing()
            .to_owned(),
    )
    .do_nothing()
    .exec(db)
    .await
    .map_err(|_| AppError::DatabaseError("会话注销暂不可用".into()))?;
    Ok(())
}

#[cfg(test)]
mod auth_revocation_tests {
    use super::*;
    use actix_web::{App, HttpResponse, cookie::Cookie, test, web};
    use sea_orm::{ActiveModelTrait, IntoActiveModel};
    #[actix_web::test]
    async fn auth_logout_revokes_copied_tokens_and_fresh_logins_remain_valid() {
        let db = crate::services::auth_test::tests::setup_test_db().await;
        let user = crate::services::auth::AuthService::register(
            web::Data::new(db.clone()),
            web::Json(crate::dto::user::RegisterResponse {
                user_name: "logoutuser".into(),
                pass_word: "password123".into(),
            }),
            web::Data::new(crate::SseNotifier::new()),
        )
        .await
        .unwrap();
        let role = crate::models::roles::Entity::find()
            .filter(crate::models::roles::Column::Code.eq("ADMIN"))
            .one(&db)
            .await
            .unwrap()
            .unwrap();
        let mut assignment = crate::models::user_roles::Entity::find()
            .filter(crate::models::user_roles::Column::UserId.eq(user.id))
            .one(&db)
            .await
            .unwrap()
            .unwrap()
            .into_active_model();
        assignment.role_id = Set(Some(role.id));
        assignment.update(&db).await.unwrap();
        let login = crate::services::auth::AuthService::login_by_pwd(
            web::Data::new(db.clone()),
            crate::dto::user::PasswordLogin {
                account: "logoutuser".into(),
                password: "password123".into(),
            },
        )
        .await
        .unwrap();
        let cookie = login
            .cookies()
            .find(|cookie| cookie.name() == "access_token")
            .unwrap();
        assert_eq!(
            cookie.max_age().unwrap().whole_seconds(),
            crate::config::manager::CONFIG.jwt.expiry
        );
        let token = cookie.value().to_string();
        let another_token = crate::utils::jwt::generate_jwt(&db, &user).await.unwrap();
        assert_ne!(
            token, another_token,
            "Every login requires a unique token ID"
        );
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(db.clone()))
                .wrap(crate::middleware::auth::Auth::new(web::Data::new(
                    db.clone(),
                )))
                .service(web::scope("/api").configure(crate::routes::auth::config_routes))
                .default_service(web::to(|| async { HttpResponse::Ok().finish() })),
        )
        .await;
        let request = || {
            test::TestRequest::get()
                .uri("/api/v1/users")
                .cookie(Cookie::new("access_token", token.clone()))
                .to_request()
        };
        assert_eq!(test::call_service(&app, request()).await.status(), 200);
        for _ in 0..2 {
            let response = test::call_service(
                &app,
                test::TestRequest::post()
                    .uri("/api/v1/auth/logout")
                    .cookie(Cookie::new("access_token", token.clone()))
                    .to_request(),
            )
            .await;
            assert_eq!(response.status(), 200);
            assert!(
                response
                    .headers()
                    .get("set-cookie")
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .contains("Max-Age=0")
            );
        }
        assert!(is_revoked(&db, &token).await.unwrap());
        assert_eq!(
            test::try_call_service(&app, request())
                .await
                .err()
                .unwrap()
                .as_response_error()
                .status_code(),
            401
        );
        let app2 = test::init_service(
            App::new()
                .wrap(crate::middleware::auth::Auth::new(web::Data::new(
                    db.clone(),
                )))
                .default_service(web::to(|| async { HttpResponse::Ok().finish() })),
        )
        .await;
        assert_eq!(
            test::try_call_service(&app2, request())
                .await
                .err()
                .unwrap()
                .as_response_error()
                .status_code(),
            401
        );
        assert_eq!(
            test::call_service(
                &app2,
                test::TestRequest::get()
                    .uri("/api/v1/users")
                    .cookie(Cookie::new("access_token", another_token))
                    .to_request()
            )
            .await
            .status(),
            200
        );
    }
}
