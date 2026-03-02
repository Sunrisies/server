use quote::quote;
use syn::{Ident, LitStr};

use crate::args::OpenApiConfig;

pub struct OpenApiGenerator<'a> {
    pub entity: &'a Ident,
    pub route_prefix: &'a LitStr,
    pub config: Option<&'a OpenApiConfig>, // 新增：自定义配置
}

impl<'a> OpenApiGenerator<'a> {
    pub fn new(
        entity: &'a Ident,
        route_prefix: &'a LitStr,
        config: Option<&'a OpenApiConfig>,
    ) -> Self {
        Self {
            entity,
            route_prefix,
            config,
        }
    }

    // pub fn generate_create_doc(&self, create_request_type: &Ident) -> proc_macro2::TokenStream {
    //     let entity_str = self.entity.to_string();
    //     let tag = self.get_primary_tag();
    //     let route_path = self.route_prefix.value();

    //     quote! {
    //         #[utoipa::path(
    //             post,
    //             summary = format!("创建{}", #entity_str),
    //             path = #route_path,
    //             tag = #tag,
    //             request_body(content = #create_request_type, description = format!("创建{}的请求数据", #entity_str)),
    //             responses(
    //                 // (status = 200, description = "创建成功", body = ApiResponse<#Self.entity::Model>),
    //                 // (status = 400, description = "请求数据无效", body = ApiResponse<()>),
    //                 // (status = 500, description = "服务器内部错误", body = ApiResponse<()>)
    //             ),
    //             security(
    //                 ("bearer_auth" = [])
    //             )
    //         )]
    //     }
    // }
    /// 检查是否应该生成文档
    pub fn should_generate(&self) -> bool {
        !self.config.map(|c| c.hidden).unwrap_or(false)
    }
    /// 获取摘要（优先使用自定义，否则自动生成）
    fn get_summary(&self, default: &str) -> String {
        self.config
            .and_then(|c| c.summary.clone())
            .unwrap_or_else(|| default.to_string())
    }
    /// 获取描述（优先使用自定义，否则使用默认值）
    fn get_description(&self, default: &str) -> String {
        self.config
            .and_then(|c| c.description.clone())
            .unwrap_or_else(|| default.to_string())
    }

    /// 获取标签（优先使用自定义）
    fn get_tag(&self) -> String {
        self.config
            .and_then(|c| c.tag.clone())
            .or_else(|| Some(self.get_default_tag()))
            .unwrap()
    }
    fn get_default_tag(&self) -> String {
        match self.entity.to_string().as_str() {
            "categories" => "分类管理".to_string(),
            "tags" => "标签管理".to_string(),
            "users" => "用户管理".to_string(),
            "external_links" => "外部链接".to_string(),
            _ => self.entity.to_string(),
        }
    }

    /// 获取 deprecated 属性
    fn get_deprecated_attr(&self) -> proc_macro2::TokenStream {
        let is_deprecated = self.config.map(|c| c.deprecated).unwrap_or(false);
        if is_deprecated {
            quote! { , deprecated }
        } else {
            quote! {}
        }
    }
    /// 生成读取单条记录的文档（标准版 - 按ID查询）
    pub fn generate_read_doc(&self, id_type: &str) -> proc_macro2::TokenStream {
        if !self.should_generate() {
            return quote! {};
        }

        let entity_str = self.entity.to_string();
        let entity = self.entity; // 将 self.entity 绑定到局部变量
        let summary = self.get_summary(&format!("获取{}详情", entity_str));
        let description = self.get_description(&format!("根据ID获取单个{}的详细信息", entity_str));
        let tag = self.get_tag();
        let route_path = format!("{}/{{id}}", self.route_prefix.value());
        let id_description = match id_type {
            "uuid" => "UUID 标识符",
            _ => "数字 ID",
        };
        let deprecated_attr = self.get_deprecated_attr();

        quote! {
            #[utoipa::path(
                get,
                path = #route_path,
                tag = #tag,
                summary = #summary,
                description = #description #deprecated_attr,
                params(
                    ("id" = String, Path, description = #id_description)
                ),
                responses(
                    (status = 200, description = "获取成功", body = crate::ApiResponse<#entity::Model>),
                    (status = 404, description = concat!(#entity_str, "不存在"), body = crate::ApiResponse<crate::EmptyResponse>),
                    (status = 500, description = "服务器内部错误", body = crate::ApiResponse<crate::EmptyResponse>)
                ),
                security(
                    ("bearer_auth" = [])
                )
            )]
        }
    }

    /// 生成列表查询的文档
    pub fn generate_list_doc(&self) -> proc_macro2::TokenStream {
        if !self.should_generate() {
            return quote! {};
        }

        let entity_str = self.entity.to_string();
        let entity = self.entity;
        let summary = self.get_summary(&format!("获取{}列表", entity_str));
        let description = self.get_description(&format!("分页获取{}列表", entity_str));
        let tag = self.get_tag();
        let route_path = self.route_prefix.value();
        let deprecated_attr = self.get_deprecated_attr();

        quote! {
            #[utoipa::path(
                get,
                path = #route_path,
                tag = #tag,
                summary = #summary,
                description = #description #deprecated_attr,
                params(
                    ("page" = Option<u64>, Query, description = "页码，从1开始，默认1",example=1),
                    ("limit" = Option<u64>, Query, description = "每页数量，默认10",example=10)
                ),
                responses(
                    (status = 200, description = "获取成功", body = crate::ApiResponse<PaginatedResp<#entity::Model>>),
                    (status = 500, description = "服务器内部错误", body = crate::ApiResponse<crate::EmptyResponse>)
                ),
                security(
                    ("bearer_auth" = [])
                )
            )]
        }
    }

    // pub fn generate_read_doc(&self, id_type: &str) -> proc_macro2::TokenStream {
    //     let entity_str = self.entity.to_string();
    //     let tag = self.get_primary_tag();
    //     let route_path = format!("{}/{{id}}", self.route_prefix.value());
    //     let id_description = match id_type {
    //         "uuid" => "UUID 标识符",
    //         _ => "数字 ID",
    //     };
    //     // println!("route_path: {}", route_path);
    //     // println!("id_description: {}", id_description);
    //     // println!("生成的 OpenAPI 文档: {}, id_type:{}", entity_str, id_type);
    //     quote! {
    //         #[utoipa::path(
    //             get,
    //             summary = format!("获取{}详情", #entity_str),
    //             path = #route_path,
    //             tag = #tag,
    //             params(
    //                 ("id" = String, Path, description = #id_description)
    //             ),
    //             responses(
    //                 // (status = 200, description = "获取成功", body = ApiResponse<#entity::Model>),
    //                 // (status = 404, description = format!("{}不存在", #entity_str), body = ApiResponse<()>),
    //                 // (status = 500, description = "服务器内部错误", body = ApiResponse<()>)
    //             ),
    //             security(
    //                 ("bearer_auth" = [])
    //             )
    //         )]

    //     }
    // }

    // pub fn generate_list_doc(&self) -> proc_macro2::TokenStream {
    //     let entity_str = self.entity.to_string();
    //     let tag = self.get_primary_tag();
    //     let route_path = self.route_prefix.value();
    //     let list_entity = format_ident!("{}List", self.entity);

    //     quote! {
    //         #[utoipa::path(
    //             get,
    //             summary = format!("获取{}列表", #entity_str),
    //             path = #route_path,
    //             tag = #tag,
    //             params(
    //                 ("page" = Option<u64>, Query, description = "页码，从1开始"),
    //                 ("limit" = Option<u64>, Query, description = "每页数量")
    //             ),
    //             responses(
    //                 (status = 200, description = "获取成功", body = ApiResponse<Vec<#entity::Model>>),
    //                 (status = 500, description = "服务器内部错误", body = ApiResponse<()>)
    //             ),
    //             security(
    //                 ("bearer_auth" = [])
    //             )
    //         )]
    //     }
    // }

    // pub fn generate_delete_doc(&self, id_type: &str) -> proc_macro2::TokenStream {
    //     let entity_str = self.entity.to_string();
    //     let tag = self.get_primary_tag();
    //     let route_path = format!("{}/{{id}}", self.route_prefix.value());
    //     let id_description = match id_type {
    //         "uuid" => "UUID 标识符",
    //         _ => "数字 ID",
    //     };

    //     quote! {
    //         #[utoipa::path(
    //             delete,
    //             summary = format!("删除{}", #entity_str),
    //             path = #route_path,
    //             tag = #tag,
    //             params(
    //                 ("id" = String, Path, description = #id_description)
    //             ),
    //             responses(
    //                 (status = 200, description = "删除成功", body = ApiResponse<()>),
    //                 (status = 404, description = format!("{}不存在", #entity_str), body = ApiResponse<()>),
    //                 (status = 500, description = "服务器内部错误", body = ApiResponse<()>)
    //             ),
    //             security(
    //                 ("bearer_auth" = [])
    //             )
    //         )]
    //     }
    // }

    fn get_primary_tag(&self) -> String {
        // if let Some(tags) = self.tags {
        //     if let Some(first_tag) = tags.first() {
        //         return first_tag.value();
        //     }
        // }

        // 默认使用实体名的中文翻译
        match self.entity.to_string().as_str() {
            "categories" => "分类".to_string(),
            "tags" => "标签".to_string(),
            "users" => "用户".to_string(),
            _ => self.entity.to_string(),
        }
    }

    // pub fn get_all_tags(&self) -> Vec<String> {
    //     if let Some(tags) = self.tags {
    //         tags.iter().map(|lit| lit.value()).collect()
    //     } else {
    //         vec![self.get_primary_tag()]
    //     }
    // }
}
