use anyhow::{Context, Result};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::config::manager::CONFIG;

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

/// 邮件服务
pub struct EmailService {
    settings: SmtpSettings,
}

impl Default for EmailService {
    fn default() -> Self {
        Self {
            settings: CONFIG.smtp.clone(),
        }
    }
}
impl EmailService {
    /// 生成6位随机验证码
    pub fn generate_verification_code() -> String {
        let mut rng = rand::thread_rng();
        (0..6).map(|_| rng.gen_range(0..10).to_string()).collect()
    }

    /// 发送验证码邮件
    pub async fn send_verification_code(&self, to_email: &str, code: &str) -> Result<()> {
        let subject = "博客系统验证码";
        let body = format!(
            r#"
            <html>
                <body>
                    <h2>博客系统验证码</h2>
                    <p>您好，</p>
                    <p>您正在尝试登录博客系统，您的验证码是：</p>
                    <h1 style="color: blue; font-size: 32px;">{}</h1>
                    <p>验证码有效期为{}分钟，请及时使用。</p>
                    <p>如果这不是您的操作，请忽略此邮件。</p>
                    <p>谢谢！</p>
                    <p>博客系统团队</p>
                </body>
            </html>
            "#,
            code,
            self.settings.code_validity_period / 60
        );
        // Ok(())
        self.send_email(to_email, subject, &body).await
    }

    /// 发送密码重置邮件
    pub async fn send_password_reset(&self, to_email: &str, reset_link: &str) -> Result<()> {
        let subject = "博客系统密码重置";
        let body = format!(
            r#"
            <html>
                <body>
                    <h2>博客系统密码重置</h2>
                    <p>您好，</p>
                    <p>您请求重置博客系统密码，请点击以下链接重置密码：</p>
                    <a href="{}" style="color: blue; font-size: 18px;">重置密码</a>
                    <p>如果您无法点击链接，请复制以下URL到浏览器地址栏：</p>
                    <p>{}</p>
                    <p>此链接有效期为{}分钟，请及时使用。</p>
                    <p>如果这不是您的操作，请忽略此邮件。</p>
                    <p>谢谢！</p>
                    <p>博客系统团队</p>
                </body>
            </html>
            "#,
            reset_link,
            reset_link,
            self.settings.code_validity_period / 60
        );

        self.send_email(to_email, subject, &body).await
    }

    /// 发送欢迎邮件
    pub async fn send_welcome(&self, to_email: &str, username: &str) -> Result<()> {
        let subject = "欢迎加入博客系统";
        let body = format!(
            r#"
            <html>
                <body>
                    <h2>欢迎加入博客系统，{}</h2>
                    <p>您好，</p>
                    <p>感谢您注册我们的博客系统！</p>
                    <p>您现在可以开始浏览、评论和发布文章了。</p>
                    <p>如有任何问题，请随时联系我们。</p>
                    <p>谢谢！</p>
                    <p>博客系统团队</p>
                </body>
            </html>
            "#,
            username
        );

        self.send_email(to_email, subject, &body).await
    }

    /// 发送新评论通知邮件
    pub async fn send_comment_notification(
        &self,
        to_email: &str,
        author_name: &str,
        post_title: &str,
        comment_content: &str,
        comment_link: &str,
    ) -> Result<()> {
        let subject = "博客系统 - 新评论通知";
        let body = format!(
            r#"
            <html>
                <body>
                    <h2>新评论通知</h2>
                    <p>您好，</p>
                    <p>用户 <strong>{}</strong> 在您的文章 <strong>{}</strong> 下发表了新评论：</p>
                    <div style="background-color: #f5f5f5; padding: 15px; border-radius: 5px; margin: 10px 0;">
                        <p>{}</p>
                    </div>
                    <p>点击以下链接查看完整评论：</p>
                    <a href="{}" style="color: blue; font-size: 18px;">查看评论</a>
                    <p>谢谢！</p>
                    <p>博客系统团队</p>
                </body>
            </html>
            "#,
            author_name, post_title, comment_content, comment_link
        );

        self.send_email(to_email, subject, &body).await
    }

    /// 发送邮件的通用方法
    async fn send_email(&self, to_email: &str, subject: &str, body: &str) -> Result<()> {
        // 创建邮件
        let email = Message::builder()
            .from(
                self.settings
                    .from_email
                    .parse()
                    .context("Invalid from email address")?,
            )
            .to(to_email.parse().context("Invalid to email address")?)
            .subject(subject)
            .multipart(
                lettre::message::MultiPart::alternative()
                    .singlepart(
                        lettre::message::SinglePart::builder()
                            .header(lettre::message::header::ContentType::TEXT_PLAIN)
                            .body(self.html_to_text(body)),
                    )
                    .singlepart(
                        lettre::message::SinglePart::builder()
                            .header(lettre::message::header::ContentType::TEXT_HTML)
                            .body(body.to_string()),
                    ),
            )
            .context("Failed to build email")?;

        // 创建SMTP传输
        let creds = Credentials::new(
            self.settings.from_email.clone(),
            self.settings.from_password.clone(),
        );
        let mailer = SmtpTransport::relay(&self.settings.smtp_server)
            .context("Failed to create SMTP transport")?
            .port(self.settings.smtp_port)
            .credentials(creds)
            .build();

        // 发送邮件
        match mailer.send(&email) {
            Ok(_) => {
                log::info!("Email successfully sent to {}", to_email);
                Ok(())
            }
            Err(e) => {
                log::error!("Could not send email to {}: {:?}", to_email, e);
                Err(anyhow::anyhow!("Failed to send email: {}", e))
            }
        }
    }

    /// 简单的HTML到纯文本转换
    fn html_to_text(&self, html: &str) -> String {
        // 这是一个简单的实现，实际项目中可能需要使用更复杂的库
        let mut text = String::new();
        let mut in_tag = false;
        let mut current_tag = String::new();

        for c in html.chars() {
            if c == '<' {
                in_tag = true;
                current_tag.clear();
            } else if c == '>' {
                in_tag = false;
                // 处理一些特定标签
                if current_tag == "br"
                    || current_tag == "/p"
                    || current_tag == "/h1"
                    || current_tag == "/h2"
                {
                    text.push('\n');
                } else if current_tag == "li" {
                    text.push_str("\n- ");
                }
            } else if in_tag {
                current_tag.push(c);
            } else {
                text.push(c);
            }
        }

        text
    }
}

/// 邮件验证码缓存
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailVerificationCode {
    /// 邮箱地址
    pub email: String,
    /// 验证码
    pub code: String,
    /// 创建时间（Unix时间戳）
    pub created_at: u64,
    /// 是否已使用
    pub used: bool,
}

impl EmailVerificationCode {
    /// 创建新的验证码记录
    pub fn new(email: String, code: String) -> Self {
        Self {
            email,
            code,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            used: false,
        }
    }

    /// 检查验证码是否有效
    pub fn is_valid(&self, input_code: &str, validity_period: u64) -> bool {
        if self.used {
            return false;
        }

        if self.code != input_code {
            return false;
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // 检查是否在有效期内
        now - self.created_at <= validity_period
    }

    /// 标记验证码为已使用
    pub fn mark_used(&mut self) {
        self.used = true;
    }
}

/// 邮件验证码管理器
pub struct EmailVerificationManager {
    /// 验证码缓存（生产环境应使用Redis等）
    codes: std::sync::Arc<
        tokio::sync::RwLock<std::collections::HashMap<String, EmailVerificationCode>>,
    >,
    /// 验证码有效期（秒）
    validity_period: u64,
}
impl Default for EmailVerificationManager {
    fn default() -> Self {
        Self::new()
    }
}
impl EmailVerificationManager {
    /// 创建新的验证码管理器
    pub fn new() -> Self {
        Self {
            codes: std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            validity_period: CONFIG.smtp.code_validity_period,
        }
    }

    /// 生成并发送验证码
    pub async fn generate_and_send_code(
        &self,
        email_service: &EmailService,
        email: &str,
    ) -> Result<String> {
        // 生成验证码
        let code = EmailService::generate_verification_code();

        // 存储验证码
        let verification_code = EmailVerificationCode::new(email.to_string(), code.clone());
        {
            let mut codes = self.codes.write().await;
            codes.insert(email.to_string(), verification_code);
        }

        // 发送邮件
        email_service.send_verification_code(email, &code).await?;

        Ok(code)
    }

    /// 验证验证码
    pub async fn verify_code(&self, email: &str, code: &str) -> Result<bool> {
        let mut codes = self.codes.write().await;

        if let Some(verification_code) = codes.get_mut(email)
            && verification_code.is_valid(code, self.validity_period)
        {
            verification_code.mark_used();
            return Ok(true);
        }

        Ok(false)
    }

    /// 启动定期清理过期验证码的任务
    pub fn start_cleanup_task(&self) {
        let codes = self.codes.clone();
        let validity_period = self.validity_period;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60 * 5)); // 每5分钟清理一次

            loop {
                interval.tick().await;

                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                let mut codes = codes.write().await;
                codes.retain(|_, verification_code| {
                    // 保留未使用且未过期的验证码
                    !verification_code.used
                        && (now - verification_code.created_at) <= validity_period
                });

                log::debug!("Cleaned up expired email verification codes");
            }
        });
    }
}
