use chrono::{DateTime, FixedOffset, Utc};

/// UTC -> 北京时间 (+8)
fn to_beijing(dt: DateTime<Utc>) -> DateTime<FixedOffset> {
    let beijing = FixedOffset::east_opt(8 * 3600).unwrap();
    dt.with_timezone(&beijing)
}

/// 序列化函数：2025-09-24 13:44:29
pub fn fmt_beijing<S>(dt: &DateTime<Utc>, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let bj = to_beijing(*dt);
    s.serialize_str(&bj.format("%Y-%m-%d %H:%M:%S").to_string())
}
