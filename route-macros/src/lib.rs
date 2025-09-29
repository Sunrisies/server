use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Expr, ExprArray, Ident, ItemFn, Lit, LitStr, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
    token::Comma,
};
// // 自定义结构来解析属性参数
#[derive(Debug)]
struct RoutePermissionArgs {
    path: String,
    method: String,
    permission: String,
}
impl Parse for RoutePermissionArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut path = None;
        let mut method = None;
        let mut permission = None;

        while !input.is_empty() {
            let key: syn::Ident = input.parse()?;
            input.parse::<syn::Token![=]>()?;
            let value: Lit = input.parse()?;

            match key.to_string().as_str() {
                "path" => {
                    if let Lit::Str(lit) = value {
                        path = Some(lit.value());
                    }
                }
                "method" => {
                    if let Lit::Str(lit) = value {
                        method = Some(lit.value().to_lowercase());
                    }
                }
                "permission" => {
                    if let Lit::Str(lit) = value {
                        permission = Some(lit.value());
                    }
                }
                _ => {}
            }

            // 检查是否有逗号分隔符
            if input.peek(Comma) {
                input.parse::<Comma>()?;
            }
        }

        Ok(RoutePermissionArgs {
            path: path.ok_or_else(|| input.error("path attribute is required"))?,
            method: method.ok_or_else(|| input.error("method attribute is required"))?,
            permission: permission
                .ok_or_else(|| input.error("permission attribute is required"))?,
        })
    }
}

/// 简化的路由权限绑定宏
#[proc_macro_attribute]
pub fn route_permission(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let args = parse_macro_input!(attr as RoutePermissionArgs);
    // 直接返回原始函数，不进行任何处理（用于测试）
    let fn_name = &input.sig.ident;
    let fn_vis = &input.vis;
    let fn_block = &input.block;
    let fn_async = &input.sig.asyncness;
    let fn_inputs = &input.sig.inputs;
    let fn_output = &input.sig.output;

    let path = &args.path;
    let method = &args.method;
    let permission = &args.permission;
    let output = quote! {
        #fn_vis #fn_async fn #fn_name(#fn_inputs) #fn_output #fn_block

        // 使用inventory进行自动注册
        inventory::submit! {
            RouteInfo {
                path: #path,
                method: #method,
                permission: #permission,
            }
        }
    };
    output.into()
}
#[derive(Debug, PartialEq)]
enum IdType {
    Uuid,
    Custom(String),
}
// CRUD 操作枚举
#[derive(Debug, PartialEq)]
enum CrudOperation {
    Create,
    Read,
    // Update,
    Delete,
    List,
}
// 新的参数结构，支持命名参数
#[derive(Debug, PartialEq)]
struct CrudEntityConfig {
    entity: Ident,
    route_prefix: LitStr,
    permission_prefix: LitStr,
    id_type: Option<IdType>,
    operations: Option<Vec<CrudOperation>>,
    create_request_type: Option<Ident>, // 新增请求类型
}

impl Parse for CrudEntityConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        syn::braced!(content in input);

        let mut entity = None;
        let mut route_prefix = None;
        let mut permission_prefix = None;
        let mut id_type = None;
        let mut operations = None;
        let mut create_request_type = None;

        // 解析键值对
        while !content.is_empty() {
            let key: Ident = content.parse()?;
            content.parse::<Token![:]>()?;

            match key.to_string().as_str() {
                "entity" => {
                    let value: Ident = content.parse()?;
                    entity = Some(value);
                }
                "route_prefix" => {
                    let value: LitStr = content.parse()?;
                    route_prefix = Some(value);
                }
                "permission_prefix" => {
                    let value: LitStr = content.parse()?;
                    permission_prefix = Some(value);
                }
                "id_type" => {
                    let value: LitStr = content.parse()?;
                    id_type = Some(match value.value().as_str() {
                        "uuid" => IdType::Uuid,
                        custom => IdType::Custom(custom.to_string()),
                    });
                }
                "operations" => {
                    let array: ExprArray = content.parse()?;
                    let mut ops = Vec::new();

                    for elem in array.elems {
                        if let Expr::Lit(lit) = elem {
                            if let syn::Lit::Str(lit_str) = lit.lit {
                                match lit_str.value().as_str() {
                                    "create" => ops.push(CrudOperation::Create),
                                    "read" => ops.push(CrudOperation::Read),
                                    // "update" => ops.push(CrudOperation::Update),
                                    "delete" => ops.push(CrudOperation::Delete),
                                    "list" => ops.push(CrudOperation::List),
                                    _ => {
                                        return Err(syn::Error::new_spanned(
                                            lit_str,
                                            "Unknown operation",
                                        ));
                                    }
                                }
                            }
                        }
                    }
                    operations = Some(ops);
                }
                "create_request_type" => {
                    let value: Ident = content.parse()?;
                    create_request_type = Some(value);
                }
                _ => {
                    return Err(syn::Error::new_spanned(key, "Unknown field"));
                }
            }

            // 检查是否有逗号分隔符
            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            }
        }

        Ok(CrudEntityConfig {
            entity: entity.ok_or_else(|| content.error("Missing required field 'entity'"))?,
            route_prefix: route_prefix
                .ok_or_else(|| content.error("Missing required field 'route_prefix'"))?,
            permission_prefix: permission_prefix
                .ok_or_else(|| content.error("Missing required field 'permission_prefix'"))?,
            id_type,
            operations,
            create_request_type, // 新增请求类型
        })
    }
}
/// 简化版 CRUD 宏 - 为实体快速生成标准 CRUD 操作
#[proc_macro]
pub fn crud_entity(input: TokenStream) -> TokenStream {
    let config = parse_macro_input!(input as CrudEntityConfig);

    let entity = &config.entity;
    let route_prefix = &config.route_prefix;
    let permission_prefix = &config.permission_prefix;
    // 根据ID类型生成不同的代码
    let id_type = config.id_type.unwrap_or(IdType::Uuid);

    let operations = config.operations.unwrap_or_else(|| {
        vec![
            CrudOperation::Create,
            CrudOperation::Read,
            // CrudOperation::Update,
            CrudOperation::Delete,
            CrudOperation::List,
        ]
    });

    // 根据 ID 类型确定相关类型
    let (id_rust_type, find_method, path_param_type) = match id_type {
        IdType::Uuid => (
            quote! { String },
            quote! { find_by_uuid },
            quote! { String },
        ),
        IdType::Custom(_custom_type) => (quote! { i32 }, quote! { find_by_id }, quote! { i32 }),
    };
    println!(
        "id_rust_type:{}, find_method:{}, path_param_type:{}",
        id_rust_type, find_method, path_param_type
    );
    let mod_name = format_ident!("{}_routes", entity.to_string().to_lowercase());
    // let mut generated_code = quote! {};
    let mut create_code = quote! {};
    let mut read_code = quote! {};
    // let mut update_code = quote! {};
    let mut delete_code = quote! {};
    let mut list_code = quote! {};

    // 为每个操作生成代码
    for operation in operations {
        match operation {
            CrudOperation::Create => {
                create_code = generate_create_code(
                    entity,
                    route_prefix,
                    permission_prefix,
                    &config.create_request_type,
                );
            }
            CrudOperation::Read => {
                read_code = generate_read_code(
                    entity,
                    route_prefix,
                    permission_prefix,
                    &id_rust_type,
                    &find_method,
                    &path_param_type,
                );
            } // CrudOperation::Update => {
            //     // update_code = generate_update_code(
            //     //     entity,
            //     //     route_prefix,
            //     //     permission_prefix,
            //     //     &id_rust_type,
            //     //     &path_param_type,
            //     // );
            // }
            CrudOperation::Delete => {
                delete_code = generate_delete_code(
                    entity,
                    route_prefix,
                    permission_prefix,
                    &id_rust_type,
                    &find_method,
                );
            }
            CrudOperation::List => {
                list_code = generate_list_code(entity, route_prefix, permission_prefix);
            }
        }
    }
    let output = quote! {
        pub mod #mod_name {
            use super::*;
              #create_code
        #read_code
        // #update_code
        #delete_code
        #list_code
        }
    };

    TokenStream::from(output)
}

fn generate_read_code(
    entity: &Ident,
    route_prefix: &LitStr,
    permission_prefix: &LitStr,
    id_rust_type: &proc_macro2::TokenStream,
    find_method: &proc_macro2::TokenStream,
    path_param_type: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let get_fn = format_ident!("get_{}", entity.to_string().to_lowercase());
    let get_handler = format_ident!("get_{}_handler", entity.to_string().to_lowercase());
    let full_path = format!("{}/{{id}}", route_prefix.value());
    let full_permission = format!("get::{}:read", permission_prefix.value());
    println!("get_fn:{},get_handler:{}", get_fn, get_handler);
    let output = quote! {
        /// 获取实体
        pub async fn #get_fn(
            db: &DatabaseConnection,
            id: #id_rust_type,
        ) -> Result<#entity::Model, AppError> {
            #entity::Entity::#find_method(&id)
                .one(db)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?
                .ok_or_else(|| AppError::NotFound(format!("{} not found", id)))
        }

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
                Ok(result) => Ok(HttpResponse::Ok().json(result)),
                Err(e) => Err(e.into()),
            }
        }
    };

    output
}

fn generate_create_code(
    entity: &Ident,
    route_prefix: &LitStr,
    permission_prefix: &LitStr,
    create_request_type: &Option<proc_macro2::Ident>,
) -> proc_macro2::TokenStream {
    let create_fn = format_ident!("create_{}", entity.to_string().to_lowercase());
    let create_handler = format_ident!("create_{}_handler", entity.to_string().to_lowercase());
    let full_path = format!("{}", route_prefix.value());
    println!("create_fn:{},create_handler:{}", create_fn, create_handler);
    // 生成权限字符串
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
    let output = quote! {
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
            // Ok(ApiResponse::success("1111", "添加成功").to_http_response())
            match #create_fn(db.get_ref(), data.into_inner()).await {
                Ok(category) => Ok(ApiResponse::success(category, "添加成功").to_http_response()),
                Err(AppError::DatabaseConnectionError(msg)) => {
                    // 统一包装：HTTP 200，业务码 200，message 提示不存在
                    Ok(ApiResponse::<()>::success_msg(&msg).to_http_response())
                }
                Err(e) => {
                    // 其他错误（数据库等）按原样返回 500/400 等
                    Ok(ApiResponse::from(e).to_http_response())
                }
            }
        }
    };

    output
}

// fn generate_update_code(
//     entity: &Ident,
//     route_prefix: &LitStr,
//     permission_prefix: &LitStr,
//     id_rust_type: &TokenStream,
//     find_method: &TokenStream,
// ) -> TokenStream {
//     let get_fn = format_ident!("get_{}", entity.to_string().to_lowercase());
//     let get_handler = format_ident!("get_{}_handler", entity.to_string().to_lowercase());
//     let full_path = format!("{}/{{id}}", route_prefix.value());
//     let full_permission = format!("{}:read", permission_prefix.value());

//     let output = quote! {};

//     output.into()
// }

fn generate_delete_code(
    entity: &Ident,
    route_prefix: &LitStr,
    permission_prefix: &LitStr,
    id_rust_type: &proc_macro2::TokenStream,
    find_method: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let delete_fn = format_ident!("delete_{}", entity.to_string().to_lowercase());
    let delete_handler = format_ident!("delete_{}_handler", entity.to_string().to_lowercase());
    let full_path = format!("{}/{{id}}", route_prefix.value());
    let full_permission = format!("{}:delete:id", permission_prefix.value());
    println!(
        "delete_fn:{},delete_handler:{},id_rust_type:{}",
        delete_fn, delete_handler, id_rust_type
    );
    let output = quote! {
        pub async fn #delete_fn(
            db: &DatabaseConnection,
            id: #id_rust_type,
        ) ->HttpResult {
            println!("Deleting {} with id: {},find_method:{}", stringify!(#entity), id,stringify!(#find_method));
            let entity = #entity::Entity::#find_method(id).one(db)
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
            id: web::Path<#id_rust_type>,
        ) -> HttpResult {
            let result = #delete_fn(db.get_ref(), id.into_inner()).await?;
            Ok(result)
        }
    };

    output
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
    println!("get_fn:{},get_handler:{}", get_fn, get_handler);
    let output = quote! {
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
    };

    output
}
