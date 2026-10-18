//! Explicit opt-in filesystem image storage. Public files use generated UUID names only.
use crate::{config::AppError, services::qiniu::UploadResult};
use actix_multipart::Field;
use actix_web::{HttpResponse, web};
use futures_util::StreamExt;
use image::{ImageFormat, ImageReader, Limits};
use std::{io::Cursor, path::PathBuf};
use uuid::Uuid;

pub const MAX_BYTES: usize = 10 * 1024 * 1024;

fn directory() -> PathBuf {
    std::env::var_os("LOCAL_IMAGE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("data/images"))
}
fn filename(key: &str) -> Result<&str, AppError> {
    let name = key.strip_prefix("local/").unwrap_or(key);
    let stem = name
        .strip_suffix(".webp")
        .ok_or_else(|| AppError::NotFound("图片不存在".into()))?;
    let uuid = Uuid::parse_str(stem).map_err(|_| AppError::NotFound("图片不存在".into()))?;
    if uuid.to_string() != stem {
        return Err(AppError::NotFound("图片不存在".into()));
    }
    Ok(name)
}
fn encode_image(bytes: Vec<u8>) -> Result<Vec<u8>, AppError> {
    let format = image::guess_format(&bytes)
        .map_err(|_| AppError::UnsupportedFileType("请上传 PNG、JPG 或 WebP 图片".into()))?;
    if !matches!(
        format,
        ImageFormat::Png | ImageFormat::Jpeg | ImageFormat::WebP
    ) {
        return Err(AppError::UnsupportedFileType(
            "请上传 PNG、JPG 或 WebP 图片".into(),
        ));
    }
    let mut reader = ImageReader::with_format(Cursor::new(bytes), format);
    let mut limits = Limits::default();
    limits.max_image_width = Some(8192);
    limits.max_image_height = Some(8192);
    limits.max_alloc = Some(64 * 1024 * 1024);
    reader.limits(limits);
    let image = reader
        .decode()
        .map_err(|_| AppError::BadRequest("图片损坏或尺寸过大，请选择其他图片".into()))?;
    let image = if image.width() > 1920 || image.height() > 1080 {
        image.resize(1920, 1080, image::imageops::FilterType::Lanczos3)
    } else {
        image
    };
    let mut output = Cursor::new(Vec::new());
    image
        .write_to(&mut output, ImageFormat::WebP)
        .map_err(|_| AppError::UploadFailed("图片处理失败，请重试".into()))?;
    Ok(output.into_inner())
}

pub async fn upload(field: &mut Field) -> Result<UploadResult, AppError> {
    let original = field
        .content_disposition()
        .and_then(|v| v.get_filename())
        .unwrap_or("image");
    let display_name: String = original
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or("image")
        .chars()
        .filter(|c| !c.is_control())
        .take(200)
        .collect();
    let mut bytes = Vec::new();
    while let Some(chunk) = field.next().await {
        let chunk = chunk.map_err(|_| AppError::BadRequest("图片上传中断，请重试".into()))?;
        if bytes.len() + chunk.len() > MAX_BYTES {
            return Err(AppError::FileTooLarge("图片不能超过 10MB".into()));
        }
        bytes.extend_from_slice(&chunk);
    }
    let encoded = web::block(move || encode_image(bytes))
        .await
        .map_err(|_| AppError::UploadFailed("图片处理失败，请重试".into()))??;
    let name = format!("{}.webp", Uuid::new_v4());
    let dir = directory();
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|_| AppError::StorageServiceError("创建图片目录失败".into()))?;
    let path = dir.join(&name);
    if tokio::fs::write(&path, &encoded).await.is_err() {
        let _ = tokio::fs::remove_file(&path).await;
        return Err(AppError::StorageServiceError("保存图片失败".into()));
    }
    Ok(UploadResult {
        url: format!("/api/v1/images/file/{name}"),
        key: format!("local/{name}"),
        filename: display_name,
        size: encoded.len() as i64,
        human_readable_size: crate::utils::file_size::FileSize::from(encoded.len() as i64)
            .to_string(),
        mime_type: "image/webp".into(),
    })
}

pub async fn remove(key: &str) -> Result<(), AppError> {
    let path = directory().join(filename(key)?);
    match tokio::fs::remove_file(path).await {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(AppError::StorageServiceError("清理图片失败".into())),
    }
}

pub async fn serve(path: web::Path<String>) -> Result<HttpResponse, AppError> {
    let name = path.into_inner();
    let name = filename(&name)?;
    let path = directory().join(name);
    let metadata = tokio::fs::metadata(&path)
        .await
        .map_err(|_| AppError::NotFound("图片不存在".into()))?;
    if !metadata.is_file() || metadata.len() > MAX_BYTES as u64 {
        return Err(AppError::NotFound("图片不存在".into()));
    }
    let bytes = tokio::fs::read(path)
        .await
        .map_err(|_| AppError::NotFound("图片不存在".into()))?;
    Ok(HttpResponse::Ok()
        .content_type("image/webp")
        .insert_header(("X-Content-Type-Options", "nosniff"))
        .insert_header(("Cache-Control", "public, max-age=31536000, immutable"))
        .body(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_generated_file_names_are_public() {
        let name = format!("{}.webp", Uuid::new_v4());
        assert!(filename(&name).is_ok());
        assert!(filename(&format!("local/{name}")).is_ok());
        for bad in [
            "../.env",
            "file.svg",
            "../../etc/passwd.webp",
            "image.webp",
            "local/../image.webp",
        ] {
            assert!(filename(bad).is_err());
        }
    }
    #[test]
    fn invalid_content_is_rejected_and_valid_image_is_reencoded() {
        assert!(encode_image(b"<svg onload='alert(1)'/>".to_vec()).is_err());
        let image = image::DynamicImage::new_rgb8(32, 20);
        let mut png = Cursor::new(Vec::new());
        image.write_to(&mut png, ImageFormat::Png).unwrap();
        let encoded = encode_image(png.into_inner()).unwrap();
        assert_eq!(image::guess_format(&encoded).unwrap(), ImageFormat::WebP);
        let decoded = image::load_from_memory(&encoded).unwrap();
        assert_eq!((decoded.width(), decoded.height()), (32, 20));
    }
}
