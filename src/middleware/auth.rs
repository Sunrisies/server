use crate::{get_all_routes, utils::perm_cache::ROLE_PERMS};
use actix_web::{
    Error,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use sea_orm::DatabaseConnection;
use std::{
    future::{Future, Ready, ready},
    pin::Pin,
};

use crate::config::AppError;

pub type DbPool = DatabaseConnection;
pub struct Auth;

impl<S, B> Transform<S, ServiceRequest> for Auth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware { service }))
    }
}

pub struct AuthMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;
    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let path = req.path().to_string(); // 克隆 path
        // 完全匹配的路径
        let exact_paths = [
            "/api/v1/auth/login",
            "/api/v1/auth/register",
            "/api/v1/sse",
            "/api/v1/ws",
        ];
        // 前缀匹配的路径

        let prefix_paths = [
            "/api/v1/tags",
            "/api/v1/posts",
            "/api/v1/categories",
            "/api/v1/rooms",
            "/api/v1/upload",
        ];
        let is_public = exact_paths.contains(&path.as_str())
            || prefix_paths.iter().any(|&prefix| path.starts_with(prefix));

        if is_public {
            // 公开路径，直接调用服务
            let fut = self.service.call(req);
            Box::pin(async move {
                let res = fut.await?;
                Ok(res)
            })
        } else {
            // 需要认证的路径，先检查cookie
            let cookie_result = req.cookie("access_token").ok_or_else(|| {
                log::error!("access_token not found");
                AppError::Unauthorized("access_token not found".to_string())
            });
            match cookie_result {
                Ok(cookie) => {
                    let token = cookie.value();

                    // 验证令牌
                    match crate::utils::jwt::decode_jwt(
                        token,
                        "uZr0aHV8Z2dRa1NmYnJ0aXN0aGViZXN0a2V5",
                    ) {
                        Ok(claims) => {
                            log::info!("s: {:?}", claims);
                            // 令牌有效，调用服务
                            let fut = self.service.call(req);
                            Box::pin(async move {
                                let routes = get_all_routes();
                                log::info!("Registered {} routes:", routes.len());
                                for route in routes {
                                    log::info!(
                                        "  {} {} -> {}",
                                        route.method,
                                        route.path,
                                        route.permission
                                    );
                                }
                                let role_perms = ROLE_PERMS.read().await;
                                if let Some(perms) = role_perms.get(&claims.role_id) {
                                    log::info!("perms: {:?}", perms);
                                }
                                let res = fut.await?;
                                Ok(res)
                            })
                        }
                        Err(_) => Box::pin(async move {
                            Err(AppError::Unauthorized("无效的令牌".to_string()).into())
                        }),
                    }
                }
                Err(e) => Box::pin(async move { Err(Error::from(e)) }),
            }
        }
    }
}
