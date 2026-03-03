use syn::{
    Expr, ExprArray, Ident, Lit, LitStr, Token,
    parse::{Parse, ParseStream},
    token::Comma,
};

#[derive(Debug, PartialEq)]
pub struct OpenApiConfig {
    pub summary: Option<String>,     // 自定义摘要
    pub description: Option<String>, // 自定义描述
    pub tag: Option<String>,         // 自定义标签
    pub deprecated: bool,            // 是否废弃
    pub hidden: bool,                // 是否隐藏
}
/// 自定义查询参数
#[derive(Debug, PartialEq)]
pub enum CustomQueryType {
    List, // 自定义列表查询
    Read, // 自定义详情查询
    All,  // 所有查询都自定义
}

// ID 类型枚举
#[derive(Debug, PartialEq)]
pub enum IdType {
    Uuid,
    Custom(String),
}

// CRUD 操作枚举
#[derive(Debug, PartialEq)]
pub enum CrudOperation {
    Create,
    Read,
    Delete,
    List,
}

// CRUD 实体配置
#[derive(Debug, PartialEq)]
pub struct CrudEntityConfig {
    pub entity: Ident,
    pub route_prefix: LitStr,
    pub permission_prefix: LitStr,
    pub id_type: Option<IdType>,
    pub operations: Option<Vec<CrudOperation>>,
    pub create_request_type: Option<Ident>,
    pub custom_queries: Option<Vec<CustomQueryType>>, // 新增：自定义查询类型
    pub custom_list_fn: Option<Ident>,                // 新增：自定义列表查询函数名
    pub custom_read_fn: Option<Ident>,                // 新增：自定义详情查询函数名
    // OpenAPI 配置
    pub openapi_summary: Option<LitStr>, // 自定义摘要
    pub openapi_read: Option<OpenApiConfig>,
    pub openapi_list: Option<OpenApiConfig>,
    pub openapi_delete: Option<OpenApiConfig>,
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
        let mut custom_queries = None;
        let mut custom_list_fn = None;
        let mut custom_read_fn = None;
        let mut openapi_summary = None;
        let mut openapi_read = None;
        let mut openapi_list = None;
        let mut openapi_delete = None;
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
                "custom_queries" => {
                    let array: ExprArray = content.parse()?;
                    let mut custom_query_types = Vec::new();
                    for elem in array.elems {
                        if let Expr::Lit(lit) = elem {
                            if let syn::Lit::Str(lit_str) = lit.lit {
                                match lit_str.value().as_str() {
                                    "list" => custom_query_types.push(CustomQueryType::List),
                                    "read" => custom_query_types.push(CustomQueryType::Read),
                                    "all" => custom_query_types.push(CustomQueryType::All),
                                    _ => {
                                        return Err(syn::Error::new_spanned(
                                            lit_str,
                                            "Unknown custom query type",
                                        ));
                                    }
                                }
                            }
                        }
                    }
                    custom_queries = Some(custom_query_types);
                }
                "custom_list_fn" => {
                    let value: Ident = content.parse()?;
                    custom_list_fn = Some(value);
                }
                "custom_read_fn" => {
                    let value: Ident = content.parse()?;
                    custom_read_fn = Some(value);
                }
                "openapi_summary" => {
                    let value: LitStr = content.parse()?;
                    openapi_summary = Some(value);
                }
                "openapi_read" => {
                    let config = parse_openapi_config(&&content)?;
                    openapi_read = Some(config);
                }
                "openapi_list" => {
                    let config = parse_openapi_config(&&content)?;
                    openapi_list = Some(config);
                }
                "openapi_delete" => {
                    let config = parse_openapi_config(&&content)?;
                    openapi_delete = Some(config);
                }
                _ => {
                    return Err(syn::Error::new_spanned(key, "Unknown field"));
                }
            }

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
            create_request_type,
            custom_queries,
            custom_list_fn,
            custom_read_fn,
            openapi_summary,
            openapi_read,
            openapi_list,
            openapi_delete,
        })
    }
}

// 路由权限参数
#[derive(Debug)]
pub struct RoutePermissionArgs {
    pub path: String,
    pub method: String,
    pub permission: String,
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

fn parse_openapi_config(input: &ParseStream) -> syn::Result<OpenApiConfig> {
    let content;
    syn::braced!(content in input);

    let mut summary = None;
    let mut description = None;
    let mut tag = None;
    let mut deprecated = false;
    let mut hidden = false;

    while !content.is_empty() {
        let key: Ident = content.parse()?;
        content.parse::<Token![:]>()?;

        match key.to_string().as_str() {
            "summary" => {
                let value: LitStr = content.parse()?;
                summary = Some(value.value());
            }
            "description" => {
                let value: LitStr = content.parse()?;
                description = Some(value.value());
            }
            "tag" => {
                let value: LitStr = content.parse()?;
                tag = Some(value.value());
            }
            "deprecated" => {
                let value: syn::LitBool = content.parse()?;
                deprecated = value.value();
            }
            "hidden" => {
                let value: syn::LitBool = content.parse()?;
                hidden = value.value();
            }
            _ => return Err(syn::Error::new_spanned(key, "Unknown openapi config field")),
        }

        if content.peek(Token![,]) {
            content.parse::<Token![,]>()?;
        }
    }

    Ok(OpenApiConfig {
        summary,
        description,
        tag,
        deprecated,
        hidden,
    })
}
