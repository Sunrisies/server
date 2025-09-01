use std::fmt;

use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationErrors};

#[derive(Validate, Debug, Default, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    #[validate(length(min = 5, max = 100, message = "用户名长度必须在5到100之间"))]
    pub user_name: String,
    #[validate(length(min = 6, max = 100, message = "密码长度必须在6到100之间"))]
    pub pass_word: String,
}

#[derive(Validate, Debug, Default, Clone, Serialize, Deserialize)]
pub struct RegisterResponse {
    #[validate(length(min = 5, max = 100, message = "用户名长度必须在5到100之间"))]
    pub user_name: String,
    #[validate(length(min = 6, max = 100, message = "密码长度必须在6到100之间"))]
    pub pass_word: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ValidationErrorItem {
    pub name: String,
    pub error: String,
}

#[derive(Serialize, Debug)]
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
