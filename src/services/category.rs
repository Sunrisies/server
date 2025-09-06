use crate::{
    ApiResponse, HttpResult,
    config::AppError,
    dto::PaginatedResp,
    models::categories::{self},
    utils::db_err_map,
};
use sea_orm::{DatabaseConnection, PaginatorTrait, entity::*};

pub struct CategoryService;

impl CategoryService {
    // 创建新分类
    pub async fn create(
        db: &DatabaseConnection,
        form_data: categories::Model,
    ) -> Result<categories::Model, AppError> {
        let active_model = categories::ActiveModel {
            name: Set(form_data.name),
            slug: Set(form_data.slug),
            description: Set(form_data.description),
            ..Default::default()
        };
        let model = active_model.insert(db).await.map_err(|e| {
            println!("添加分类失败: {}", e);
            AppError::DatabaseConnectionError(db_err_map(e).to_owned())
        })?;

        Ok(model)
    }

    // 根据ID查找分类
    pub async fn find_by_id(
        db: &DatabaseConnection,
        id: i32,
    ) -> Result<categories::Model, AppError> {
        categories::Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| AppError::DatabaseConnectionError(format!("获取分类失败: {}", e)))?
            .ok_or_else(|| AppError::NotFound("分类不存在".to_string()))
    }

    // 获取所有分类（可分页）
    pub async fn find_all(db: &DatabaseConnection, page: u64, limit: u64) -> HttpResult {
        // 1. 建立分页器
        let paginator = categories::Entity::find().paginate(db, limit);

        // 2. 并发拿总数 + 当前页数据（Sea-ORM 顺序执行，但代码简洁）
        let total = match paginator.num_items().await {
            Ok(t) => t,
            Err(e) => {
                println!("查询分类总数失败: {}", e);
                return Ok(ApiResponse::from(AppError::DatabaseConnectionError(
                    "获取总数失败".to_string(),
                ))
                .to_http_response());
            }
        };

        let data = match paginator.fetch_page(page.saturating_sub(1)).await {
            Ok(list) => list,
            Err(e) => {
                println!("查询分类列表失败: {}", e);
                return Ok(ApiResponse::from(AppError::DatabaseConnectionError(
                    "获取列表失败".to_string(),
                ))
                .to_http_response());
            }
        };

        // 3. 组装成前端需要的分页结构
        let resp = PaginatedResp {
            data,
            total, // u64 -> usize
            page,
            limit,
        };

        // 4. 统一出口
        Ok(ApiResponse::success(resp, "获取成功").to_http_response())
    }

    // 根据slug查找分类
    // pub async fn find_by_slug(
    //     db: &DatabaseConnection,
    //     slug: &str,
    // ) -> Result<Option<categories::Model>, DbErr> {
    //     categories::Entity::find()
    //         .filter(Column::Slug.eq(slug))
    //         .one(db)
    //         .await
    // }

    // 更新分类
    // pub async fn update(
    //     db: &DatabaseConnection,
    //     id: i32,
    //     form_data: categories::Model,
    // ) -> Result<categories::Model, DbErr> {
    //     let categories: categories::ActiveModel = categories::Entity::find_by_id(id)
    //         .one(db)
    //         .await?
    //         .ok_or(DbErr::Custom("Cannot find categories.".to_owned()))
    //         .map(Into::into)?;

    //     ActiveModel {
    //         id: category.id,
    //         name: Set(form_data.name),
    //         slug: Set(form_data.slug),
    //         description: Set(form_data.description),
    //         updated_at: Set(chrono::Utc::now().naive_utc()),
    //     }
    //     .update(db)
    //     .await
    // }

    // 删除分类
    pub async fn delete(category: &categories::Model, db: &DatabaseConnection) -> HttpResult {
        match category.clone().delete(db).await {
            Ok(_res) => Ok(ApiResponse::<()>::success_msg("分类已删除").to_http_response()),
            Err(e) => {
                println!("删除分类失败: {}", e);
                Ok(
                    ApiResponse::from(AppError::DatabaseConnectionError(db_err_map(e).to_owned()))
                        .to_http_response(),
                )
            }
        }
    }
}
