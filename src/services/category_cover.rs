use crate::{config::AppError, models::categories};
use actix_web::{HttpResponse, web};
use sea_orm::{DatabaseConnection, EntityTrait};

/// Empty covers are computed at read time, so category changes never leave stale defaults.
pub fn cover_url(cover: Option<String>, category_id: i32) -> String {
    cover
        .filter(|url| !url.trim().is_empty() && !url.starts_with("/api/v1/posts/category-cover/"))
        .unwrap_or_else(|| format!("/api/v1/posts/category-cover/{category_id}.svg"))
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn render(name: &str, id: i32) -> String {
    // Stable per category: no random network calls, AI service or bitmap dependency.
    let seed = name
        .as_bytes()
        .iter()
        .fold(id.unsigned_abs(), |hash, byte| {
            hash.wrapping_mul(31).wrapping_add(u32::from(*byte))
        });
    let palettes = [
        ("#071f2b", "#104552", "#42e8c0", "#a8fff0"),
        ("#111b39", "#283b74", "#8aa8ff", "#d5e4ff"),
        ("#251831", "#633858", "#ffa8b9", "#ffe3c2"),
        ("#24251b", "#535b32", "#d9ed96", "#f4f5cd"),
    ];
    let (base, shade, accent, soft) = palettes[seed as usize % palettes.len()];
    let label: String = name.chars().take(28).collect();
    let count = label.chars().count();
    let font_size = if count > 10 {
        44
    } else if count > 7 {
        58
    } else {
        76
    };
    let first: String = label.chars().take(14).collect();
    let second: String = label.chars().skip(14).collect();
    let subtitle_y = if second.is_empty() { 358 } else { 408 };
    let second_line = if second.is_empty() {
        String::new()
    } else {
        format!(
            r#"<text x="80" y="335" font-size="{font_size}">{}</text>"#,
            escape(&second)
        )
    };
    let visual = match seed % 3 {
        0 => {
            r##"<g transform="translate(898 290)"><circle r="176" fill="none" stroke="url(#ink)" stroke-width="1"/><circle r="136" fill="none" stroke="url(#ink)" stroke-width="1" opacity=".5"/><ellipse rx="205" ry="76" transform="rotate(-35)" fill="none" stroke="url(#ink)" stroke-width="2"/><ellipse rx="205" ry="76" transform="rotate(35)" fill="none" stroke="url(#ink)" stroke-width="1" opacity=".55"/><circle r="75" fill="url(#sphere)"/><circle cx="-156" cy="92" r="11" fill="url(#ink)"/><circle cx="132" cy="-110" r="5" fill="#ffffff"/><path d="M-217 0h38M179 0h38M0-217v38M0 179v38" stroke="url(#ink)" opacity=".6"/></g>"##
        }
        1 => {
            r##"<g transform="translate(770 123)"><path d="M0 72 147 0 318 84 168 163Z" fill="url(#ink)" opacity=".18"/><path d="M0 72v184l168 94V163Z" fill="url(#ink)" opacity=".08"/><path d="M168 163 318 84v182l-150 84Z" fill="url(#ink)" opacity=".25"/><path d="M0 72 147 0 318 84v182l-150 84L0 256ZM0 72l168 91 150-79M168 163v187" stroke="url(#ink)" stroke-width="1.5" fill="none" opacity=".85"/><path d="m40 110 146-71 91 45-109 57Z" fill="url(#ink)" opacity=".25"/><path d="m40 178 66-33 91 46-67 34Z" fill="url(#ink)" opacity=".6"/><path d="m40 220 66-33 91 46-67 34Z" fill="url(#ink)" opacity=".3"/><path d="m208 194 69-35v54l-69 35Z" fill="url(#ink)" opacity=".45"/><circle cx="318" cy="84" r="5" fill="#ffffff"/><circle cx="0" cy="256" r="5" fill="url(#ink)"/></g>"##
        }
        _ => {
            r##"<g transform="translate(880 290) rotate(-24)"><rect x="-122" y="-185" width="235" height="335" rx="116" fill="url(#ink)" opacity=".12"/><rect x="-65" y="-138" width="122" height="236" rx="61" fill="none" stroke="url(#ink)" stroke-width="2"/><rect x="-155" y="-100" width="310" height="190" rx="94" fill="none" stroke="url(#ink)" stroke-width="1"/><path d="M-205 140C-110-70 150 200 210-140" stroke="url(#ink)" stroke-width="26" fill="none" opacity=".6"/><path d="M-205 140C-110-70 150 200 210-140" stroke="#ffffff" stroke-width="1" fill="none" opacity=".7"/><circle cx="-158" cy="-123" r="16" fill="url(#ink)"/><circle cx="153" cy="144" r="6" fill="#ffffff"/></g>"##
        }
    };
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="630" viewBox="0 0 1200 630" role="img" aria-label="{label}">
<title>{label}</title><defs>
<linearGradient id="bg" x2="1" y2="1"><stop stop-color="{base}"/><stop offset="1" stop-color="{shade}"/></linearGradient>
<linearGradient id="ink" x2="1" y2="1"><stop stop-color="{soft}"/><stop offset="1" stop-color="{accent}"/></linearGradient>
<radialGradient id="glow"><stop stop-color="{accent}" stop-opacity=".25"/><stop offset="1" stop-color="{accent}" stop-opacity="0"/></radialGradient>
<radialGradient id="sphere" cx="30%" cy="25%" r="80%"><stop stop-color="{soft}"/><stop offset=".4" stop-color="{accent}"/><stop offset="1" stop-color="{shade}"/></radialGradient>
<pattern id="grid" width="48" height="48" patternUnits="userSpaceOnUse"><path d="M48 0H0v48" fill="none" stroke="#ffffff" stroke-opacity=".045" stroke-width="1"/><circle cx="0" cy="0" r="1" fill="#ffffff" fill-opacity=".16"/></pattern>
</defs><rect width="1200" height="630" fill="url(#bg)"/><rect width="1200" height="630" fill="url(#grid)"/><ellipse cx="930" cy="230" rx="450" ry="420" fill="url(#glow)"/>
<path d="M-100 610 550-40M280 760 1030-80" stroke="#ffffff" stroke-width="1" opacity=".035"/>
<g font-family="sans-serif"><rect x="80" y="72" width="112" height="32" rx="16" fill="{accent}" fill-opacity=".12"/><circle cx="98" cy="88" r="3" fill="{accent}"/><text x="112" y="94" font-size="14" letter-spacing="2" fill="{soft}">分类专栏</text>
<path d="M80 176h38" stroke="{accent}" stroke-width="4"/>
<g fill="#ffffff" font-weight="600"><text x="80" y="270" font-size="{font_size}">{first}</text>{second_line}</g>
<text x="83" y="{subtitle_y}" font-size="16" letter-spacing="4" fill="{soft}" opacity=".65">IDEAS / BUILD / SHARE</text>
{visual}
<path d="M80 527h1040" stroke="#ffffff" opacity=".16"/><text x="80" y="565" font-size="13" letter-spacing="3" fill="#ffffff" opacity=".45">NOTES &amp; PERSPECTIVES</text><text x="1120" y="565" text-anchor="end" font-size="13" letter-spacing="2" fill="{soft}" opacity=".65">精选内容 · 持续更新</text></g></svg>"##,
        label = escape(&label),
        first = escape(&first)
    )
}

pub async fn handler(
    db: web::Data<DatabaseConnection>,
    id: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    let category = categories::Entity::find_by_id(*id)
        .one(db.get_ref())
        .await?
        .ok_or_else(|| AppError::NotFound("分类不存在".into()))?;
    Ok(HttpResponse::Ok()
        .content_type("image/svg+xml; charset=utf-8")
        .insert_header(("Cache-Control", "public, max-age=0, must-revalidate"))
        .insert_header(("X-Content-Type-Options", "nosniff"))
        .insert_header((
            "Content-Security-Policy",
            "default-src 'none'; style-src 'unsafe-inline'",
        ))
        .body(render(&category.name, category.id)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn defaults_follow_category_and_uploads_win() {
        assert_eq!(cover_url(None, 3), "/api/v1/posts/category-cover/3.svg");
        assert_eq!(
            cover_url(Some(" ".into()), 4),
            "/api/v1/posts/category-cover/4.svg"
        );
        assert_eq!(
            cover_url(Some("/api/v1/posts/category-cover/3.svg".into()), 4),
            "/api/v1/posts/category-cover/4.svg"
        );
        assert_eq!(
            cover_url(Some("https://cdn.example/image.webp".into()), 4),
            "https://cdn.example/image.webp"
        );
    }
    #[test]
    fn category_text_is_escaped_and_bounded() {
        let svg = render("<script> & \"分类\"", 1);
        assert!(!svg.contains("<script>"));
        assert!(svg.contains("&lt;script&gt; &amp; &quot;分类&quot;"));
        assert!(!render(&"字".repeat(100), 1).contains(&"字".repeat(29)));
    }
}
