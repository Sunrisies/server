use crate::config::manager::CONFIG;
use crate::models::roles::{self};
use crate::models::user_roles;
use crate::{config::AppError, models::users::Model};
use chrono::{Duration as ChronoDuration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub user_uuid: String,
    pub user_name: String,
    pub exp: i64, // u64 更安全
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jti: Option<String>,
    pub role_id: i32, //
}

impl TokenClaims {
    async fn from_user(
        db_pool: &sea_orm::DatabaseConnection,
        user: &Model,
    ) -> Result<Self, AppError> {
        let exp = (Utc::now() + ChronoDuration::seconds(CONFIG.jwt.expiry)).timestamp();
        let role_id = Self::get_permission_codes(db_pool, user).await?;

        Ok(TokenClaims {
            jti: Some(uuid::Uuid::new_v4().to_string()),
            user_uuid: user.uuid.clone(),
            user_name: user.user_name.clone(),
            exp,
            role_id,
        })
    }
    /// 获取权限码集合
    async fn get_permission_codes(
        db_pool: &sea_orm::DatabaseConnection,
        user: &Model,
    ) -> Result<i32, AppError> {
        let assignments = user_roles::Entity::find()
            .filter(user_roles::Column::UserId.eq(user.id))
            .all(db_pool)
            .await
            .map_err(|_| AppError::DatabaseError("用户角色查询失败".into()))?;
        let ids: Vec<i32> = assignments
            .iter()
            .filter_map(|assignment| assignment.role_id)
            .collect();
        let roles = roles::Entity::find()
            .filter(roles::Column::Id.is_in(ids))
            .all(db_pool)
            .await
            .map_err(|_| AppError::DatabaseError("用户角色查询失败".into()))?;
        roles
            .first()
            .map(|role| role.id)
            .ok_or_else(|| AppError::Forbidden("用户未分配角色".into()))
    }
}

/// 生成 JWT
pub async fn generate_jwt(
    db_pool: &sea_orm::DatabaseConnection,
    user: &Model,
) -> Result<String, AppError> {
    let claims = TokenClaims::from_user(db_pool, user).await?;
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(CONFIG.jwt.secret.as_bytes()),
    )
    .map_err(|e| {
        log::error!("JWT 生成失败: {}", e);
        AppError::InternalServerError("登录服务暂时不可用".into())
    })
}

/// 解析 JWT
pub fn decode_jwt(token: &str) -> Result<TokenClaims, AppError> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.leeway = 0;
    decode::<TokenClaims>(
        token,
        &DecodingKey::from_secret(CONFIG.jwt.secret.as_bytes()),
        &validation,
    )
    .map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
            AppError::Unauthorized("令牌已过期，请重新登录".into())
        }
        jsonwebtoken::errors::ErrorKind::InvalidSignature => {
            AppError::Unauthorized("令牌签名无效".into())
        }
        _ => {
            log::error!("JWT 解析失败: {}", e);
            AppError::Unauthorized("令牌无效".into())
        }
    })
    .map(|data| data.claims)
}

#[cfg(test)]
mod auth_role_tests {
    use super::*;
    #[actix_web::test]
    async fn auth_user_without_roles_returns_error_without_panicking() {
        let db = crate::services::auth_test::tests::setup_test_db().await;
        let user = crate::services::auth::AuthService::register(
            actix_web::web::Data::new(db.clone()),
            actix_web::web::Json(crate::dto::user::RegisterResponse {
                user_name: "emptyroles".into(),
                pass_word: "password123".into(),
            }),
            actix_web::web::Data::new(crate::SseNotifier::new()),
        )
        .await
        .unwrap();
        user_roles::Entity::delete_many()
            .filter(user_roles::Column::UserId.eq(user.id))
            .exec(&db)
            .await
            .unwrap();
        assert!(TokenClaims::get_permission_codes(&db, &user).await.is_err());
    }
}
