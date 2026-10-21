use actix_web::{
    Error, HttpMessage,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use std::{
    future::{Future, Ready, ready},
    pin::Pin,
};

use crate::config::AppError;
use actix_web::{http::Method, web};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::rc::Rc;

#[derive(Clone)]
pub struct AdminAccess;
#[derive(Default)]
pub struct Auth {
    db: Option<web::Data<DatabaseConnection>>,
}
impl Auth {
    pub fn new(db: web::Data<DatabaseConnection>) -> Self {
        Self { db: Some(db) }
    }
}

impl<S, B> Transform<S, ServiceRequest> for Auth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware {
            service: Rc::new(service),
            db: self.db.clone(),
        }))
    }
}

pub struct AuthMiddleware<S> {
    service: Rc<S>,
    db: Option<web::Data<DatabaseConnection>>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;
    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let db = self.db.clone();
        Box::pin(async move {
            authorize(&req, db.as_ref()).await?;
            service.call(req).await
        })
    }
}

fn within(path: &str, prefix: &str) -> bool {
    path == prefix
        || path
            .strip_prefix(prefix)
            .is_some_and(|rest| rest.starts_with('/'))
}

fn public_request(path: &str, method: &Method) -> bool {
    if *method == Method::OPTIONS {
        return true;
    }
    if [
        "/api/v1/auth/login",
        "/api/v1/auth/register",
        "/api/v1/auth/logout",
        "/api/v1/email/send-verification-code",
        "/api/v1/feedback",
        "/api/v1/subscriptions",
    ]
    .contains(&path)
    {
        return *method == Method::POST;
    }
    // Clipboard and room handlers authenticate their own tokens. Drop uploads
    // remain public at the route layer so text can be anonymous; media uploads
    // check the JWT claims inserted below.
    if within(path, "/api/v1/clipboard") {
        return path != "/api/v1/clipboard/channel";
    }
    if within(path, "/api/v1/drop") {
        return *method == Method::GET || *method == Method::POST;
    }
    if within(path, "/api/v1/rooms") || ["/api/v1/ws", "/api/v1/sse"].contains(&path) {
        return true;
    }
    if *method == Method::POST
        && path
            .strip_prefix("/api/v1/posts/")
            .is_some_and(|rest| rest.ends_with("/view") && rest.matches('/').count() == 1)
    {
        return true;
    }
    if [
        "/api/v1/users/default-avatar.svg",
        "/api/v1/users/default-avatar.webp",
    ]
    .contains(&path)
    {
        return *method == Method::GET || *method == Method::HEAD;
    }
    if within(path, "/api/v1/images") {
        return path.starts_with("/api/v1/images/file/")
            && (*method == Method::GET || *method == Method::HEAD);
    }
    [
        "/api/v1/posts",
        "/api/v1/tags",
        "/api/v1/categories",
        "/api/v1/links",
        "/api/v1/version",
    ]
    .iter()
    .any(|prefix| within(path, prefix))
        && (*method == Method::GET || *method == Method::HEAD)
}

async fn authorize(
    req: &ServiceRequest,
    db: Option<&web::Data<DatabaseConnection>>,
) -> Result<(), Error> {
    let claims = req
        .cookie("access_token")
        .and_then(|cookie| crate::utils::jwt::decode_jwt(cookie.value()).ok());
    let public = public_request(req.path(), req.method());
    let Some(claims) = claims else {
        return if public {
            Ok(())
        } else {
            Err(AppError::Unauthorized("请先登录".into()).into())
        };
    };
    let db = db.ok_or_else(|| AppError::InternalServerError("鉴权服务暂时不可用".into()))?;
    let token = req
        .cookie("access_token")
        .ok_or_else(|| AppError::Unauthorized("请先登录".into()))?;
    if crate::utils::token_revocation::is_revoked(db.get_ref(), token.value()).await? {
        return if public {
            Ok(())
        } else {
            Err(AppError::Unauthorized("会话已注销".into()).into())
        };
    }
    let needs_admin = !public;
    // Published/draft visibility on content reads depends on current DB roles.
    if needs_admin || within(req.path(), "/api/v1/posts") {
        let user = crate::models::users::Entity::find_by_uuid(&claims.user_uuid)
            .one(db.get_ref())
            .await
            .map_err(|_| AppError::InternalServerError("鉴权服务暂时不可用".into()))?
            .ok_or_else(|| AppError::Unauthorized("用户不存在".into()))?;
        let assignments = crate::models::user_roles::Entity::find()
            .filter(crate::models::user_roles::Column::UserId.eq(user.id))
            .all(db.get_ref())
            .await
            .map_err(|_| AppError::InternalServerError("鉴权服务暂时不可用".into()))?;
        let role_ids: Vec<i32> = assignments
            .iter()
            .filter_map(|assignment| assignment.role_id)
            .collect();
        let roles = crate::models::roles::Entity::find()
            .filter(crate::models::roles::Column::Id.is_in(role_ids))
            .all(db.get_ref())
            .await
            .map_err(|_| AppError::InternalServerError("鉴权服务暂时不可用".into()))?;
        let admin = roles
            .iter()
            .any(|role| role.code == "ADMIN" || role.code == "SUPER_ADMIN");
        if needs_admin && !admin {
            return Err(AppError::Forbidden("需要管理员权限".into()).into());
        }
        if admin {
            req.extensions_mut().insert(AdminAccess);
        }
    }
    req.extensions_mut().insert(claims);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{App, HttpResponse, test};

    #[actix_web::test]
    async fn anonymous_management_writes_are_blocked() {
        let app = test::init_service(
            App::new()
                .wrap(Auth::default())
                .default_service(web::to(|| async { HttpResponse::Ok().finish() })),
        )
        .await;
        for path in [
            "/api/v1/categories",
            "/api/v1/tags",
            "/api/v1/links",
            "/api/v1/posts",
            "/api/v1/images/upload",
            "/api/v1/upload",
            "/api/v1/clipboard/channel",
        ] {
            let response =
                test::try_call_service(&app, test::TestRequest::post().uri(path).to_request())
                    .await;
            assert!(response.is_err(), "{path} must require authentication");
        }
        for path in ["/api/v1/posts", "/api/v1/tags", "/api/v1/clipboard"] {
            assert_eq!(
                test::call_service(&app, test::TestRequest::get().uri(path).to_request())
                    .await
                    .status(),
                200
            );
        }
        assert!(
            test::try_call_service(
                &app,
                test::TestRequest::get()
                    .uri("/api/v1/posts-malicious")
                    .to_request()
            )
            .await
            .is_err()
        );
        assert_eq!(
            test::call_service(
                &app,
                test::TestRequest::post()
                    .uri("/api/v1/posts/uuid/view")
                    .to_request()
            )
            .await
            .status(),
            200
        );
        assert_eq!(
            test::call_service(
                &app,
                test::TestRequest::post()
                    .uri("/api/v1/clipboard/channel/auth")
                    .to_request()
            )
            .await
            .status(),
            200
        );
    }
    #[actix_web::test]
    async fn management_access_uses_current_database_roles() {
        use actix_web::cookie::Cookie;
        use sea_orm::{ActiveModelTrait, IntoActiveModel, Set};
        let db = crate::services::auth_test::tests::setup_test_db().await;
        let user = crate::services::auth::AuthService::register(
            web::Data::new(db.clone()),
            web::Json(crate::dto::user::RegisterResponse {
                user_name: "rolecheck".into(),
                pass_word: "password123".into(),
            }),
            web::Data::new(crate::SseNotifier::new()),
        )
        .await
        .unwrap();
        let claims = crate::utils::jwt::TokenClaims {
            user_uuid: user.uuid,
            user_name: user.user_name,
            exp: chrono::Utc::now().timestamp() + 3600,
            role_id: 999,
            jti: None,
        };
        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(
                crate::config::manager::CONFIG.jwt.secret.as_bytes(),
            ),
        )
        .unwrap();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(db.clone()))
                .wrap(Auth::new(web::Data::new(db.clone())))
                .default_service(web::to(|| async { HttpResponse::Ok().finish() })),
        )
        .await;
        let request = || {
            test::TestRequest::post()
                .uri("/api/v1/categories")
                .cookie(Cookie::new("access_token", token.clone()))
                .to_request()
        };
        let denied = test::try_call_service(&app, request()).await.err().unwrap();
        assert_eq!(denied.as_response_error().status_code(), 403, "{denied:?}");
        let role = crate::models::roles::Entity::find()
            .filter(crate::models::roles::Column::Code.eq("ADMIN"))
            .one(&db)
            .await
            .unwrap()
            .unwrap();
        let assignment = crate::models::user_roles::Entity::find()
            .filter(crate::models::user_roles::Column::UserId.eq(user.id))
            .one(&db)
            .await
            .unwrap()
            .unwrap();
        let mut assignment = assignment.into_active_model();
        assignment.role_id = Set(Some(role.id));
        let assignment = assignment.update(&db).await.unwrap();
        assert_eq!(test::call_service(&app, request()).await.status(), 200);
        assignment.into_active_model().delete(&db).await.unwrap();
        assert_eq!(
            test::try_call_service(&app, request())
                .await
                .err()
                .unwrap()
                .as_response_error()
                .status_code(),
            403
        );
    }
}
