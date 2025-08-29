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
pub struct ValidationErrorMsg<'a>(pub &'a ValidationErrors);

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
