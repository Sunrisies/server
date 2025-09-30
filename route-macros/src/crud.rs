use super::openapi::OpenApiGenerator;
use crate::args::{CrudEntityConfig, CrudOperation, IdType};
use crate::log::{LOGGER, LogLevel};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, LitStr, parse_macro_input};

/// 简化版 CRUD 宏 - 为实体快速生成标准 CRUD 操作
pub fn crud_entity(input: TokenStream) -> TokenStream {
    let config = parse_macro_input!(input as CrudEntityConfig);

    let entity = &config.entity;
    let route_prefix = &config.route_prefix;
    let permission_prefix = &config.permission_prefix;
    let id_type = config.id_type.unwrap_or(IdType::Uuid);

    let operations = config.operations.unwrap_or_else(|| {
        vec![
            CrudOperation::Create,
            CrudOperation::Read,
            CrudOperation::Delete,
            CrudOperation::List,
        ]
    });

    let (fn_arg, call_expr, path_param_type, id_type_str) = match id_type {
        IdType::Uuid => (
            quote! { id: String },
            quote! { #entity::Entity::find_by_uuid(&id) },
            quote! { String },
            "uuid",
        ),
        IdType::Custom(_) => (
            quote! { id: i32 },
            quote! { #entity::Entity::find_by_id(id) },
            quote! { i32 },
            "id",
        ),
    };

    let mod_name = format_ident!("{}_routes", entity.to_string().to_lowercase());

    // 创建 OpenAPI 生成器
    let openapi_gen = OpenApiGenerator::new(entity, route_prefix);

    let mut create_code = quote! {};
    let mut read_code = quote! {};
    let mut delete_code = quote! {};
    let mut list_code = quote! {};

    // 记录操作信息
    let module_name = entity.to_string();
    let mut operation_logs = Vec::new();

    for operation in &operations {
        match operation {
            CrudOperation::Create => {
                create_code = generate_create_code(
                    entity,
                    route_prefix,
                    permission_prefix,
                    &config.create_request_type,
                );
                operation_logs.push(format!(
                    "创建操作: create_{}_handler",
                    entity.to_string().to_lowercase()
                ));
            }
            CrudOperation::Read => {
                read_code = generate_read_code(
                    entity,
                    route_prefix,
                    permission_prefix,
                    &path_param_type,
                    &fn_arg,
                    &call_expr,
                    &openapi_gen,
                    id_type_str,
                );
                operation_logs.push(format!(
                    "读取操作: get_{}_handler",
                    entity.to_string().to_lowercase()
                ));
            }
            CrudOperation::Delete => {
                delete_code = generate_delete_code(
                    entity,
                    route_prefix,
                    permission_prefix,
                    &path_param_type,
                    &fn_arg,
                    &call_expr,
                );
                operation_logs.push(format!(
                    "删除操作: delete_{}_handler",
                    entity.to_string().to_lowercase()
                ));
            }
            CrudOperation::List => {
                list_code = generate_list_code(entity, route_prefix, permission_prefix);
                operation_logs.push(format!(
                    "列表操作: get_{}_all_handler",
                    entity.to_string().to_lowercase()
                ));
            }
        }
    }

    // 记录到日志
    LOGGER.with(|logger| {
        let mut logger = logger.borrow_mut();
        logger.log(
            &module_name,
            LogLevel::Info,
            format!(
                "实体: {}, 路由前缀: {}, 权限前缀: {}",
                entity,
                route_prefix.value(),
                permission_prefix.value()
            ),
        );

        logger.log(
            &module_name,
            LogLevel::Success,
            format!("ID类型: {:?}, 操作数量: {}", id_type, operations.len()),
        );

        for op_log in operation_logs {
            logger.log(&module_name, LogLevel::Debug, op_log);
        }

        logger.log(
            &module_name,
            LogLevel::Info,
            format!("生成模块: {}", mod_name),
        );
    });

    let output = quote! {
        pub mod #mod_name {
            use super::*;
            #create_code
            #read_code
            #delete_code
            #list_code
        }
    };

    output.into()
}

fn generate_read_code(
    entity: &Ident,
    route_prefix: &LitStr,
    permission_prefix: &LitStr,
    path_param_type: &proc_macro2::TokenStream,
    fn_arg: &proc_macro2::TokenStream,
    call_expr: &proc_macro2::TokenStream,
    openapi_gen: &OpenApiGenerator,
    id_type_str: &str,
) -> proc_macro2::TokenStream {
    let get_fn = format_ident!("get_{}", entity.to_string().to_lowercase());
    let get_handler = format_ident!("get_{}_handler", entity.to_string().to_lowercase());
    let full_path = format!("{}/{{id}}", route_prefix.value());
    let full_permission = format!("get::{}:read", permission_prefix.value());
    let openapi_doc = openapi_gen.generate_read_doc(id_type_str);

    quote! {

        /// 获取实体
        pub async fn #get_fn(
            db: &DatabaseConnection,
             #fn_arg,
        ) -> Result<#entity::Model, AppError> {
            #call_expr
                .one(db)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?
                .ok_or_else(|| AppError::NotFound(format!("{} not found", id)))
        }

        #openapi_doc
        #[crate::route_permission(
            path = #full_path,
            method = "get",
            permission = #full_permission
        )]
        pub async fn #get_handler(
            db: web::Data<DatabaseConnection>,
            path: web::Path<#path_param_type>,
        ) -> HttpResult {
            let id = path.into_inner();
            match #get_fn(db.get_ref(), id).await {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(AppError::NotFound(msg)) => {
                    Ok(ApiResponse::<()>::success_msg(&msg).to_http_response())
                },
                _ => todo!()
            }
        }
    }
}

fn generate_create_code(
    entity: &Ident,
    route_prefix: &LitStr,
    permission_prefix: &LitStr,
    create_request_type: &Option<Ident>,
) -> proc_macro2::TokenStream {
    let create_fn = format_ident!("create_{}", entity.to_string().to_lowercase());
    let create_handler = format_ident!("create_{}_handler", entity.to_string().to_lowercase());
    let full_path = format!("{}", route_prefix.value());
    let full_permission = format!("{}:create", permission_prefix.value());

    let create_request_type = match create_request_type {
        Some(ident) => ident,
        None => {
            return syn::Error::new_spanned(
                entity,
                "create_request_type is required for Create operation",
            )
            .to_compile_error()
            .into();
        }
    };

    quote! {
       pub async fn #create_fn(
            db: &DatabaseConnection,
            data: #create_request_type,
        ) ->Result<#entity::Model, AppError> {
            let active_model = #entity::ActiveModel::from(data);

            let model = active_model.insert(db).await.map_err(|e| {
                println!("添加分类失败: {}", e);
                AppError::DatabaseConnectionError(db_err_map(e).to_owned())
            })?;

            Ok(model)
        }

        #[crate::route_permission(
            path = #full_path,
            method = "post",
            permission = #full_permission
        )]
        pub async fn #create_handler(
            db: web::Data<DatabaseConnection>,
            data: web::Json<#create_request_type>,
        ) -> HttpResult {
            log::info!("Creating new {}", stringify!(#entity));
            match #create_fn(db.get_ref(), data.into_inner()).await {
                Ok(category) => Ok(ApiResponse::success(category, "添加成功").to_http_response()),
                Err(AppError::DatabaseConnectionError(msg)) => {
                    Ok(ApiResponse::<()>::success_msg(&msg).to_http_response())
                }
                Err(e) => {
                    Ok(ApiResponse::from(e).to_http_response())
                }
            }
        }
    }
}

fn generate_delete_code(
    entity: &Ident,
    route_prefix: &LitStr,
    permission_prefix: &LitStr,
    path_param_type: &proc_macro2::TokenStream,
    fn_arg: &proc_macro2::TokenStream,
    call_expr: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let delete_fn = format_ident!("delete_{}", entity.to_string().to_lowercase());
    let delete_handler = format_ident!("delete_{}_handler", entity.to_string().to_lowercase());
    let full_path = format!("{}/{{id}}", route_prefix.value());
    let full_permission = format!("{}:delete:id", permission_prefix.value());

    quote! {
        pub async fn #delete_fn(
            db: &DatabaseConnection,
            #fn_arg,
        ) ->HttpResult {
            let entity = #call_expr.one(db)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?
                .ok_or_else(|| AppError::NotFound(format!("{} not found", id)))?;
            match entity.delete(db).await {
                Ok(_res) => Ok(ApiResponse::<()>::success_msg("删除成功").to_http_response()),
                Err(e) => {
                    println!("删除失败: {}", e);
                    Ok(
                        ApiResponse::from(AppError::DatabaseConnectionError(db_err_map(e).to_owned()))
                            .to_http_response(),
                    )
                }
            }
        }

        #[crate::route_permission(
            path = #full_path,
            method = "delete",
            permission = #full_permission
        )]
        pub async fn #delete_handler(
            db: web::Data<DatabaseConnection>,
            id: web::Path<#path_param_type>,
        ) -> HttpResult {
            let result = #delete_fn(db.get_ref(), id.into_inner()).await?;
            Ok(result)
        }
    }
}

fn generate_list_code(
    entity: &Ident,
    route_prefix: &LitStr,
    permission_prefix: &LitStr,
) -> proc_macro2::TokenStream {
    let get_fn = format_ident!("get_{}_all", entity.to_string().to_lowercase());
    let get_handler = format_ident!("get_{}_all_handler", entity.to_string().to_lowercase());
    let full_path = format!("{}", route_prefix.value());
    let full_permission = format!("get::{}:read::list", permission_prefix.value());

    quote! {
        pub async fn #get_fn(
            db_pool: &DatabaseConnection,
            page: u64,
            limit: u64,
        ) -> Result<HttpResponse,AppError> {
            match #entity::Entity::find()
                    .limit(limit)
                    .offset((page - 1) * limit)
                    .all(db_pool)
                    .await {
                    Ok(data) => Ok(HttpResponse::Ok().json(data)),
                    Err(e) => {
                        println!("Database query error: {}", e);
                        Err(AppError::DatabaseConnectionError(e.to_string()))
                    }
            }
        }
        #[crate::route_permission(
            path = #full_path,
            method = "get",
            permission = #full_permission
        )]
        pub async fn #get_handler(
            db: web::Data<DatabaseConnection>,
            query: web::Query<PaginationQuery>,
        ) -> HttpResult {
            let PaginationQuery { page, limit } = query.into_inner();
            let result =  #get_fn(db.as_ref(), page, limit).await?;
            Ok(result)
        }
    }
}
