use actix_web::HttpResponse;

pub const DEFAULT_AVATAR: &str = "/api/v1/users/default-avatar.webp?v=3";
pub fn avatar_url(image: Option<&str>) -> String {
    image
        .map(str::trim)
        .filter(|url| {
            !url.is_empty()
                && !url.chars().any(char::is_whitespace)
                && !url.contains(['<', '>', '\\'])
                && ((url.starts_with('/') && !url.starts_with("//"))
                    || url.starts_with("https://")
                    || url.starts_with("http://"))
        })
        .unwrap_or(DEFAULT_AVATAR)
        .to_owned()
}
pub async fn legacy_handler() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("image/svg+xml; charset=utf-8")
        .insert_header(("Cache-Control", "public, max-age=86400"))
        .insert_header(("X-Content-Type-Options", "nosniff"))
        .insert_header(("Content-Security-Policy", "default-src 'none'"))
        .body(include_str!("../assets/default-avatar.svg"))
}
pub async fn handler() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("image/webp")
        .insert_header(("Cache-Control", "public, max-age=86400"))
        .insert_header(("X-Content-Type-Options", "nosniff"))
        .body(include_bytes!("../assets/default-avatar-cat.webp").as_slice())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_or_invalid_avatar_uses_default() {
        for value in [
            None,
            Some(""),
            Some("  "),
            Some("javascript:alert(1)"),
            Some("//example.org/x"),
            Some("/bad\\image"),
        ] {
            assert_eq!(avatar_url(value), DEFAULT_AVATAR);
        }
    }
    #[test]
    fn real_avatar_wins_over_default() {
        assert_eq!(
            avatar_url(Some("https://cdn.example/avatar.webp")),
            "https://cdn.example/avatar.webp"
        );
        assert_eq!(avatar_url(Some(" /avatars/me.webp ")), "/avatars/me.webp");
    }
}
