use chrono::{DateTime, Utc};

/// 序列化函数：2025-09-24 13:44:29
pub fn fmt_beijing<S>(dt: &DateTime<Utc>, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.serialize_str(&dt.format("%Y-%m-%d %H:%M:%S").to_string())
}
