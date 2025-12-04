use crate::config::manager::CONFIG;
use crate::models::roles::{self};
use crate::models::user_roles;
use crate::{config::AppError, models::users::Model};
use base64::engine::{Engine as _, general_purpose};
use chrono::{Duration as ChronoDuration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub user_uuid: String,
    pub user_name: String,
    pub exp: i64,     // u64 更安全
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
        let roles = user_roles::Entity::find()
            .filter(user_roles::Column::UserId.eq(user.id))
            .find_with_related(roles::Entity)
            .all(db_pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to get user roles: {}", e)))?;

        // 生成角色哈希值
        let role_ids: Vec<i32> = roles
            .iter()
            .flat_map(|(_, related_roles)| related_roles.iter().map(|role| role.id))
            .collect();
        Ok(role_ids[0])
    }
}

/// 生成 JWT
pub async fn generate_jwt(
    db_pool: &sea_orm::DatabaseConnection,
    user: &Model,
) -> Result<String, AppError> {
    let claims = TokenClaims::from_user(db_pool, user).await?;
    log::info!("claims: {:?},{}", claims, &CONFIG.jwt.secret);
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(
            &general_purpose::STANDARD
                .decode(&CONFIG.jwt.secret)
                .map_err(|_| AppError::InternalServerError("JWT 密钥格式错误".into()))?,
        ),
    )
    .map_err(|e| {
        log::error!("JWT 生成失败: {}", e);
        AppError::InternalServerError("登录服务暂时不可用".into())
    })
}

/// 解析 JWT
pub fn decode_jwt(token: &str) -> Result<TokenClaims, AppError> {
    let validation = Validation::new(Algorithm::HS256);
    decode::<TokenClaims>(
        token,
        &DecodingKey::from_secret(
            &general_purpose::STANDARD
                .decode(&CONFIG.jwt.secret)
                .map_err(|_| AppError::InternalServerError("JWT 密钥格式错误".into()))?,
        ),
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
