// // src/services/category.rs
// // use crate::entities::category;
// use crate::models::categories::{self, ActiveModel, Column, Entity};
// use sea_orm::{DatabaseConnection, DeleteResult, entity::*, error::DbErr};

// pub struct CategoryService;

// impl CategoryService {
//     // 创建新分类
//     pub async fn create(
//         db: &DatabaseConnection,
//         form_data: categories::Model,
//     ) -> Result<categories::Model, DbErr> {
//         let active_model = categories::ActiveModel {
//             name: Set(form_data.name),
//             slug: Set(form_data.slug),
//             description: Set(form_data.description),
//             ..Default::default()
//         };
//         active_model.insert(db).await
//     }

//     // 根据ID查找分类
//     pub async fn find_by_id(
//         db: &DatabaseConnection,
//         id: i32,
//     ) -> Result<Option<categories::Model>, DbErr> {
//         categories::Entity::find_by_id(id).one(db).await
//     }

//     // 获取所有分类（可分页）
//     pub async fn find_all(db: &DatabaseConnection) -> Result<Vec<categories::Model>, DbErr> {
//         categories::Entity::find().all(db).await
//     }

//     // 根据slug查找分类
//     // pub async fn find_by_slug(
//     //     db: &DatabaseConnection,
//     //     slug: &str,
//     // ) -> Result<Option<categories::Model>, DbErr> {
//     //     categories::Entity::find()
//     //         .filter(Column::Slug.eq(slug))
//     //         .one(db)
//     //         .await
//     // }

//     // 更新分类
//     // pub async fn update(
//     //     db: &DatabaseConnection,
//     //     id: i32,
//     //     form_data: categories::Model,
//     // ) -> Result<categories::Model, DbErr> {
//     //     let categories: categories::ActiveModel = categories::Entity::find_by_id(id)
//     //         .one(db)
//     //         .await?
//     //         .ok_or(DbErr::Custom("Cannot find categories.".to_owned()))
//     //         .map(Into::into)?;

//     //     ActiveModel {
//     //         id: category.id,
//     //         name: Set(form_data.name),
//     //         slug: Set(form_data.slug),
//     //         description: Set(form_data.description),
//     //         updated_at: Set(chrono::Utc::now().naive_utc()),
//     //     }
//     //     .update(db)
//     //     .await
//     // }

//     // 删除分类
//     pub async fn delete(db: &DatabaseConnection, id: i32) -> Result<DeleteResult, DbErr> {
//         let category: categories::ActiveModel = category::Entity::find_by_id(id)
//             .one(db)
//             .await?
//             .ok_or(DbErr::Custom("Cannot find category.".to_owned()))
//             .map(Into::into)?;

//         category.delete(db).await
//     }
// }
