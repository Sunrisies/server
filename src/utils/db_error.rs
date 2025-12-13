use actix_web::error::JsonPayloadError::{self};
use sea_orm::DbErr;
use validator::ValidationErrors;

use crate::{ApiResponse, config::AppError, dto::user::ValidationErrorJson};
/// 把 Sea-ORM 底层数据库错误转成用户能看懂的 &str
pub fn db_err_map(e: DbErr) -> &'static str {
    // Sea-ORM 的 DbErr::Query 里会带数据库原始错误信息
    let detail = e.to_string();

    if detail.contains("duplicate key value violates unique constraint") {
        // 可以细化到字段
        if detail.contains("categories_slug_key") {
            "英文名（slug）已存在，请更换"
        } else if detail.contains("categories_name_key") {
            "分类名称已存在"
        } else if detail.contains("users_email_key") {
            "邮箱已被注册"
        } else if detail.contains("users_username_key") {
            "用户名已被使用"
        } else if detail.contains("posts_slug_key") {
            "文章英文名（slug）已存在"
        } else {
            "数据重复，请检查唯一字段"
        }
    } else if detail.contains("foreign key constraint") {
        "关联数据不存在，无法操作"
    } else if detail.contains("violates not-null constraint") {
        "必填字段不能为空"
    } else if detail.contains("value too long") {
        "字段长度超出限制"
    } else if detail.contains("invalid input syntax for type uuid") {
        "无效的ID格式"
    } else {
        // 其他数据库错误统一模糊提示
        "数据库操作失败，请稍后再试"
    }
}

// json
pub fn json_err_map(err: JsonPayloadError) -> actix_web::Error {
    let mut validation_errors = ValidationErrors::new();

    match err {
        JsonPayloadError::Deserialize(ref error) => {
            log::error!("JSON deserialization error: {:?}", error);

            // 提取缺失字段的逻辑封装为函数
            if let Some(field) = extract_missing_field(&error) {
                validation_errors.add(
                    "name", // 固定字段名
                    validator::ValidationError::new("required")
                        .with_message(format!("缺少必填字段: {}", field).into()),
                );
            } else {
                validation_errors.add(
                    "json",
                    validator::ValidationError::new("invalid")
                        .with_message(format!("数据反序列化错误: {}", error).into()),
                );
            }
        }
        _ => {
            log::error!("JSON processing error: {:?}", err);
            validation_errors.add(
                "json",
                validator::ValidationError::new("invalid")
                    .with_message(std::borrow::Cow::Borrowed("JSON 解析错误")),
            );
        }
    }

    let error_response = ApiResponse::from(AppError::ValidationError(
        ValidationErrorJson::from_validation_errors(&validation_errors),
    ))
    .to_http_response();

    actix_web::error::InternalError::from_response(format!("{}", err), error_response).into()
}

// 辅助函数，返回拥有所有权的String
fn extract_missing_field(error: &impl std::fmt::Display) -> Option<String> {
    let error_str = error.to_string();
    if error_str.contains("missing field") {
        error_str
            .split('`')
            .nth(1)
            .map(|s| s.trim_matches('"').to_string())
    } else {
        None
    }
}
