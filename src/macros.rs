#[macro_export]
macro_rules! impl_entity_unique_check {
    ($entity:ident,$model:ident) => {
        impl $entity {
            pub fn find_by_col<C, V>(col: C, val: V) -> Select<Self>
            where
                C: ColumnTrait,
                V: Into<sea_orm::Value>,
            {
                Self::find().filter(col.eq(val))
            }
            pub async fn check_unique(
                db: &sea_orm::DatabaseConnection,
                col: impl sea_orm::ColumnTrait,
                value: impl Into<sea_orm::Value>,
            ) -> Result<Option<$model>, sea_orm::DbErr> {
                Self::find().filter(col.eq(value)).one(db).await
            }
        }
    };
}

#[macro_export]
macro_rules! check_unique_field {
    ($entity:ident, $db:expr, $field:ident, $value:expr) => {
        if let Some(_existing) =
            $entity::Entity::check_unique($db, <$entity::Column>::$field, $value.to_string())
                .await?
        {
            return Err(AppError::DatabaseConnectionError("已存在".to_string()));
        }
    };
}
