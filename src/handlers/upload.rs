use actix_multipart::Multipart;

use crate::{HttpResult, upload::UploadManager};

#[utoipa::path(
    post,
    path = "/api/v1/upload",
    tag = "文件上传",
    summary = "上传文件到七牛云",
    description = "上传通用文件（图片等），返回七牛云访问 URL",
    request_body(content = String, description = "文件数据（multipart/form-data）"),
    responses(
        (status = 200, description = "上传成功", body = crate::ApiResponse<crate::upload::UploadResult>),
        (status = 400, description = "请求参数错误", body = crate::ApiResponse<crate::EmptyResponse>),
        (status = 500, description = "服务器内部错误", body = crate::ApiResponse<crate::EmptyResponse>)
    )
)]
pub async fn upload_file_handler(payload: Multipart) -> HttpResult {
    let upload_manager = UploadManager::default();
    let result = upload_manager.handle_upload(payload).await?;
    Ok(result)
}
