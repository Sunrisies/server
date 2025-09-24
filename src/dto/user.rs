use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;
use utoipa::ToSchema;
use validator::{Validate, ValidationErrors};
// ------------------ 独立变体结构体 ------------------
#[derive(Debug, Validate, Deserialize, Serialize, ToSchema)]
pub struct PasswordLogin {
    #[validate(length(min = 1, max = 100))]
    pub account: String,

    #[validate(length(min = 6, max = 100))]
    pub password: String,
}

#[derive(Debug, Validate, Deserialize, Serialize, ToSchema)]
pub struct EmailLogin {
    #[validate(email)]
    pub email: String,

    #[validate(length(min = 6, max = 6, message = "验证码格式错误"))]
    #[schema(default = "123456")]
    pub code: String,
}

#[derive(Debug, Validate, Deserialize, Serialize, ToSchema)]
pub struct PhoneLogin {
    #[validate(regex(path = "*RE_PHONE", message = "手机号格式错误"))]
    pub phone: String,

    #[validate(length(min = 6, max = 6, message = "验证码格式错误"))]
    #[schema(default = "123456")]
    pub code: String,
}

#[derive(Debug, Validate, Deserialize, Serialize, ToSchema)]
pub struct OAuthLogin {
    #[validate(length(min = 1))]
    pub provider: String,

    #[validate(length(min = 1))]
    pub openid: String,

    #[validate(length(min = 1))]
    pub access_token: String,
}

// ------------------ 枚举只做分发 ------------------
#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(tag = "login_type")]
pub enum LoginRequest {
    #[serde(rename = "password")]
    Password(PasswordLogin),

    #[serde(rename = "email")]
    Email(EmailLogin),

    #[serde(rename = "phone")]
    Phone(PhoneLogin),

    #[serde(rename = "oauth")]
    OAuth(OAuthLogin),
}

/* 统一校验入口 */
impl LoginRequest {
    pub fn validate(&self) -> Result<(), validator::ValidationErrors> {
        match self {
            LoginRequest::Password(v) => v.validate(),
            LoginRequest::Email(v) => v.validate(),
            LoginRequest::Phone(v) => v.validate(),
            LoginRequest::OAuth(v) => v.validate(),
        }
    }
}
// 手机号正则
lazy_static! {
    static ref RE_PHONE: Regex = Regex::new(r"^1[3-9]\d{9}$").unwrap();
}
// #[derive(Validate, Debug, Default, Clone, Serialize, Deserialize)]
// pub struct LoginRequest {
//     #[validate(length(min = 5, max = 100, message = "用户名长度必须在5到100之间"))]
//     pub user_name: String,
//     #[validate(length(min = 6, max = 100, message = "密码长度必须在6到100之间"))]
//     pub pass_word: String,
//    /// 登录类型，一般登录,邮箱登录,手机号登录,第三方登录
//    #[validate()]
//     pub login_type: String,
// }

#[derive(Validate, Debug, Default, Clone, Serialize, Deserialize, ToSchema)]
pub struct RegisterResponse {
    #[validate(length(min = 5, max = 100, message = "用户名长度必须在5到100之间"))]
    pub user_name: String,
    #[validate(length(min = 6, max = 100, message = "密码长度必须在6到100之间"))]
    pub pass_word: String,
}
#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ValidationErrorItem {
    pub name: String,
    pub error: String,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct ValidationErrorJson {
    // 字段名 -> 错误消息数组
    pub errors: Vec<ValidationErrorItem>,
}
pub struct ValidationErrorMsg<'a>(pub &'a ValidationErrors);
impl ValidationErrorJson {
    pub fn from_validation_errors(errs: &ValidationErrors) -> Self {
        let mut list = Vec::new();
        for (field, field_errs) in errs.field_errors() {
            for err in field_errs {
                let msg = err
                    .message
                    .as_ref()
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "invalid value".into());
                list.push(ValidationErrorItem {
                    name: field.to_string(),
                    error: msg,
                });
            }
        }
        ValidationErrorJson { errors: list }
    }
}
// impl fmt::Display for ValidationErrorJson {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         let parts = self
//             .errors
//             .iter()
//             .map(|item| format!("{}: {}", item.name, item.error));
//         // 多个错误信息用分号分隔
//         write!(f, "{}", parts.collect::<Vec<_>>().join("; "))
//         // write!(f, "{}", parts.join("; "))
//     }
// }

impl fmt::Display for ValidationErrorMsg<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 逐个字段、逐个错误打印
        for (field, errs) in self.0.field_errors() {
            for err in errs {
                writeln!(
                    f,
                    "{}: {}",
                    field,
                    err.message
                        .as_ref()
                        .map(|cow| cow.as_ref())
                        .unwrap_or("invalid value")
                )?;
            }
        }
        Ok(())
    }
}

#[derive(Deserialize, Debug, Default, Clone, Serialize, Validate)]
pub struct UpdateUserRequest {
    #[validate(length(min = 5, max = 100, message = "用户名长度必须在5到100之间"))]
    pub user_name: String,
    #[serde(rename = "image")]
    pub image: Option<String>,
    pub permissions: Option<Vec<String>>,
}
