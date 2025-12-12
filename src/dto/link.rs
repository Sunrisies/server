use crate::models::external_links;
use sea_orm::ActiveValue::Set;
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;
use validator::{Validate, ValidationError};
fn validate_protocol(url: &str) -> Result<(), ValidationError> {
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err(ValidationError::new("invalid_protocol"));
    }
    Ok(())
}

#[derive(Deserialize, Serialize, ToSchema, Debug, Validate, Clone)]
pub struct CreateLinkRequest {
    #[validate(length(min = 2, max = 255))]
    pub name: String,
    pub description: Option<String>,
    #[validate(length(min = 10), custom(function = "validate_protocol"))]
    pub url: String,
    pub icon_url: Option<String>,
    #[validate(length(min = 1, max = 100))]
    pub category: String,
    pub tags: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, ToSchema, Debug, Validate, Clone)]
pub struct UpdateLinkRequest {
    #[validate(length(min = 2, max = 255))]
    pub name: Option<String>,
    pub description: Option<String>,
    #[validate(custom(function = "validate_protocol"))]
    pub url: Option<String>,
    pub icon_url: Option<String>,
    #[validate(length(min = 1, max = 100))]
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub valid_from: Option<String>,
    pub valid_to: Option<String>,
    pub is_active: Option<bool>,
    pub visibility: Option<String>,
    pub allowed_roles: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
pub struct LinkFilterQuery {
    pub page: u64,
    pub limit: u64,
    pub category: Option<String>,
    pub q: Option<String>,
    pub tags: Option<Vec<String>>,
}

impl From<CreateLinkRequest> for external_links::ActiveModel {
    fn from(req: CreateLinkRequest) -> Self {
        let protocol = if req.url.starts_with("https://") {
            "https"
        } else {
            "http"
        };
        let tags_json = json!(req.tags.unwrap_or_default());

        external_links::ActiveModel {
            uuid: Set(uuid::Uuid::new_v4().to_string()),
            name: Set(req.name),
            description: Set(req.description),
            url: Set(req.url),
            protocol: Set(protocol.to_string()),
            icon_url: Set(req.icon_url),
            category: Set(req.category),
            tags: Set(tags_json),
            ..Default::default()
        }
    }
}
