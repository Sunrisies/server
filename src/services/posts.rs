use crate::config::AppError;
use crate::dto::posts::{
    CreatePostRequest,
    PostResponse,
    UpdatePostRequest,
    // UpdatePostRequest
};
use crate::models::{categories, post_tags, posts, tags, users};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
    TransactionTrait,
};
use uuid::Uuid;

pub struct PostService;

impl PostService {
    /// 创建文章
    pub async fn create_post(
        db: &DatabaseConnection,
        user_id: i32,
        post_data: CreatePostRequest,
    ) -> Result<PostResponse, AppError> {
        let now = Utc::now();
        let published_at = if post_data.status == 1 {
            Some(now)
        } else {
            None
        };

        // 计算文章大小（字符数）
        let size = post_data.content.len() as i32;

        // 开启事务
        let txn = db.begin().await.map_err(|e| {
            log::error!("开启事务失败: {}", e);
            AppError::DatabaseError("服务器内部错误".to_string())
        })?;

        // 创建文章
        let new_post = posts::ActiveModel {
            uuid: Set(Uuid::new_v4().to_string()),
            author_id: Set(user_id),
            category_id: Set(post_data.category_id),
            title: Set(post_data.title),
            summary: Set(post_data.summary),
            content: Set(post_data.content),
            markdowncontent: Set(post_data.markdowncontent),
            cover_image: Set(post_data.cover_image),
            status: Set(post_data.status),
            featured: Set(post_data.featured),
            view_count: Set(0),
            created_at: Set(now),
            updated_at: Set(now),
            published_at: Set(published_at.unwrap_or(now)),
            size: Set(size),
            ..Default::default()
        };

        let created_post = new_post.insert(&txn).await.map_err(|e| {
            log::error!("创建文章失败: {}", e);
            AppError::DatabaseError("创建文章失败".to_string())
        })?;

        // 添加文章标签关联
        for tag_id in post_data.tag_ids {
            let post_tag = post_tags::ActiveModel {
                post_id: Set(created_post.id),
                tag_id: Set(tag_id),
            };

            post_tag.insert(&txn).await.map_err(|e| {
                log::error!("添加文章标签关联失败: {}", e);
                AppError::DatabaseError("添加文章标签关联失败".to_string())
            })?;
        }

        // 提交事务
        txn.commit().await.map_err(|e| {
            log::error!("提交事务失败: {}", e);
            AppError::DatabaseError("服务器内部错误".to_string())
        })?;

        // 查询作者信息
        let author = users::Entity::find_by_id(created_post.author_id)
            .one(db)
            .await
            .map_err(|e| {
                log::error!("查询作者信息失败: {}", e);
                AppError::DatabaseError("查询作者信息失败".to_string())
            })?
            .map(|user| user.user_name)
            .unwrap_or_else(|| "未知作者".to_string());

        // 查询分类信息
        let category = categories::Entity::find_by_id(created_post.category_id)
            .one(db)
            .await
            .map_err(|e| {
                log::error!("查询分类信息失败: {}", e);
                AppError::DatabaseError("查询分类信息失败".to_string())
            })?
            .map(|cat| crate::dto::posts::CategoryResponse {
                id: cat.id,
                name: cat.name,
            });

        // 查询标签信息
        let tag_relations = post_tags::Entity::find()
            .filter(post_tags::Column::PostId.eq(created_post.id))
            .find_also_related(tags::Entity)
            .all(db)
            .await
            .map_err(|e| {
                log::error!("查询标签信息失败: {}", e);
                AppError::DatabaseError("查询标签信息失败".to_string())
            })?;

        let tags: Vec<crate::dto::posts::TagResponse> = tag_relations
            .into_iter()
            .filter_map(|(_, tag_option)| tag_option)
            .map(|tag| crate::dto::posts::TagResponse {
                id: tag.id,
                name: tag.name,
            })
            .collect();

        // 构建响应
        Ok(PostResponse {
            id: created_post.id,
            uuid: created_post.uuid,
            title: created_post.title,
            cover: created_post.cover_image.unwrap_or_default(),
            author,
            publish_time: created_post.published_at,
            update_time: created_post.updated_at,
            views: created_post.view_count,
            is_top: created_post.featured,
            is_publish: created_post.status == 1,
            is_hide: created_post.status == 2,
            description: created_post.summary.unwrap_or_default(),
            size: created_post.size,
            category,
            tags,
        })
    }

    /// 更新文章
    pub async fn update_post(
        db: &DatabaseConnection,
        user_id: i32,
        uuid: &str,
        post_data: UpdatePostRequest,
    ) -> Result<PostResponse, AppError> {
        let now = Utc::now();

        // 查询文章是否存在
        let post = posts::Entity::find_by_uuid(uuid)
            .one(db)
            .await
            .map_err(|e| {
                log::error!("查询文章失败: {}", e);
                AppError::DatabaseError("服务器内部错误".to_string())
            })?
            .ok_or_else(|| AppError::NotFound("文章不存在".to_string()))?;

        // 检查权限（只有作者可以编辑自己的文章）
        if post.author_id != user_id {
            return Err(AppError::Unauthorized("没有权限编辑此文章".to_string()));
        }

        // 开启事务
        let txn = db.begin().await.map_err(|e| {
            log::error!("开启事务失败: {}", e);
            AppError::DatabaseError("服务器内部错误".to_string())
        })?;

        // 更新文章
        let mut active_post: posts::ActiveModel = post.into();

        // 只更新提供的字段
        if let Some(title) = post_data.title {
            active_post.title = Set(title);
        }
        if let Some(summary) = post_data.summary {
            active_post.summary = Set(Some(summary));
        }
        if let Some(content) = post_data.content {
            active_post.content = Set(content.clone());
            active_post.size = Set(content.len() as i32);
        }
        if let Some(markdowncontent) = post_data.markdowncontent {
            active_post.markdowncontent = Set(markdowncontent);
        }
        if let Some(cover_image) = post_data.cover_image {
            active_post.cover_image = Set(Some(cover_image));
        }
        if let Some(category_id) = post_data.category_id {
            active_post.category_id = Set(category_id);
        }
        if let Some(status) = post_data.status {
            active_post.status = Set(status);
            // // 如果状态从非发布变为发布，设置发布时间
            // if post.status != 1 && status == 1 {
            //     active_post.published_at = Set(now);
            // }
        }
        if let Some(featured) = post_data.featured {
            active_post.featured = Set(featured);
        }

        active_post.updated_at = Set(now);

        let updated_post = active_post.update(&txn).await.map_err(|e| {
            log::error!("更新文章失败: {}", e);
            AppError::DatabaseError("更新文章失败".to_string())
        })?;

        // 如果提供了标签，更新文章标签关联
        if let Some(tag_ids) = post_data.tag_ids {
            // 先删除所有现有标签关联
            post_tags::Entity::delete_many()
                .filter(post_tags::Column::PostId.eq(updated_post.id))
                .exec(&txn)
                .await
                .map_err(|e| {
                    log::error!("删除文章标签关联失败: {}", e);
                    AppError::DatabaseError("更新文章标签关联失败".to_string())
                })?;

            // 添加新的标签关联
            for tag_id in tag_ids {
                let post_tag = post_tags::ActiveModel {
                    post_id: Set(updated_post.id),
                    tag_id: Set(tag_id),
                };

                post_tag.insert(&txn).await.map_err(|e| {
                    log::error!("添加文章标签关联失败: {}", e);
                    AppError::DatabaseError("更新文章标签关联失败".to_string())
                })?;
            }
        }

        // 提交事务
        txn.commit().await.map_err(|e| {
            log::error!("提交事务失败: {}", e);
            AppError::DatabaseError("服务器内部错误".to_string())
        })?;

        // 查询作者信息
        let author = users::Entity::find_by_id(updated_post.author_id)
            .one(db)
            .await
            .map_err(|e| {
                log::error!("查询作者信息失败: {}", e);
                AppError::DatabaseError("查询作者信息失败".to_string())
            })?
            .map(|user| user.user_name)
            .unwrap_or_else(|| "未知作者".to_string());

        // 查询分类信息
        let category = categories::Entity::find_by_id(updated_post.category_id)
            .one(db)
            .await
            .map_err(|e| {
                log::error!("查询分类信息失败: {}", e);
                AppError::DatabaseError("查询分类信息失败".to_string())
            })?
            .map(|cat| crate::dto::posts::CategoryResponse {
                id: cat.id,
                name: cat.name,
            });

        // 查询标签信息
        let tag_relations = post_tags::Entity::find()
            .filter(post_tags::Column::PostId.eq(updated_post.id))
            .find_also_related(tags::Entity)
            .all(db)
            .await
            .map_err(|e| {
                log::error!("查询标签信息失败: {}", e);
                AppError::DatabaseError("查询标签信息失败".to_string())
            })?;

        let tags: Vec<crate::dto::posts::TagResponse> = tag_relations
            .into_iter()
            .filter_map(|(_, tag_option)| tag_option)
            .map(|tag| crate::dto::posts::TagResponse {
                id: tag.id,
                name: tag.name,
            })
            .collect();

        // 构建响应
        Ok(PostResponse {
            id: updated_post.id,
            uuid: updated_post.uuid,
            title: updated_post.title,
            cover: updated_post.cover_image.unwrap_or_default(),
            author,
            publish_time: updated_post.published_at,
            update_time: updated_post.updated_at,
            views: updated_post.view_count,
            is_top: updated_post.featured,
            is_publish: updated_post.status == 1,
            is_hide: updated_post.status == 2,
            description: updated_post.summary.unwrap_or_default(),
            size: updated_post.size,
            category,
            tags,
        })
    }

    /// 删除文章
    pub async fn delete_post(
        db: &DatabaseConnection,
        user_id: i32,
        uuid: &str,
    ) -> Result<(), AppError> {
        // 查询文章是否存在
        let post = posts::Entity::find_by_uuid(uuid)
            .one(db)
            .await
            .map_err(|e| {
                log::error!("查询文章失败: {}", e);
                AppError::DatabaseError("服务器内部错误".to_string())
            })?
            .ok_or_else(|| AppError::NotFound("文章不存在".to_string()))?;

        // 检查权限（只有作者可以删除自己的文章）
        if post.author_id != user_id {
            return Err(AppError::Unauthorized("没有权限删除此文章".to_string()));
        }

        // 开启事务
        let txn = db.begin().await.map_err(|e| {
            log::error!("开启事务失败: {}", e);
            AppError::DatabaseError("服务器内部错误".to_string())
        })?;

        // 删除文章标签关联
        post_tags::Entity::delete_many()
            .filter(post_tags::Column::PostId.eq(post.id))
            .exec(&txn)
            .await
            .map_err(|e| {
                log::error!("删除文章标签关联失败: {}", e);
                AppError::DatabaseError("删除文章失败".to_string())
            })?;

        // 删除文章
        posts::Entity::delete_by_id(post.id)
            .exec(&txn)
            .await
            .map_err(|e| {
                log::error!("删除文章失败: {}", e);
                AppError::DatabaseError("删除文章失败".to_string())
            })?;

        // 提交事务
        txn.commit().await.map_err(|e| {
            log::error!("提交事务失败: {}", e);
            AppError::DatabaseError("服务器内部错误".to_string())
        })?;

        Ok(())
    }
}
