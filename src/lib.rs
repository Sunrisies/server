mod routes;
pub use routes::config_routes;
pub mod models;
pub use models::prelude;
pub mod config;
#[macro_use]
pub mod macros;
pub use config::create_db_pool;
pub mod dto;
pub use dto::RegisterResponse;
pub mod middleware;
pub use middleware::helpers::*;
pub mod utils;
pub use utils::sse::*;
pub mod handlers;
pub mod services;
pub use handlers::auth;
pub use services::*;

use inventory::collect;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::RwLock;
// 重新导出宏
pub use route_macros::route_permission;

// 定义路由信息结构体（必须与宏crate中的完全一致）
#[derive(Debug, Clone)]
pub struct RouteInfo {
    pub path: &'static str,
    pub method: &'static str,
    pub permission: &'static str,
    // pub handler: fn(), // 简化表示
}
// 收集所有被宏标记的路由
collect!(RouteInfo);

// 全局路由注册表
lazy_static! {
    pub static ref ROUTE_REGISTRY: RwLock<HashMap<String, RouteInfo>> = RwLock::new(HashMap::new());
}

/// 初始化路由注册表。
///
/// # Errors
///
/// 如果无法获取路由注册表的写锁（例如锁中毒），返回 `AppError::InternalServerError`。
pub fn init_route_registry() -> Result<(), AppError> {
    {
        let mut registry = ROUTE_REGISTRY
            .write()
            .map_err(|e| AppError::InternalServerError(format!("获取写锁失败: {e}")))?;

        // inventory::iter() 会返回所有被收集的 RouteInfo 实例
        for route_info in inventory::iter::<RouteInfo> {
            let key = format!("{}:{}", route_info.method.to_uppercase(), route_info.path);
            registry.insert(key, route_info.clone());
        }
    }
    Ok(())
}
/// 获取所有已注册的路由信息。
///
/// # Errors
///
/// 如果路由注册表的读写锁被中毒，返回 `AppError::InternalServerError`。
pub fn get_all_routes() -> Result<Vec<RouteInfo>, AppError> {
    let registry = ROUTE_REGISTRY
        .read()
        .map_err(|e| AppError::InternalServerError(format!("读取路由注册表失败: {e}")))?;
    Ok(registry.values().cloned().collect())
}
use route_macros::flush_crud_logs;

use crate::config::AppError;

flush_crud_logs!();
