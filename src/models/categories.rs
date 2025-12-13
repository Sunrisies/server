use crate::{impl_entity_unique_check, utils::fmt_beijing};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, ToSchema)]
#[sea_orm(table_name = "categories")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    #[sea_orm(unique)]
    pub slug: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub description: Option<String>,
    #[schema(value_type = String, format = DateTime)]
    #[serde(serialize_with = "fmt_beijing")]
    pub created_at: DateTimeUtc,

    #[schema(value_type = String, format = DateTime)]
    #[serde(serialize_with = "fmt_beijing")]
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation, Serialize, Deserialize)]
pub enum Relation {
    #[sea_orm(has_many = "super::posts::Entity")]
    Posts,
}

impl Related<super::posts::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Posts.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
// impl_entity_unique_check!(Model, name, name);
impl_entity_unique_check!(Entity, Model);
