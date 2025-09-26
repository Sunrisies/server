use proc_macro::TokenStream;
use quote::quote;
use syn::{
    ItemFn, Lit,
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
