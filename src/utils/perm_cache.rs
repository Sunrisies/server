use crate::config::AppError;
use crate::models::{
    permissions::{self},
    role_permissions,
};
use once_cell::sync::Lazy;
use sea_orm::{DatabaseConnection, EntityTrait};
use std::collections::{HashMap, HashSet};
use tokio::sync::RwLock;

/// 角色 → 权限码集合
pub static ROLE_PERMS: Lazy<RwLock<HashMap<i32, HashSet<String>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

pub async fn load_perm_cache(db: &DatabaseConnection) -> Result<(), AppError> {
    // 1. 角色权限模板
    let rp = role_permissions::Entity::find()
        .find_with_related(permissions::Entity)
        .all(db)
        .await?;
    // let mut role_map = HashMap::new();
    let mut role_map: HashMap<i32, HashSet<String>> = HashMap::new();
    for (rp, perms) in rp {
        let codes: HashSet<_> = perms.into_iter().map(|p| p.code).collect();
        role_map
            .entry(rp.role_id.unwrap()) // 先拿 Entry
            .or_default() // 没有就新建空 Set
            .extend(codes); // 再把本次权限码合并进去
    }
    log::info!("role_map: {:#?}", role_map);
    *ROLE_PERMS.write().await = role_map;

    Ok(())
}
