use actix_web::{
    Error, HttpMessage,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use std::{
    future::{Future, Ready, ready},
    pin::Pin,
};

use crate::config::AppError;
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
        let path = req.path().to_string();

        // 如果携带了 access_token cookie，尝试解析并注入用户信息
        // （供 handlers 中获取当前登录用户使用）
        if let Some(cookie) = req.cookie("access_token")
            && let Ok(claims) = crate::utils::jwt::decode_jwt(cookie.value())
        {
            log::info!("用户已登录: {} ({})", claims.user_name, claims.user_uuid);
            req.extensions_mut().insert(claims);
        }

        // 完全匹配的公开路径（无需认证）
        let exact_paths = [
            "/api/v1/auth/login",
            "/api/v1/auth/register",
            "/api/v1/sse",
            "/api/v1/ws",
        ];
        // 前缀匹配的公开路径（无需认证）
        let prefix_paths = [
            "/api/v1/tags",
            "/api/v1/posts",
            "/api/v1/categories",
            "/api/v1/clipboard",
            "/api/v1/rooms",
            "/api/v1/upload",
            "/api/v1/email",
            "/api/v1/version",
            "/api/v1/links",
            "/api/v1/images",
        ];
        let is_public = exact_paths.contains(&path.as_str())
            || prefix_paths.iter().any(|&prefix| path.starts_with(prefix));

        if is_public {
            // 公开路径，直接放行
            let fut = self.service.call(req);
            Box::pin(async move {
                let res = fut.await?;
                Ok(res)
            })
        } else {
            // 需要认证的路径，检查 JWT（先判断是否存在，释放借用后再 move req）
            let has_claims = req
                .extensions()
                .get::<crate::utils::jwt::TokenClaims>()
                .is_some();
            if has_claims {
                let fut = self.service.call(req);
                Box::pin(async move {
                    let res = fut.await?;
                    Ok(res)
                })
            } else {
                Box::pin(async move { Err(AppError::Unauthorized("请先登录".to_string()).into()) })
            }
        }
    }
}
