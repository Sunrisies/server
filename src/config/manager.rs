use dotenvy::dotenv;
use serde::{Deserialize, Serialize};

use std::{env, sync::LazyLock};

use crate::{email::SmtpSettings, services::qiniu::QiNiuSettings};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    pub salt: String,
    pub token_expiration: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadSettings {
    pub max_size: u64,
    pub allowed_types: Option<Vec<String>>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseSettings,
    pub jwt: JwtSettings,
    pub qi_niu: QiNiuSettings,
    pub smtp: SmtpSettings,
    pub security: SecuritySettings,
    pub upload: UploadSettings,
    pub server: ServerSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseSettings {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: u64,
    pub acquire_timeout: u64,
    pub idle_timeout: u64,
    pub max_lifetime: u64,
    pub sqlx_logging: bool,
    pub test_before_acquire: bool,
    pub set_schema_search_path: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct JwtSettings {
    pub secret: String,
    pub expiry: i64,
}

impl std::fmt::Debug for JwtSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JwtSettings")
            .field("secret", &"[REDACTED]")
            .field("expiry", &self.expiry)
            .finish()
    }
}

// 其他设置结构...
// 添加默认值和从环境变量加载配置的方法
impl Default for AppConfig {
    fn default() -> Self {
        dotenv().ok();

        log::info!("Loading configuration from environment variables");
        Self {
            database: DatabaseSettings {
                url: env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://blog.db".to_string()),
                max_connections: env_or_default("DATABASE_MAX_CONNECTIONS", 10),
                min_connections: env_or_default("DATABASE_MIN_CONNECTIONS", 1),
                connect_timeout: env_or_default("DATABASE_CONNECT_TIMEOUT", 10),
                acquire_timeout: env_or_default("DATABASE_ACQUIRE_TIMEOUT", 10),
                idle_timeout: env_or_default("DATABASE_IDLE_TIMEOUT", 10),
                max_lifetime: env_or_default("DATABASE_MAX_LIFETIME", 10),
                sqlx_logging: env_or_default("DATABASE_SQLX_LOGGING", false),
                test_before_acquire: env_or_default("DATABASE_TEST_BEFORE_ACQUIRE", true),
                set_schema_search_path: env::var("DATABASE_SET_SCHEMA_SEARCH_PATH")
                    .unwrap_or_else(|_| "public".to_string()),
            },
            jwt: JwtSettings {
                secret: validated_jwt_secret(env::var("JWT_SECRET").ok())
                    .expect("JWT_SECRET must be set to a strong secret of at least 32 bytes"),
                expiry: env_or_default("JWT_EXPIRATION", 86400),
            },
            qi_niu: QiNiuSettings {
                access_key: env::var("QINIU_ACCESS_KEY")
                    .unwrap_or_else(|_| "your_access_key".to_string()),
                secret_key: env::var("QINIU_SECRET_KEY")
                    .unwrap_or_else(|_| "your_secret_key".to_string()),
                bucket_name: env::var("QINIU_BUCKET").unwrap_or_else(|_| "your_bucket".to_string()),
                domain_url: env::var("QINIU_DOMAIN").unwrap_or_else(|_| "your_domain".to_string()),
                token_expiry_secs: env_or_default("QINIU_TOKEN_EXPIRY_SECS", 3600),
            },
            smtp: SmtpSettings {
                smtp_server: env::var("SMTP_HOST")
                    .unwrap_or_else(|_| "smtp.example.com".to_string()),
                smtp_port: env_or_default("SMTP_PORT", 587),
                from_email: env::var("SMTP_USERNAME")
                    .unwrap_or_else(|_| "your_username".to_string()),

                from_password: env::var("SMTP_PASSWORD")
                    .unwrap_or_else(|_| "your_password".to_string()),
                code_validity_period: env_or_default("CODE_VALIDITY_PERIOD", 300),
            },
            security: SecuritySettings {
                salt: env::var("SECURITY_SALT").unwrap_or_else(|_| "your_salt".to_string()),
                token_expiration: env_or_default("SECURITY_TOKEN_EXPIRATION", 86400),
            },
            upload: UploadSettings {
                max_size: env_or_default("UPLOAD_MAX_SIZE", 10_485_760),
                allowed_types: None,
            },
            server: ServerSettings {
                host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env_or_default("SERVER_PORT", 3000),
                // debug: std::env::var("CRUD_MACRO_DEBUG").is_ok(),
            },
        }
        //     enabled: std::env::var("CRUD_MACRO_DEBUG").is_ok(),
        //     colorize: true,
        // }
    }
}
pub static CONFIG: LazyLock<AppConfig> = LazyLock::new(AppConfig::default);

impl AppConfig {
    // 获取全局配置的静态方法
    pub fn global() -> &'static AppConfig {
        &CONFIG
    }
}

fn env_or_default<T>(name: &str, default: T) -> T
where
    T: std::str::FromStr,
{
    match env::var(name) {
        Ok(value) => match value.trim().parse() {
            Ok(parsed) => parsed,
            Err(_) => {
                log::warn!("Invalid {name}; using the documented default");
                default
            }
        },
        Err(_) => default,
    }
}

fn validated_jwt_secret(secret: Option<String>) -> Result<String, &'static str> {
    let secret = secret.ok_or("missing JWT_SECRET")?;
    if secret.trim().len() < 32
        || secret.contains("your_secret")
        || secret.contains("your-secret")
        || secret.contains("change_me")
        || secret.contains("change-in-production")
    {
        return Err("invalid JWT_SECRET");
    }
    Ok(secret)
}

#[cfg(test)]
mod jwt_secret_tests {
    use super::{JwtSettings, validated_jwt_secret};
    #[test]
    fn debug_output_redacts_secret() {
        let config = JwtSettings {
            secret: "never-print-this-secret".into(),
            expiry: 3600,
        };
        let output = format!("{config:?}");
        assert!(!output.contains("never-print-this-secret"));
        assert!(output.contains("[REDACTED]"));
    }
    #[test]
    fn rejects_missing_weak_and_placeholder_keys() {
        for value in [
            None,
            Some("your_secret_key".into()),
            Some(" ".repeat(64)),
            Some("change_me".repeat(8)),
        ] {
            assert!(validated_jwt_secret(value).is_err());
        }
        assert!(validated_jwt_secret(Some("0123456789abcdef0123456789abcdef".into())).is_ok());
    }
}
