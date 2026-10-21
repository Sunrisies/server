use super::*;
use std::io::Cursor;

fn png(width: u32, height: u32) -> Vec<u8> {
    let image = image::DynamicImage::new_rgba8(width, height);
    let mut output = Cursor::new(Vec::new());
    image.write_to(&mut output, ImageFormat::Png).unwrap();
    output.into_inner()
}

// A valid PNG header with extreme dimensions and a tiny body: no large allocation in the test.
fn header_png(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = png(1, 1);
    bytes[16..20].copy_from_slice(&width.to_be_bytes());
    bytes[20..24].copy_from_slice(&height.to_be_bytes());
    let mut crc = !0u32;
    for &byte in &bytes[12..29] {
        crc ^= byte as u32;
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb88320u32.wrapping_mul(crc & 1));
        }
    }
    bytes[29..33].copy_from_slice(&(!crc).to_be_bytes());
    bytes
}

fn upload_result(path: &Path, name: &str, _: &str) -> Result<UploadResult, AppError> {
    assert!(path.is_file());
    Ok(UploadResult {
        url: "https://example.invalid/test.png".into(),
        key: "images/test.png".into(),
        filename: name.into(),
        size: fs::metadata(path)?.len() as i64,
        human_readable_size: "test".into(),
        mime_type: "image/png".into(),
    })
}

async fn temp_bytes(dir: &Path, bytes: &[u8]) -> TempFile {
    let (mut temp, mut writer) = create_temp("test.png".into(), dir.into()).await.unwrap();
    writer.write_all(bytes).await.unwrap();
    writer.flush().await.unwrap();
    drop(writer);
    temp.size = bytes.len() as i64;
    temp
}

#[test]
fn image_dimensions_and_decoded_allocation_are_both_enforced() {
    for bytes in [
        header_png(8193, 1),
        header_png(1, 8193),
        header_png(5000, 5000),
    ] {
        let reader = ImageReader::with_format(Cursor::new(bytes), ImageFormat::Png);
        assert!(matches!(
            limited_decode(reader),
            Err(image::ImageError::Limits(_))
        ));
    }
}

#[test]
fn formats_are_preserved_and_small_images_are_not_upscaled() {
    let dir = tempfile::tempdir().unwrap();
    for (name, format) in [
        ("small.png", ImageFormat::Png),
        ("small.JPG", ImageFormat::Jpeg),
        ("small.webp", ImageFormat::WebP),
    ] {
        let path = dir.path().join(name);
        image::DynamicImage::new_rgb8(32, 16)
            .save_with_format(&path, format)
            .unwrap();
        process_image(&path, name).unwrap();
        let reader = ImageReader::open(&path)
            .unwrap()
            .with_guessed_format()
            .unwrap();
        assert_eq!(reader.format(), Some(format));
        assert_eq!(reader.into_dimensions().unwrap(), (32, 16));
    }
}

#[test]
fn oversized_valid_image_is_downscaled_with_aspect_ratio() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("large.png");
    fs::write(&path, png(2400, 1200)).unwrap();
    process_image(&path, "large.png").unwrap();
    assert_eq!(image::image_dimensions(path).unwrap(), (1920, 960));
}

#[tokio::test]
async fn cleanup_on_success_decode_failure_and_upload_failure() {
    let dir = tempfile::tempdir().unwrap();
    let slots = Arc::new(Semaphore::new(1));
    for mode in ["success", "decode-failure", "upload-failure"] {
        let temp = temp_bytes(
            dir.path(),
            if mode == "decode-failure" {
                b"not an image".to_vec()
            } else {
                png(4, 4)
            }
            .as_slice(),
        )
        .await;
        let path = temp.path().to_owned();
        let permit = slots.clone().acquire_owned().await.unwrap();
        let result = finish_upload(
            temp,
            permit,
            true,
            "images".into(),
            move |path, name, prefix| {
                if mode == "upload-failure" {
                    Err(AppError::UploadFailed("injected upload failure".into()))
                } else {
                    upload_result(path, name, prefix)
                }
            },
        )
        .await;
        assert_eq!(result.is_ok(), mode == "success");
        assert!(!path.exists());
        assert_eq!(slots.available_permits(), 1);
    }
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
}

#[tokio::test]
async fn cleanup_on_worker_panic() {
    let dir = tempfile::tempdir().unwrap();
    let temp = temp_bytes(dir.path(), &png(4, 4)).await;
    let path = temp.path().to_owned();
    let slots = Arc::new(Semaphore::new(1));
    let permit = slots.clone().acquire_owned().await.unwrap();
    let result = finish_upload(temp, permit, false, "images".into(), |_, _, _| {
        panic!("injected worker panic")
    })
    .await;
    assert!(result.is_err());
    assert!(!path.exists());
    assert_eq!(slots.available_permits(), 1);
}

#[actix_web::test]
async fn slow_uploader_does_not_stall_request_runtime() {
    let dir = tempfile::tempdir().unwrap();
    let temp = temp_bytes(dir.path(), &png(4, 4)).await;
    let slots = Arc::new(Semaphore::new(1));
    let permit = slots.acquire_owned().await.unwrap();
    let (started, ready) = tokio::sync::oneshot::channel();
    let (release, wait) = std::sync::mpsc::channel();
    let task = actix_web::rt::spawn(finish_upload(
        temp,
        permit,
        false,
        "images".into(),
        move |path, name, prefix| {
            started.send(()).unwrap();
            wait.recv_timeout(Duration::from_secs(2)).unwrap();
            upload_result(path, name, prefix)
        },
    ));
    tokio::time::timeout(Duration::from_secs(1), ready)
        .await
        .unwrap()
        .unwrap();
    // A real handler executes on this same single-thread Actix runtime while upload is blocked.
    let app = actix_web::test::init_service(actix_web::App::new().route(
        "/ready",
        actix_web::web::get().to(|| async { actix_web::HttpResponse::Ok().finish() }),
    ))
    .await;
    let response = tokio::time::timeout(
        Duration::from_millis(200),
        actix_web::test::call_service(
            &app,
            actix_web::test::TestRequest::get()
                .uri("/ready")
                .to_request(),
        ),
    )
    .await
    .unwrap();
    assert_eq!(response.status(), 200);
    assert!(!task.is_finished());
    release.send(()).unwrap();
    assert!(task.await.unwrap().is_ok());
}

#[actix_web::test]
async fn cancelled_request_keeps_guard_and_slot_until_worker_finishes() {
    let dir = tempfile::tempdir().unwrap();
    let temp = temp_bytes(dir.path(), &png(4, 4)).await;
    let path = temp.path().to_owned();
    let slots = Arc::new(Semaphore::new(1));
    let permit = slots.clone().acquire_owned().await.unwrap();
    let (started, ready) = tokio::sync::oneshot::channel();
    let (release, wait) = std::sync::mpsc::channel();
    let task = actix_web::rt::spawn(finish_upload(
        temp,
        permit,
        false,
        "images".into(),
        move |path, name, prefix| {
            started.send(()).unwrap();
            wait.recv_timeout(Duration::from_secs(2)).unwrap();
            upload_result(path, name, prefix)
        },
    ));
    ready.await.unwrap();
    task.abort();
    let _ = task.await;
    assert!(path.exists());
    assert_eq!(slots.available_permits(), 0);
    release.send(()).unwrap();
    let _permit = tokio::time::timeout(Duration::from_secs(2), slots.acquire_owned())
        .await
        .unwrap()
        .unwrap();
    assert!(!path.exists());
}

#[tokio::test]
async fn saturated_upload_slots_return_bounded_busy_error() {
    let slots = Arc::new(Semaphore::new(2));
    let _first = slots.clone().acquire_owned().await.unwrap();
    let _second = slots.clone().acquire_owned().await.unwrap();
    let result = tokio::time::timeout(Duration::from_secs(3), acquire_upload_slot(slots))
        .await
        .unwrap();
    assert!(matches!(result, Err(AppError::MaintenanceMode(_))));
}

#[actix_web::test]
async fn malformed_or_oversized_streams_leave_no_temporary_files() {
    let directory = Path::new("temp_uploads");
    let before: std::collections::HashSet<_> = fs::read_dir(directory)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .collect();
    for data in [
        b"--upload-test\r\nContent-Disposition: form-data; name=\"file\"; filename=\"test.png\"\r\n\r\n123456789\r\n--upload-test--\r\n".as_slice(),
        b"--upload-test\r\nContent-Disposition: form-data; name=\"file\"; filename=\"test.png\"\r\n\r\ntruncated".as_slice(),
    ] {
        let req=actix_web::test::TestRequest::default().insert_header(("content-type","multipart/form-data; boundary=upload-test")).to_http_request();
        let stream=futures_util::stream::once(async move {Ok::<_,actix_web::error::PayloadError>(actix_web::web::Bytes::copy_from_slice(data))});
        let mut multipart=actix_multipart::Multipart::new(req.headers(),stream);
        let mut field=multipart.next().await.unwrap().unwrap();
        assert!(save_to_temp(&mut field,if data.ends_with(b"--\r\n") {4} else {100}).await.is_err());
    }
    let after: std::collections::HashSet<_> = fs::read_dir(directory)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    assert_eq!(before, after);
}
