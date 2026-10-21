//! Read-only EXPLAIN of representative public queries, restricted to the local acceptance DB.
use sea_orm::{
    ColumnTrait, ConnectionTrait, Database, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
    QueryTrait,
};
use serde_json::json;
use web_server::models::{categories, post_tags, posts, tags};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::var("DATABASE_URL")?;
    let parsed = url::Url::parse(&url)?;
    if !matches!(parsed.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"))
        || parsed.path() != "/server_acceptance"
    {
        return Err("Only the isolated local acceptance database is allowed".into());
    }
    let db = Database::connect(url).await?;
    let backend = db.get_database_backend();
    let list = posts::Entity::find()
        .find_also_related(categories::Entity)
        .filter(posts::Column::Status.eq(1))
        .order_by_desc(posts::Column::CreatedAt)
        .order_by_desc(posts::Column::Id)
        .limit(12)
        .offset(0)
        .build(backend);
    let detail = posts::Entity::find()
        .filter(posts::Column::Uuid.eq("00000000-0000-4000-8000-000000000001"))
        .filter(posts::Column::Status.eq(1))
        .limit(1)
        .build(backend);
    let tag_batch = post_tags::Entity::find()
        .filter(post_tags::Column::PostId.is_in(1..=12))
        .find_also_related(tags::Entity)
        .build(backend);
    let mut plans = vec![];
    for (name, mut statement) in [
        ("public-list-first-page", list),
        ("public-detail-by-uuid", detail),
        ("tags-batch-12-posts", tag_batch),
    ] {
        statement.sql = format!("EXPLAIN (ANALYZE, BUFFERS, FORMAT JSON) {}", statement.sql);
        let row = db
            .query_one(statement)
            .await?
            .ok_or("EXPLAIN returned no row")?;
        let plan: serde_json::Value = row.try_get("", "QUERY PLAN")?;
        plans.push(json!({"query": name, "explain": plan}));
    }
    println!("{}", serde_json::to_string_pretty(&plans)?);
    db.close().await?;
    Ok(())
}
