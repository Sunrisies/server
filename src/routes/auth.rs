use crate::handlers::auth::login;
use actix_web::{HttpResponse, web};

async fn registration_disabled() -> HttpResponse {
    HttpResponse::Forbidden().json(serde_json::json!({
        "code": 403, "message": "公开注册已关闭", "data": null
    }))
}

pub fn config_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/v1/auth")
            .route("/register", web::post().to(registration_disabled))
            .route("/login", web::post().to(login)),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{App, test};

    #[actix_web::test]
    async fn public_registration_is_disabled_without_database_access() {
        let app = test::init_service(App::new().configure(config_routes)).await;
        let request = test::TestRequest::post()
            .uri("/v1/auth/register")
            .set_json(serde_json::json!({"user_name":"newadmin", "pass_word":"password123"}))
            .to_request();
        let response = test::call_service(&app, request).await;
        assert_eq!(response.status(), 403);
        let body: serde_json::Value = test::read_body_json(response).await;
        assert_eq!(body["code"], 403);
    }
}
