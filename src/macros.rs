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

// 定义转换宏
#[macro_export]
macro_rules! impl_from_request {
    // 基本用法：字段名相同，直接映射
    ($request:ty => $model:ty {
        $($field:ident),* $(,)?
    }) => {
        impl From<$request> for $model {
            fn from(request: $request) -> Self {
                let mut model = <$model>::default();
                $(
                    model.$field = sea_orm::Set(request.$field);
                )*
                model
            }
        }
    };

}
#[macro_export]
macro_rules! impl_from_request_with_default {
    // 复杂用法：支持字段转换和默认值
    ($request:ty => $model:ty {
        fields: {
            $($field:ident: $transform:expr),* $(,)?
        },
        defaults: {
            $($default_field:ident: $default_value:expr),* $(,)?
        }
    }) => {
        impl $model {
            pub fn from_request(request: $request) -> Self {
                let mut model = <$model>::default();
                $(
                    model.$field = sea_orm::Set($transform);
                )*
                $(
                    model.$default_field = sea_orm::Set($default_value);
                )*
                model
            }
        }
    };
}
