use lazy_static::lazy_static;
use std::env;

/// 邮件配置
#[derive(Debug, Clone)]
pub struct EmailConfig {
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

impl EmailConfig {
    /// 从环境变量加载配置
    pub fn from_env() -> Self {
        Self {
            smtp_server: env::var("SMTP_SERVER").unwrap_or_else(|_| "smtp.gmail.com".to_string()),
            smtp_port: env::var("SMTP_PORT")
                .unwrap_or_else(|_| "587".to_string())
                .parse()
                .unwrap_or(587),
            from_email: env::var("FROM_EMAIL").expect("FROM_EMAIL must be set"),
            from_password: env::var("FROM_EMAIL_PASSWORD")
                .expect("FROM_EMAIL_PASSWORD must be set"),
            code_validity_period: env::var("EMAIL_CODE_VALIDITY_PERIOD")
                .unwrap_or_else(|_| "300".to_string()) // 默认5分钟
                .parse()
                .unwrap_or(300),
        }
    }
}

lazy_static! {
    /// 全局邮件配置
    pub static ref EMAIL_CONFIG: EmailConfig = EmailConfig::from_env();
}
