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
    Integer,
    Custom(String),
}
// CRUD 操作枚举
#[derive(Debug, PartialEq)]

enum CrudOperation {
    // Create,
    Read,
    // Update,
    // Delete,
    // List,
}
// 新的参数结构，支持命名参数
#[derive(Debug, PartialEq)]
struct CrudEntityConfig {
    entity: Ident,
    route_prefix: LitStr,
    permission_prefix: LitStr,
    id_type: Option<IdType>,
    operations: Option<Vec<CrudOperation>>,
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
                        "integer" => IdType::Integer,
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
                                    // "create" => ops.push(CrudOperation::Create),
                                    "read" => ops.push(CrudOperation::Read),
                                    // "update" => ops.push(CrudOperation::Update),
                                    // "delete" => ops.push(CrudOperation::Delete),
                                    // "list" => ops.push(CrudOperation::List),
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
                _ => return Err(syn::Error::new_spanned(key, "Unknown field")),
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
    let id_type = config.id_type.unwrap_or(IdType::Integer);

    let operations = config.operations.unwrap_or_else(|| {
        vec![
            // CrudOperation::Create,
            CrudOperation::Read,
            // CrudOperation::Update,
            // CrudOperation::Delete,
            // CrudOperation::List,
        ]
    });

    // 根据 ID 类型确定相关类型
    let (id_rust_type, find_method, path_param_type) = match id_type {
        IdType::Uuid => (
            quote! { String },
            quote! { find_by_uuid },
            quote! { String },
        ),
        IdType::Integer => (quote! { i32 }, quote! { find_by_id }, quote! { i32 }),
        IdType::Custom(custom_type) => {
            let custom_ident = format_ident!("{}", custom_type);
            (
                quote! { #custom_ident },
                quote! { find_by_id },
                quote! { #custom_ident },
            )
        }
    };

    let mod_name = format_ident!("{}_routes", entity.to_string().to_lowercase());
    // let mut generated_code = quote! {};
    // let mut create_code = quote! {};
    let mut read_code = quote! {};
    // let mut update_code = quote! {};
    // let mut delete_code = quote! {};
    // let mut list_code = quote! {};

    // 为每个操作生成代码
    for operation in operations {
        match operation {
            // CrudOperation::Create => {
            //     // create_code =
            //     //     generate_create_code(entity, route_prefix, permission_prefix, &id_rust_type);
            // }
            CrudOperation::Read => {
                read_code = generate_read_code(
                    entity,
                    // route_prefix,
                    // permission_prefix,
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
              // CrudOperation::Delete => {
              //     // delete_code = generate_delete_code(
              //     //     entity,
              //     //     route_prefix,
              //     //     permission_prefix,
              //     //     &id_rust_type,
              //     //     &path_param_type,
              //     // );
              // }
              // CrudOperation::List => {
              //     // list_code = generate_list_code(entity, route_prefix, permission_prefix);
              // }
        }
    }
    // println!("read_code:{:#?}", read_code);

    // let fragments_iter = fragments.iter();
    let output = quote! {
        pub mod #mod_name {
            use super::*;
            //   #create_code
        #read_code
        // #update_code
        // #delete_code
        // #list_code
        }
    };

    TokenStream::from(output)
}

fn generate_read_code(
    entity: &Ident,
    // route_prefix: &LitStr,
    // permission_prefix: &LitStr,
    id_rust_type: &proc_macro2::TokenStream,
    find_method: &proc_macro2::TokenStream,
    path_param_type: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let get_fn = format_ident!("get_{}", entity.to_string().to_lowercase());
    let get_handler = format_ident!("get_{}_handler", entity.to_string().to_lowercase());
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

        // #[::route_macros::route_permission(
        //     path = #full_path,
        //     method = "get",
        //     permission = #full_permission
        // )]
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

// fn generate_create_code(
//     entity: &Ident,
//     route_prefix: &LitStr,
//     permission_prefix: &LitStr,
//     id_rust_type: &TokenStream,
//     // find_method: &TokenStream,
//     // path_param_type: &TokenStream,
// ) -> TokenStream {
//     let get_fn = format_ident!("get_{}", entity.to_string().to_lowercase());
//     let get_handler = format_ident!("get_{}_handler", entity.to_string().to_lowercase());
//     let full_path = format!("{}/{{id}}", route_prefix.value());
//     let full_permission = format!("{}:read", permission_prefix.value());

//     let output = quote! {
//         /// 获取实体

//     };

//     output.into()
// }

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

// fn generate_delete_code(
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

// fn generate_list_code(
//     entity: &Ident,
//     route_prefix: &LitStr,
//     permission_prefix: &LitStr,
// ) -> TokenStream {
//     let get_fn = format_ident!("get_{}", entity.to_string().to_lowercase());
//     let get_handler = format_ident!("get_{}_handler", entity.to_string().to_lowercase());
//     let full_path = format!("{}/{{id}}", route_prefix.value());
//     let full_permission = format!("{}:read", permission_prefix.value());

//     let output = quote! {
//         /// 获取实体

//     };

//     output.into()
// }
