use dotenvy::dotenv;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QiNiuSettings {
    pub access_key: String,
    pub secret_key: String,
    pub bucket: String,
    pub domain: String,
}
/// 邮件配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpSettings {
    /// SMTP服务器地址
    pub smtp_server: String,
    /// SMTP服务器端口
    pub smtp_port: u16,
    /// 发件人邮箱
    pub from_email: String,
    /// 发件人邮箱密码或应用专用密码
    pub from_password: String,
    /// 验证码有效期（秒）
    pub code_validity_period: u64,
}
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtSettings {
    pub secret: String,
    pub expiry: i64,
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
                max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap(),
                min_connections: env::var("DATABASE_MIN_CONNECTIONS")
                    .unwrap_or_else(|_| "1".to_string())
                    .parse()
                    .unwrap(),
                connect_timeout: env::var("DATABASE_CONNECT_TIMEOUT")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap(),
                acquire_timeout: env::var("DATABASE_ACQUIRE_TIMEOUT")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap(),
                idle_timeout: env::var("DATABASE_IDLE_TIMEOUT")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap(),
                max_lifetime: env::var("DATABASE_MAX_LIFETIME")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap(),
                sqlx_logging: env::var("DATABASE_SQLX_LOGGING")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .unwrap(),
                test_before_acquire: env::var("DATABASE_TEST_BEFORE_ACQUIRE")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap(),
                set_schema_search_path: env::var("DATABASE_SET_SCHEMA_SEARCH_PATH")
                    .unwrap_or_else(|_| "public".to_string()),
            },
            jwt: JwtSettings {
                secret: env::var("JWT_SECRET").unwrap_or_else(|_| "your_secret_key".to_string()),
                expiry: env::var("JWT_EXPIRATION")
                    .unwrap_or_else(|_| "86400".to_string())
                    .parse()
                    .unwrap(),
            },
            qi_niu: QiNiuSettings {
                access_key: env::var("QINIU_ACCESS_KEY")
                    .unwrap_or_else(|_| "your_access_key".to_string()),
                secret_key: env::var("QINIU_SECRET_KEY")
                    .unwrap_or_else(|_| "your_secret_key".to_string()),
                bucket: env::var("QINIU_BUCKET").unwrap_or_else(|_| "your_bucket".to_string()),
                domain: env::var("QINIU_DOMAIN").unwrap_or_else(|_| "your_domain".to_string()),
            },
            smtp: SmtpSettings {
                smtp_server: env::var("SMTP_HOST")
                    .unwrap_or_else(|_| "smtp.example.com".to_string()),
                smtp_port: env::var("SMTP_PORT")
                    .unwrap_or_else(|_| "587".to_string())
                    .parse()
                    .unwrap(),
                from_email: env::var("SMTP_USERNAME")
                    .unwrap_or_else(|_| "your_username".to_string()),

                from_password: env::var("SMTP_PASSWORD")
                    .unwrap_or_else(|_| "your_password".to_string()),
                code_validity_period: env::var("CODE_VALIDITY_PERIOD")
                    .unwrap_or_else(|_| "300".to_string())
                    .parse()
                    .unwrap(),
            },
            security: SecuritySettings {
                salt: env::var("SECURITY_SALT").unwrap_or_else(|_| "your_salt".to_string()),
                token_expiration: env::var("SECURITY_TOKEN_EXPIRATION")
                    .unwrap_or_else(|_| "86400".to_string())
                    .parse()
                    .unwrap(),
            },
            upload: UploadSettings {
                max_size: env::var("UPLOAD_MAX_SIZE")
                    .unwrap_or_else(|_| "10485760".to_string())
                    .parse()
                    .unwrap(),
                allowed_types: None,
            },
            server: ServerSettings {
                host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("SERVER_PORT")
                    .unwrap_or_else(|_| "3000".to_string())
                    .parse()
                    .unwrap(),
                // debug: std::env::var("CRUD_MACRO_DEBUG").is_ok(),
            },
        }
        //     enabled: std::env::var("CRUD_MACRO_DEBUG").is_ok(),
        //     colorize: true,
        // }
    }
}
pub static CONFIG: Lazy<AppConfig> = Lazy::new(AppConfig::default);

impl AppConfig {
    // 获取全局配置的静态方法
    pub fn global() -> &'static AppConfig {
        &CONFIG
    }
}
