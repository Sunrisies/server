use actix_multipart::Multipart;

use crate::{HttpResult, upload::UploadManager};

/// 处理文件上传
pub async fn upload_file_handler(payload: Multipart) -> HttpResult {
    let upload_manager = UploadManager::default();
    let result = upload_manager.handle_upload(payload).await?;
    Ok(result)
}
