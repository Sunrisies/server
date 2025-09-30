use crate::utils::fmt_beijing;
use sea_orm::{DeleteResult, entity::prelude::*};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
#[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i32, // 自增id
    #[sea_orm(unique)]
    pub uuid: String, // 用户UUID
    #[sea_orm(unique)]
    pub user_name: String, // 用户名
    #[serde(skip_serializing)] // ✅ 通用、最简单 跳过序列化
    pub pass_word: String, // 密码
    #[sea_orm(unique)]
    pub email: Option<String>, // 邮箱
    pub image: Option<String>, // 头像
    #[sea_orm(unique)]
    pub phone: Option<String>, // 手机号
    pub binding: Option<String>, // authentication绑定
    #[sea_orm(default_value_t = DateTimeUtc::default())]
    #[serde(serialize_with = "fmt_beijing")]
    pub created_at: DateTimeUtc,
    #[sea_orm(default_value_t = DateTimeUtc::default())]
    #[serde(serialize_with = "fmt_beijing")]
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::posts::Entity")]
    Posts,
    #[sea_orm(has_many = "super::user_permissions::Entity")]
    UserPermissions,
    #[sea_orm(has_many = "super::user_roles::Entity")]
    UserRoles,
}

impl Related<super::posts::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Posts.def()
    }
}

impl Entity {
    /// 按任意唯一列查询（编译期已知列 & 值类型）
    pub fn find_by_col<C, V>(col: C, val: V) -> Select<Self>
    where
        C: ColumnTrait,
        V: Into<sea_orm::Value>,
    {
        Self::find().filter(col.eq(val))
    }

    // 添加按UUID查询的方法
    pub fn find_by_uuid(uuid: &str) -> Select<Entity> {
        Self::find().filter(Column::Uuid.eq(uuid))
    }

    // 添加使用UUID删除的方法
    pub async fn delete_by_uuid(
        db: &DatabaseConnection,
        uuid: &str,
    ) -> Result<DeleteResult, sea_orm::DbErr> {
        let result = Self::delete_many()
            .filter(Column::Uuid.eq(uuid))
            .exec(db)
            .await?;
        Ok(result)
    }
    // 根据用户名称查询
    pub fn find_by_name(user_name: &str) -> Select<Entity> {
        Self::find().filter(Column::UserName.eq(user_name))
    }
}

impl From<Model> for JsonValue {
    fn from(model: Model) -> JsonValue {
        serde_json::to_value(model).unwrap()
    }
}

impl TryFrom<JsonValue> for Model {
    type Error = serde_json::Error;

    fn try_from(value: JsonValue) -> Result<Self, Self::Error> {
        serde_json::from_value(value)
    }
}

impl ActiveModelBehavior for ActiveModel {}
