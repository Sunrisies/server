use super::openapi::OpenApiGenerator;
use crate::args::{CrudEntityConfig, CrudOperation, CustomQueryType, IdType};

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, LitStr, parse_macro_input};

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
    let openapi_summary = config
        .openapi_summary
        .unwrap_or_else(|| LitStr::new(&format!("{} CRUD operations", entity), entity.span()));
    let error_type = config
        .error_type
        .as_ref()
        .map(|t| quote! { #t })
        .unwrap_or_else(|| quote! { crate::AppError });

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
    let mut create_code = quote! {};
    let mut read_code = quote! {};
    let update_code = quote! {};
    let mut delete_code = quote! {};
    let mut list_code = quote! {};
    let mut operation_logs = Vec::new();
    let _use_custom_list = custom_queries.contains(&CustomQueryType::All)
        || custom_queries.contains(&CustomQueryType::List);
    let use_custom_read = custom_queries.contains(&CustomQueryType::All)
        || custom_queries.contains(&CustomQueryType::Read);

    for operation in &operations {
        match operation {
            CrudOperation::Create => {
                let og = OpenApiGenerator::new(
                    entity,
                    route_prefix,
                    &openapi_summary,
                    config.openapi_create.as_ref(),
                );
                create_code = generate_create_code(
                    entity,
                    route_prefix,
                    permission_prefix,
                    &config.create_request_type,
                    &config.unique_field,
                    &error_type,
                    &og,
                );
                operation_logs.push(format!("创建操作: create_{}_handler", entity));
            }
            CrudOperation::Read => {
                let og = OpenApiGenerator::new(
                    entity,
                    route_prefix,
                    &openapi_summary,
                    config.openapi_read.as_ref(),
                );
                read_code = generate_read_code(
                    entity,
                    route_prefix,
                    permission_prefix,
                    &path_param_type,
                    &fn_arg,
                    &call_expr,
                    &og,
                    id_type_str,
                    use_custom_read,
                    &custom_read_fn,
                    &error_type,
                );
                operation_logs.push(format!("读取操作: get_{}_handler", entity));
            }
            CrudOperation::Update => {
                operation_logs.push(format!("更新操作: update_{}_handler（待实现）", entity));
            }
            CrudOperation::Delete => {
                let og = OpenApiGenerator::new(
                    entity,
                    route_prefix,
                    &openapi_summary,
                    config.openapi_delete.as_ref(),
                );
                delete_code = generate_delete_code(
                    entity,
                    route_prefix,
                    permission_prefix,
                    &path_param_type,
                    &fn_arg,
                    &call_expr,
                    &og,
                    id_type_str,
                    &error_type,
                );
                operation_logs.push(format!("删除操作: delete_{}_handler", entity));
            }
            CrudOperation::List => {
                let og = OpenApiGenerator::new(
                    entity,
                    route_prefix,
                    &openapi_summary,
                    config.openapi_list.as_ref(),
                );
                list_code =
                    generate_list_code(entity, route_prefix, permission_prefix, &og, &error_type);
                operation_logs.push(format!("列表操作: get_{}_all_handler", entity));
            }
        }
    }

    eprintln!(
        "[route-macros] {}: {} 路由, {} 操作",
        entity,
        route_prefix.value(),
        operations.len()
    );
    for op in &operation_logs {
        eprintln!("[route-macros]   └─ {}", op);
    }
    if let Some(ref f) = custom_list_fn {
        eprintln!("[route-macros]   └─ 自定义列表: {}", f);
    }
    if let Some(ref f) = custom_read_fn {
        eprintln!("[route-macros]   └─ 自定义详情: {}", f);
    }

    quote! {
        pub mod #mod_name { use super::*; #create_code #read_code #update_code #delete_code #list_code }
    }.into()
}

#[allow(clippy::too_many_arguments)]
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
    error_type: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let get_fn = format_ident!("get_{}", entity);
    let handler = format_ident!("get_{}_handler", entity);
    let full_path = format!("{}/{{id}}", route_prefix.value());
    let perm = format!("get::{}:read", permission_prefix.value());
    let doc = openapi_gen.generate_read_doc(id_type_str);

    if use_custom {
        let f = custom_fn.as_ref().unwrap_or(&get_fn);
        quote! {
            #[crate::route_permission(path = #full_path, method = "get", permission = #perm)]
            pub async fn #handler(db: web::Data<DatabaseConnection>, path: web::Path<#path_param_type>,
            ) -> HttpResult<#error_type> {
                match #f(db.get_ref(), path.into_inner()).await {
                    Ok(r) => Ok(r),
                    Err(e) => { log::error!("自定义查询失败: {}", e);
                        Err(#error_type::DatabaseConnectionError("查询失败".into())) }
                }
            }
        }
    } else {
        quote! {
            pub async fn #get_fn(db: &DatabaseConnection, #fn_arg) -> Result<#entity::Model, #error_type> {
                #call_expr.one(db).await
                    .map_err(|e| #error_type::DatabaseError(e.to_string()))?
                    .ok_or_else(|| #error_type::NotFound(format!("{} not found", id)))
            }
            #doc
            #[crate::route_permission(path = #full_path, method = "get", permission = #perm)]
            pub async fn #handler(db: web::Data<DatabaseConnection>, path: web::Path<#path_param_type>,
            ) -> HttpResult<#error_type> {
                match #get_fn(db.get_ref(), path.into_inner()).await {
                    Ok(data) => Ok(ApiResponse::success(data, "获取成功").to_http_response()),
                    Err(e) => Err(e),
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn generate_create_code(
    entity: &Ident,
    route_prefix: &LitStr,
    permission_prefix: &LitStr,
    create_request_type: &Option<Ident>,
    unique_field: &Option<Ident>,
    error_type: &proc_macro2::TokenStream,
    openapi_gen: &OpenApiGenerator,
) -> proc_macro2::TokenStream {
    let create_fn = format_ident!("create_{}", entity);
    let handler = format_ident!("create_{}_handler", entity);
    let full_path = route_prefix.value().to_string();
    let perm = format!("{}:create", permission_prefix.value());
    let req_ty = match create_request_type {
        Some(t) => t,
        None => {
            return syn::Error::new_spanned(entity, "create_request_type required")
                .to_compile_error();
        }
    };
    let doc = openapi_gen.generate_create_doc(req_ty);
    let unique_check = if let Some(field) = unique_field {
        let f = Ident::new(&field.to_string().to_lowercase(), field.span());
        quote! {
            if let Some(_) = #entity::Entity::check_unique(db, <#entity::Column>::#field, data.#f.to_string()).await? {
                return Err(#error_type::DatabaseConnectionError("已存在".into()));
            }
        }
    } else {
        quote! {}
    };

    quote! {
        pub async fn #create_fn(db: &DatabaseConnection, data: #req_ty) -> Result<#entity::Model, #error_type> {
            if let Err(e) = data.validate() {
                eprintln!("Validation errors: {:?}", e);
                return Err(#error_type::ValidationError(ValidationErrorJson::from_validation_errors(&e)));
            }
            #unique_check
            #entity::ActiveModel::from(data).insert(db).await
                .map_err(|e| { eprintln!("创建失败: {}", e); #error_type::DatabaseConnectionError(db_err_map(e).into()) })
        }
        #doc
        #[crate::route_permission(path = #full_path, method = "post", permission = #perm)]
        pub async fn #handler(db: web::Data<DatabaseConnection>, data: web::Json<#req_ty>,
        ) -> HttpResult<#error_type> {
            log::info!("Creating new {}", stringify!(#entity));
            match #create_fn(db.get_ref(), data.into_inner()).await {
                Ok(r) => Ok(ApiResponse::success(r, "添加成功").to_http_response()),
                Err(e) => Err(e),
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn generate_delete_code(
    entity: &Ident,
    route_prefix: &LitStr,
    permission_prefix: &LitStr,
    path_param_type: &proc_macro2::TokenStream,
    fn_arg: &proc_macro2::TokenStream,
    call_expr: &proc_macro2::TokenStream,
    openapi_gen: &OpenApiGenerator,
    id_type_str: &str,
    error_type: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let delete_fn = format_ident!("delete_{}", entity);
    let handler = format_ident!("delete_{}_handler", entity);
    let full_path = format!("{}/{{id}}", route_prefix.value());
    let perm = format!("{}:delete:id", permission_prefix.value());
    let doc = openapi_gen.generate_delete_doc(id_type_str);

    quote! {
        pub async fn #delete_fn(db: &DatabaseConnection, #fn_arg) -> HttpResult<#error_type> {
            let entity = #call_expr.one(db).await
                .map_err(|e| #error_type::DatabaseError(e.to_string()))?
                .ok_or_else(|| #error_type::NotFound(format!("{} not found", id)))?;
            entity.delete(db).await.map_err(|e| {
                eprintln!("删除失败: {}", e); #error_type::DatabaseConnectionError(db_err_map(e).into())
            })?;
            Ok(ApiResponse::<EmptyResponse>::success(EmptyResponse, "删除成功").to_http_response())
        }
        #doc
        #[crate::route_permission(path = #full_path, method = "delete", permission = #perm)]
        pub async fn #handler(db: web::Data<DatabaseConnection>, id: web::Path<#path_param_type>,
        ) -> HttpResult<#error_type> { #delete_fn(db.get_ref(), id.into_inner()).await }
    }
}

#[allow(clippy::too_many_arguments)]
fn generate_list_code(
    entity: &Ident,
    route_prefix: &LitStr,
    permission_prefix: &LitStr,
    openapi_gen: &OpenApiGenerator,
    error_type: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let get_fn = format_ident!("get_{}_all", entity);
    let handler = format_ident!("get_{}_all_handler", entity);
    let full_path = route_prefix.value().to_string();
    let perm = format!("get::{}:read::list", permission_prefix.value());
    let doc = openapi_gen.generate_list_doc();

    quote! {
        #doc
        pub async fn #get_fn(db_pool: &DatabaseConnection, page: u64, limit: u64) -> Result<HttpResponse, #error_type> {
            let paginator = #entity::Entity::find().paginate(db_pool, limit);
            let total = paginator.num_items().await
                .map_err(|e| { eprintln!("查询总数失败: {}", e); #error_type::DatabaseConnectionError("获取失败".into()) })?;
            let data = paginator.fetch_page(page.saturating_sub(1)).await
                .map_err(|e| { eprintln!("查询列表失败: {}", e); #error_type::DatabaseConnectionError("获取列表失败".into()) })?;
            Ok(ApiResponse::success(PaginatedResp { data, pagination: Pagination { total, page, limit } }, "获取成功").to_http_response())
        }
        #[crate::route_permission(path = #full_path, method = "get", permission = #perm)]
        pub async fn #handler(db: web::Data<DatabaseConnection>, query: web::Query<PaginationQuery>,
        ) -> HttpResult<#error_type> {
            let PaginationQuery { page, limit, .. } = query.into_inner();
            #get_fn(db.as_ref(), page, limit).await
        }
    }
}
