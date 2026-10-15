//! Explicit local-only fixture; never used by server startup.
use sea_orm::{ActiveModelTrait, ConnectionTrait, Database, EntityTrait, Schema, Set, Statement};
use web_server::{models::*, utils};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let url = std::env::var("DATABASE_URL")?;
    let parsed = url::Url::parse(&url)?;
    anyhow::ensure!(
        matches!(parsed.host_str(), Some("localhost" | "127.0.0.1")),
        "Local database required"
    );
    anyhow::ensure!(
        parsed.path() == "/server_acceptance",
        "Dedicated acceptance database required"
    );
    let password = std::env::var("LOCAL_ADMIN_PASSWORD")?;
    let db = Database::connect(url).await?;
    let schema = Schema::new(db.get_database_backend());
    macro_rules! table {
        ($($entity:ident),+ $(,)?) => {$({
            let mut table = schema.create_table_from_entity($entity::Entity);
            table.if_not_exists();
            db.execute(db.get_database_backend().build(&table)).await?;
        })+};
    }
    table!(
        users,
        roles,
        permissions,
        categories,
        tags,
        posts,
        post_tags,
        post_views,
        user_roles,
        role_permissions,
        user_permissions,
        images,
        uploads,
        clipboard_channels,
        clipboard_entries,
        rooms,
        room_messages,
        external_links,
        verification_codes,
        revoked_tokens
    );
    for sql in include_str!("../src/sql/performance_guards.sql").split(';') {
        if !sql.trim().is_empty() {
            db.execute(Statement::from_string(
                db.get_database_backend(),
                sql.to_owned(),
            ))
            .await?;
        }
    }
    utils::login_limit::initialize(&db).await?;
    if users::Entity::find().one(&db).await?.is_some() {
        println!("Local fixture already exists; preserved existing data.");
        return Ok(());
    }
    let now = chrono::Utc::now();
    let user = users::ActiveModel {
        uuid: Set("00000000-0000-4000-8000-000000000001".into()),
        user_name: Set("local_admin".into()),
        pass_word: Set(
            utils::crypto_pwd::hash(&password).map_err(|e| anyhow::anyhow!(e.to_string()))?
        ),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(&db)
    .await?;
    for code in ["ADMIN", "USER"] {
        let role = roles::ActiveModel {
            code: Set(code.into()),
            name: Set(code.into()),
            is_system: Set(Some(true)),
            created_at: Set(Some(now.into())),
            ..Default::default()
        }
        .insert(&db)
        .await?;
        if code == "ADMIN" {
            user_roles::ActiveModel {
                user_id: Set(Some(user.id)),
                role_id: Set(Some(role.id)),
                is_primary: Set(Some(true)),
                created_at: Set(now),
                ..Default::default()
            }
            .insert(&db)
            .await?;
        }
    }
    let category = categories::ActiveModel {
        name: Set("本地验收".into()),
        slug: Set("local-acceptance".into()),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(&db)
    .await?;
    let tag = tags::ActiveModel {
        name: Set("Nuxt".into()),
        created_at: Set(now),
        ..Default::default()
    }
    .insert(&db)
    .await?;
    let markdown = "## 本地验收\n\n这是隔离环境中的测试文章，用于验证 SSR、分页和文章管理。\n\n```js\nconst ready = true\nconsole.log(ready)\n```\n\n## 性能样本\n\n所有数据均为本地生成。\n".repeat(12);
    for index in 1..=525 {
        let post = posts::ActiveModel {
            uuid: Set(format!("00000000-0000-4000-8000-{index:012}")),
            author_id: Set(user.id),
            category_id: Set(category.id),
            title: Set(format!("本地验收文章 {index}")),
            summary: Set(Some("本地隔离数据，用于分页和压力验收".into())),
            content: Set("<p>本地测试 HTML</p>".into()),
            markdowncontent: Set(markdown.clone()),
            status: Set(1),
            featured: Set(index == 1),
            view_count: Set(0),
            size: Set(markdown.len() as i32),
            created_at: Set(now),
            updated_at: Set(now),
            published_at: Set(now),
            ..Default::default()
        }
        .insert(&db)
        .await?;
        post_tags::ActiveModel {
            post_id: Set(post.id),
            tag_id: Set(tag.id),
            ..Default::default()
        }
        .insert(&db)
        .await?;
    }
    println!(
        "Created 525 local articles, one category/tag, and local_admin. Password is in the local deployment env file."
    );
    Ok(())
}
