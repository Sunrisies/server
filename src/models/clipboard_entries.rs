//! `SeaORM` Entity — 云剪贴板条目

use crate::utils::fmt_beijing;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, ToSchema)]
#[sea_orm(table_name = "clipboard_entries")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(unique)]
    pub uuid: String,
    pub channel_id: i32,
    pub r#type: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub content: Option<String>,
    pub file_url: Option<String>,
    pub file_name: Option<String>,
    pub file_size: Option<i64>,
    pub mime_type: Option<String>,
    pub pinned: bool,
    #[schema(value_type = String, format = DateTime)]
    #[serde(serialize_with = "fmt_beijing")]
    pub created_at: DateTimeUtc,
    #[schema(value_type = String, format = DateTime)]
    #[serde(serialize_with = "fmt_beijing")]
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::clipboard_channels::Entity",
        from = "Column::ChannelId",
        to = "super::clipboard_channels::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Channels,
}

impl Related<super::clipboard_channels::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Channels.def()
    }
}

impl Entity {
    pub fn find_by_uuid(uuid: &str) -> Select<Entity> {
        Self::find().filter(Column::Uuid.eq(uuid))
    }

    pub fn find_by_channel(_db: &DatabaseConnection, channel_id: i32) -> Select<Entity> {
        Self::find().filter(Column::ChannelId.eq(channel_id))
    }
}

impl ActiveModelBehavior for ActiveModel {}
