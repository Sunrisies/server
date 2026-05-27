//! 频道 JWT token — 用于云剪贴板频道认证，独立于用户认证系统

use crate::config::AppError;
use chrono::{Duration as ChronoDuration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

/// 频道 JWT 的 claims
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChannelClaims {
    pub channel_id: i32,
    pub channel_name: String,
    pub exp: i64,
}

/// 频道密钥（不与 JWT_SECRET 共用，从环境变量 CHANNEL_SECRET 读取，默认使用 JWT_SECRET）
fn channel_secret() -> String {
    std::env::var("CHANNEL_SECRET")
        .or_else(|_| std::env::var("JWT_SECRET"))
        .unwrap_or_else(|_| {
            log::warn!("CHANNEL_SECRET 未设置，使用固定默认值，请在生产环境配置");
            "change_this_channel_secret_in_production".to_string()
        })
}

/// 生成频道 token（不过期，永久有效）
pub fn generate_channel_token(channel_id: i32, channel_name: &str) -> Result<String, AppError> {
    let claims = ChannelClaims {
        channel_id,
        channel_name: channel_name.to_string(),
        exp: (Utc::now() + ChronoDuration::days(36500)).timestamp(), // ～100年
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(channel_secret().as_bytes()),
    )
    .map_err(|e| {
        log::error!("频道 token 生成失败: {}", e);
        AppError::InternalServerError("生成频道凭证失败".to_string())
    })
}

/// 解析频道 token
pub fn decode_channel_token(token: &str) -> Result<ChannelClaims, AppError> {
    let validation = Validation::new(Algorithm::HS256);
    decode::<ChannelClaims>(
        token,
        &DecodingKey::from_secret(channel_secret().as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|e| {
        log::error!("频道 token 解析失败: {}", e);
        AppError::Unauthorized("频道凭证无效或已过期".to_string())
    })
}
