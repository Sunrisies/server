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

// 自动初始化路由注册表
pub fn init_route_registry() {
    let mut registry = ROUTE_REGISTRY.write().unwrap();

    // inventory::iter() 会返回所有被收集的 RouteInfo 实例
    for route_info in inventory::iter::<RouteInfo> {
        let key = format!("{}:{}", route_info.method.to_uppercase(), route_info.path);
        registry.insert(key, route_info.clone());
    }
}

// 获取路由权限
pub fn get_route_permission(path: &str, method: &str) -> Option<&'static str> {
    let registry = ROUTE_REGISTRY.read().unwrap();
    let key = format!("{}:{}", method.to_uppercase(), path);
    registry.get(&key).map(|r| r.permission)
}

// 获取所有注册的路由（用于调试或生成文档）
pub fn get_all_routes() -> Vec<RouteInfo> {
    let registry = ROUTE_REGISTRY.read().unwrap();
    registry.values().cloned().collect()
}
use route_macros::flush_crud_logs;

flush_crud_logs!();
