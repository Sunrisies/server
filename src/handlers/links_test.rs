#[cfg(test)]
mod tests {
    use crate::ApiResponse;
    use crate::dto::link::CreateLinkRequest;
    use crate::handlers::links::{
        create_link_secure_handler, list_links_handler, redirect_link_handler,
    };
    use actix_web::HttpRequest;
    use actix_web::web;
    use sea_orm::{ConnectionTrait, Database, DatabaseConnection, Statement};

    async fn setup_test_db() -> DatabaseConnection {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        let create_links = Statement::from_string(
            db.get_database_backend(),
            r#"
            CREATE TABLE external_links (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                uuid TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                description TEXT,
                url TEXT NOT NULL,
                protocol TEXT NOT NULL,
                icon_url TEXT,
                category TEXT NOT NULL,
                tags TEXT,
                click_count INTEGER DEFAULT 0,
                is_active INTEGER DEFAULT 1,
                is_blocked INTEGER DEFAULT 0,
                blocked_reason TEXT,
                safety_score INTEGER DEFAULT 100,
                created_at TEXT,
                updated_at TEXT,
                valid_from TEXT,
                valid_to TEXT,
                access_control TEXT,
                created_by INTEGER
            );
            "#
            .to_string(),
        );
        db.execute(create_links).await.unwrap();
        let create_clicks = Statement::from_string(
            db.get_database_backend(),
            r#"
            CREATE TABLE link_clicks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                link_id INTEGER NOT NULL,
                user_id INTEGER,
                referrer TEXT,
                ip TEXT,
                user_agent TEXT,
                clicked_at TEXT
            );
            "#
            .to_string(),
        );
        db.execute(create_clicks).await.unwrap();
        db
    }

    #[tokio::test]
    async fn test_create_and_list_links() {
        let db = setup_test_db().await;
        let db_pool = web::Data::new(db);
        let req = CreateLinkRequest {
            name: "Rust 官网".to_string(),
            description: Some("Rust 官方网站".to_string()),
            url: "https://www.rust-lang.org".to_string(),
            icon_url: None,
            category: "docs".to_string(),
            tags: Some(vec!["rust".to_string(), "lang".to_string()]),
            valid_from: None,
            valid_to: None,
            visibility: Some("public".to_string()),
            allowed_roles: None,
        };

        let created = create_link_secure_handler(db_pool.clone(), web::Json(req)).await;
        assert!(created.is_ok());

        let list = list_links_handler(
            db_pool.clone(),
            web::Query(crate::dto::link::LinkFilterQuery {
                page: 1,
                limit: 10,
                category: Some("docs".to_string()),
                q: None,
                tags: None,
            }),
        )
        .await;
        assert!(list.is_ok());
    }

    #[tokio::test]
    async fn test_redirect_records_click() {
        let db = setup_test_db().await;
        let db_pool = web::Data::new(db);
        let req = CreateLinkRequest {
            name: "Actix".to_string(),
            description: None,
            url: "https://actix.rs".to_string(),
            icon_url: None,
            category: "framework".to_string(),
            tags: None,
            valid_from: None,
            valid_to: None,
            visibility: Some("public".to_string()),
            allowed_roles: None,
        };
        assert!(
            create_link_secure_handler(db_pool.clone(), web::Json(req))
                .await
                .is_ok()
        );
        let http_req = HttpRequest::default();
        let res = redirect_link_handler(db_pool.clone(), web::Path::from(1), http_req).await;
        assert!(res.is_ok());
    }
}
