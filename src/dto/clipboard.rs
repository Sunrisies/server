use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

/// 上传文本请求
#[derive(Debug, Validate, Deserialize, Serialize, ToSchema)]
pub struct CreateTextRequest {
    #[validate(length(min = 1, max = 100000, message = "内容长度需要在1-100000之间"))]
    #[schema(example = "这是一段需要跨设备同步的文本内容")]
    pub content: String,
}

/// 剪贴板条目响应
#[derive(Debug, Serialize, ToSchema)]
pub struct ClipboardEntryResponse {
    pub uuid: String,
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_size: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    pub pinned: bool,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<crate::models::clipboard_entries::Model> for ClipboardEntryResponse {
    fn from(m: crate::models::clipboard_entries::Model) -> Self {
        Self {
            uuid: m.uuid,
            r#type: m.r#type,
            content: m.content,
            file_url: m.file_url,
            file_name: m.file_name,
            file_size: m.file_size,
            mime_type: m.mime_type,
            pinned: m.pinned,
            created_at: m.created_at,
        }
    }
}

/// 剪贴板列表筛选
#[derive(Debug, Deserialize, Serialize, IntoParams)]
#[into_params(style = Form, parameter_in = Query)]
pub struct ClipboardQuery {
    /// 页码，默认 1
    #[param(example = 1)]
    pub page: Option<u64>,
    /// 每页数量，默认 20
    #[param(example = 20)]
    pub limit: Option<u64>,
    /// 筛选类型：text / image / file
    #[param(example = "text")]
    pub r#type: Option<String>,
    /// 搜索关键词（搜索文本内容，仅对 text 类型有效）
    #[param(example = "关键词")]
    pub q: Option<String>,
    /// 开始日期（YYYY-MM-DD），筛选该日期之后创建的内容
    #[param(example = "2026-05-01")]
    pub start_date: Option<String>,
    /// 结束日期（YYYY-MM-DD），筛选该日期之前创建的内容
    #[param(example = "2026-05-27")]
    pub end_date: Option<String>,
}

/// 文件上传结果（七牛返回）
#[derive(Debug)]
pub struct UploadResult {
    pub url: String,
    pub key: String,
    pub file_name: String,
    pub file_size: i64,
    pub mime_type: String,
}
