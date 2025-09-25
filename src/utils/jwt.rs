use base64::engine::{Engine as _, general_purpose};
use chrono::{Duration as ChronoDuration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::{config::AppError, models::users::Model};

type Seconds = u64;

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub user_uuid: String,
    pub user_name: String,
    pub exp: Seconds, // u64 更安全
    pub permissions: Option<String>,
}

impl TokenClaims {
    fn from_user(user: &Model, ttl: Seconds) -> Self {
        let exp = (Utc::now() + ChronoDuration::seconds(ttl as i64)).timestamp() as Seconds;
        TokenClaims {
            user_uuid: user.uuid.clone(),
            user_name: user.user_name.clone(),
            exp,
            permissions: user.permissions.clone(),
        }
    }
}

/// 生成 JWT
pub fn generate_jwt(user: &Model, secret_b64: &str, ttl: Seconds) -> Result<String, AppError> {
    let claims = TokenClaims::from_user(user, ttl);
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(
            &general_purpose::STANDARD
                .decode(secret_b64)
                .map_err(|_| AppError::InternalServerError("JWT 密钥格式错误".into()))?,
        ),
    )
    .map_err(|e| {
        log::error!("JWT 生成失败: {}", e);
        AppError::InternalServerError("登录服务暂时不可用".into())
    })
}

/// 解析 JWT
pub fn decode_jwt(token: &str, secret_b64: &str) -> Result<TokenClaims, AppError> {
    let validation = Validation::new(Algorithm::HS256);
    decode::<TokenClaims>(
        token,
        &DecodingKey::from_secret(
            &general_purpose::STANDARD
                .decode(secret_b64)
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
