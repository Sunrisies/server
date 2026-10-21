use crate::{
    ApiResponse, HttpResult,
    config::AppError,
    dto::{
        category::UpdateCategoryRequest, link::UpdateLinkRequest, tag::CreateTagRequest,
        user::ValidationErrorJson,
    },
    models::{categories, external_links, tags},
};
use actix_web::web;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter,
    Set,
};
use sea_orm::{PaginatorTrait, TransactionTrait};
use validator::Validate;

fn validate<T: Validate>(value: &T) -> Result<(), AppError> {
    value
        .validate()
        .map_err(|e| AppError::ValidationError(ValidationErrorJson::from_validation_errors(&e)))
}

pub async fn update_category(
    db: web::Data<DatabaseConnection>,
    id: web::Path<i32>,
    body: web::Json<UpdateCategoryRequest>,
) -> HttpResult {
    validate(&body.0)?;
    let id = id.into_inner();
    let item = categories::Entity::find_by_id(id)
        .one(db.get_ref())
        .await?
        .ok_or_else(|| AppError::NotFound("分类不存在".into()))?;
    if categories::Entity::find()
        .filter(categories::Column::Name.eq(&body.name))
        .filter(categories::Column::Id.ne(id))
        .one(db.get_ref())
        .await?
        .is_some()
    {
        return Err(AppError::Conflict("分类名称已存在".into()));
    }
    let mut item = item.into_active_model();
    item.name = Set(body.name.clone());
    item.updated_at = Set(chrono::Utc::now());
    Ok(ApiResponse::success(item.update(db.get_ref()).await?, "分类已更新").to_http_response())
}

pub async fn update_tag(
    db: web::Data<DatabaseConnection>,
    id: web::Path<i32>,
    body: web::Json<CreateTagRequest>,
) -> HttpResult {
    validate(&body.0)?;
    let id = id.into_inner();
    let item = tags::Entity::find_by_id(id)
        .one(db.get_ref())
        .await?
        .ok_or_else(|| AppError::NotFound("标签不存在".into()))?;
    if tags::Entity::find()
        .filter(tags::Column::Name.eq(&body.name))
        .filter(tags::Column::Id.ne(id))
        .one(db.get_ref())
        .await?
        .is_some()
    {
        return Err(AppError::Conflict("标签名称已存在".into()));
    }
    let mut item = item.into_active_model();
    item.name = Set(body.name.clone());
    Ok(ApiResponse::success(item.update(db.get_ref()).await?, "标签已更新").to_http_response())
}

pub async fn update_link(
    db: web::Data<DatabaseConnection>,
    id: web::Path<i32>,
    body: web::Json<UpdateLinkRequest>,
) -> HttpResult {
    validate(&body.0)?;
    if body.valid_from.is_some()
        || body.valid_to.is_some()
        || body.is_active.is_some()
        || body.visibility.is_some()
        || body.allowed_roles.is_some()
    {
        return Err(AppError::NotImplemented(
            "链接权限和时效字段尚未支持".into(),
        ));
    }
    let id = id.into_inner();
    let item = external_links::Entity::find_by_id(id)
        .one(db.get_ref())
        .await?
        .ok_or_else(|| AppError::NotFound("链接不存在".into()))?;
    if let Some(name) = &body.name {
        if external_links::Entity::find()
            .filter(external_links::Column::Name.eq(name))
            .filter(external_links::Column::Id.ne(id))
            .one(db.get_ref())
            .await?
            .is_some()
        {
            return Err(AppError::Conflict("链接名称已存在".into()));
        }
    }
    let mut item = item.into_active_model();
    if let Some(name) = &body.name {
        item.name = Set(name.clone());
    }
    if let Some(url) = &body.url {
        item.url = Set(url.clone());
        item.protocol = Set(if url.starts_with("https://") {
            "https"
        } else {
            "http"
        }
        .into());
    }
    if body.description.is_some() {
        item.description = Set(body.description.clone());
    }
    if body.icon_url.is_some() {
        item.icon_url = Set(body.icon_url.clone());
    }
    if let Some(category) = &body.category {
        item.category = Set(category.clone());
    }
    if let Some(tags) = &body.tags {
        item.tags = Set(serde_json::json!(tags));
    }
    item.updated_at = Set(chrono::Utc::now());
    Ok(ApiResponse::success(item.update(db.get_ref()).await?, "链接已更新").to_http_response())
}

pub async fn delete_category(db: web::Data<DatabaseConnection>, id: web::Path<i32>) -> HttpResult {
    let tx = db.begin().await?;
    let id = id.into_inner();
    let item = categories::Entity::find_by_id(id)
        .one(&tx)
        .await?
        .ok_or_else(|| AppError::NotFound("分类不存在".into()))?;
    if crate::models::posts::Entity::find()
        .filter(crate::models::posts::Column::CategoryId.eq(id))
        .count(&tx)
        .await?
        > 0
    {
        return Err(AppError::Conflict(
            "分类仍被文章使用，请先调整文章分类".into(),
        ));
    }
    item.into_active_model().delete(&tx).await?;
    tx.commit().await?;
    Ok(ApiResponse::success((), "分类已删除").to_http_response())
}

pub async fn delete_tag(db: web::Data<DatabaseConnection>, id: web::Path<i32>) -> HttpResult {
    let tx = db.begin().await?;
    let id = id.into_inner();
    let item = tags::Entity::find_by_id(id)
        .one(&tx)
        .await?
        .ok_or_else(|| AppError::NotFound("标签不存在".into()))?;
    crate::models::post_tags::Entity::delete_many()
        .filter(crate::models::post_tags::Column::TagId.eq(id))
        .exec(&tx)
        .await?;
    item.into_active_model().delete(&tx).await?;
    tx.commit().await?;
    Ok(ApiResponse::success((), "标签已删除").to_http_response())
}

#[cfg(test)]
mod auth_crud_tests {
    use super::*;
    use actix_web::{App, cookie::Cookie, test};
    use sea_orm::{ConnectionTrait, Schema};
    #[actix_web::test]
    async fn auth_management_crud_matches_frontend_payloads() {
        let db = crate::services::auth_test::tests::setup_test_db().await;
        let schema = Schema::new(db.get_database_backend());
        for table in [
            schema.create_table_from_entity(categories::Entity),
            schema.create_table_from_entity(tags::Entity),
            schema.create_table_from_entity(external_links::Entity),
        ] {
            db.execute(db.get_database_backend().build(&table))
                .await
                .unwrap();
        }
        let user = crate::services::auth::AuthService::register(
            web::Data::new(db.clone()),
            web::Json(crate::dto::user::RegisterResponse {
                user_name: "crudadmin".into(),
                pass_word: "password123".into(),
            }),
            web::Data::new(crate::SseNotifier::new()),
        )
        .await
        .unwrap();
        let role = crate::models::roles::Entity::find()
            .filter(crate::models::roles::Column::Code.eq("ADMIN"))
            .one(&db)
            .await
            .unwrap()
            .unwrap();
        let mut assignment = crate::models::user_roles::Entity::find()
            .filter(crate::models::user_roles::Column::UserId.eq(user.id))
            .one(&db)
            .await
            .unwrap()
            .unwrap()
            .into_active_model();
        assignment.role_id = Set(Some(role.id));
        assignment.update(&db).await.unwrap();
        let token = crate::utils::jwt::generate_jwt(&db, &user).await.unwrap();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(db.clone()))
                .wrap(crate::middleware::auth::Auth::new(web::Data::new(
                    db.clone(),
                )))
                .service(
                    web::scope("/api")
                        .configure(crate::routes::categories::config_routes)
                        .configure(crate::routes::tags::config_routes)
                        .configure(crate::routes::links::config_routes),
                ),
        )
        .await;
        let response = test::call_service(
            &app,
            test::TestRequest::post()
                .uri("/api/v1/categories")
                .cookie(Cookie::new("access_token", token.clone()))
                .set_json(serde_json::json!({"name":"Original category"}))
                .to_request(),
        )
        .await;
        assert_eq!(response.status(), 200);
        let category: serde_json::Value = test::read_body_json(response).await;
        let response = test::call_service(
            &app,
            test::TestRequest::put()
                .uri(&format!("/api/v1/categories/{}", category["data"]["id"]))
                .cookie(Cookie::new("access_token", token.clone()))
                .set_json(serde_json::json!({"name":"Renamed category"}))
                .to_request(),
        )
        .await;
        assert_eq!(response.status(), 200);
        let updated: serde_json::Value = test::read_body_json(response).await;
        assert_eq!(updated["data"]["name"], "Renamed category");
        assert_eq!(updated["data"]["slug"], category["data"]["slug"]);
        assert_eq!(
            updated["data"]["created_at"],
            category["data"]["created_at"]
        );
        let response = test::call_service(
            &app,
            test::TestRequest::post()
                .uri("/api/v1/tags")
                .cookie(Cookie::new("access_token", token.clone()))
                .set_json(serde_json::json!({"name":"Original tag"}))
                .to_request(),
        )
        .await;
        assert_eq!(response.status(), 200);
        let tag: serde_json::Value = test::read_body_json(response).await;
        let response = test::call_service(
            &app,
            test::TestRequest::put()
                .uri(&format!("/api/v1/tags/{}", tag["data"]["id"]))
                .cookie(Cookie::new("access_token", token.clone()))
                .set_json(serde_json::json!({"name":"Renamed tag"}))
                .to_request(),
        )
        .await;
        assert_eq!(response.status(), 200);
        let tag: serde_json::Value = test::read_body_json(response).await;
        assert_eq!(tag["data"]["name"], "Renamed tag");
        let response = test::call_service(&app, test::TestRequest::post().uri("/api/v1/links").cookie(Cookie::new("access_token",token.clone())).set_json(serde_json::json!({"name":"Example link","url":"https://example.com","icon":"https://example.com/icon.png"})).to_request()).await;
        assert_eq!(response.status(), 200);
        let link: serde_json::Value = test::read_body_json(response).await;
        assert_eq!(link["data"]["icon_url"], "https://example.com/icon.png");
        assert_eq!(link["data"]["category"], "其他");
        let response = test::call_service(&app, test::TestRequest::put().uri(&format!("/api/v1/links/{}",link["data"]["id"])).cookie(Cookie::new("access_token",token.clone())).set_json(serde_json::json!({"name":"Updated link","url":"http://example.com/new","icon_url":"https://example.com/new.png"})).to_request()).await;
        assert_eq!(response.status(), 200);
        let updated: serde_json::Value = test::read_body_json(response).await;
        assert_eq!(updated["data"]["protocol"], "http");
        assert_eq!(updated["data"]["icon_url"], "https://example.com/new.png");
        assert_eq!(updated["data"]["created_at"], link["data"]["created_at"]);
        let missing = test::call_service(
            &app,
            test::TestRequest::put()
                .uri("/api/v1/categories/9999")
                .cookie(Cookie::new("access_token", token))
                .set_json(serde_json::json!({"name":"Missing"}))
                .to_request(),
        )
        .await;
        assert_eq!(missing.status(), 404);
    }
}
