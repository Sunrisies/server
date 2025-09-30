use proc_macro::TokenStream;
use syn::{ItemFn, parse_macro_input};

use crate::args::RoutePermissionArgs;

/// 简化的路由权限绑定宏
pub fn route_permission(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let args = parse_macro_input!(attr as RoutePermissionArgs);

    let fn_name = &input.sig.ident;
    let fn_vis = &input.vis;
    let fn_block = &input.block;
    let fn_async = &input.sig.asyncness;
    let fn_inputs = &input.sig.inputs;
    let fn_output = &input.sig.output;

    let path = &args.path;
    let method = &args.method;
    let permission = &args.permission;

    let output = quote::quote! {
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
