use super::openapi::OpenApiGenerator;
use crate::args::{CrudEntityConfig, CrudOperation, CustomQueryType, IdType};
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
    let custom_queries = config.custom_queries.unwrap_or_default();
    let custom_list_fn = config.custom_list_fn;
    let custom_read_fn = config.custom_read_fn;
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

    // 检查是否需要自定义查询
    let use_custom_list = custom_queries.contains(&CustomQueryType::All)
        || custom_queries.contains(&CustomQueryType::List);
    let use_custom_read = custom_queries.contains(&CustomQueryType::All)
        || custom_queries.contains(&CustomQueryType::Read);

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
                    use_custom_read,
                    &custom_read_fn,
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
                list_code = generate_list_code(
                    entity,
                    route_prefix,
                    permission_prefix,
                    // use_custom_list,
                    // &custom_list_fn,
                );
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
    // 记录自定义查询信息
    if use_custom_list || use_custom_read {
        let mut custom_logs = Vec::new();
        if use_custom_list {
            if let Some(ref fn_name) = custom_list_fn {
                custom_logs.push(format!("自定义列表函数: {}", fn_name));
            } else {
                custom_logs.push("自定义列表函数: 使用默认命名".to_string());
            }
        }
        if use_custom_read {
            if let Some(ref fn_name) = custom_read_fn {
                custom_logs.push(format!("自定义详情函数: {}", fn_name));
            } else {
                custom_logs.push("自定义详情函数: 使用默认命名".to_string());
            }
        }

        LOGGER.with(|logger| {
            let mut logger = logger.borrow_mut();
            for log in custom_logs {
                logger.log(&module_name, LogLevel::Info, log);
            }
        });
    }

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
    use_custom: bool,
    custom_fn: &Option<Ident>,
) -> proc_macro2::TokenStream {
    let get_fn = format_ident!("get_{}", entity.to_string().to_lowercase());
    let get_handler = format_ident!("get_{}_handler", entity.to_string().to_lowercase());
    let full_path = format!("{}/{{id}}", route_prefix.value());
    let full_permission = format!("get::{}:read", permission_prefix.value());
    let openapi_doc = openapi_gen.generate_read_doc(id_type_str);
    if use_custom {
        let custom_fn_name = custom_fn.as_ref().unwrap_or(&get_fn);

        quote! {
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
                match #custom_fn_name(db.get_ref(), page, limit).await {
                    Ok(result) => Ok(result),
                    Err(e) => {
                        log::error!("自定义查询失败: {}", e);
                        Ok(ApiResponse::from(AppError::DatabaseConnectionError(
                            "查询失败".to_string(),
                        )).to_http_response())
                    }
                }
            }
        }
    } else {
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
                    Ok(data) => Ok(ApiResponse::success(data,"获取成功").to_http_response()),
                    Err(AppError::NotFound(msg)) => {
                        Ok(ApiResponse::<()>::success_msg(&msg).to_http_response())
                    },
                    _ => todo!()
                }
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

        // 1. 建立分页器
        let paginator = #entity::Entity::find().paginate(db_pool, limit);

        // 2. 并发拿总数 + 当前页数据（Sea-ORM 顺序执行，但代码简洁）
        let total = match paginator.num_items().await {
            Ok(t) => t,
            Err(e) => {
                println!("查询{}总数失败: {}",stringify!(#full_path),e);
                return Ok(ApiResponse::from(AppError::DatabaseConnectionError(
                    "获取失败".to_string(),
                ))
                .to_http_response());
            }
        };

        let data = match paginator.fetch_page(page.saturating_sub(1)).await {
            Ok(list) => list,
            Err(e) => {
                println!("查询{}列表失败: {}",stringify!(#full_path), e);
                return Ok(ApiResponse::from(AppError::DatabaseConnectionError(
                    "获取列表失败".to_string(),
                ))
                .to_http_response());
            }
        };
        log::info!("data:{:?}", data);
        // 3. 组装成前端需要的分页结构
        let resp = PaginatedResp {
            data,
            pagination:Pagination{
                total, // u64 -> usize
                page,
                limit,
            }
        };

        // 4. 统一出口
            Ok(ApiResponse::success(resp, "获取成功").to_http_response())
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
            // 4. 统一出口
            Ok(result)
        }
    }
}
