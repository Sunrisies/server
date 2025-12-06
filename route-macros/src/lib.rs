mod args;
mod crud;
mod log;
mod openapi;
mod route_permission;

#[proc_macro_attribute]
pub fn route_permission(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    route_permission::route_permission(attr, item)
}
#[proc_macro]
pub fn crud_entity(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    crud::crud_entity(input)
}

/// 刷新并显示所有日志
#[proc_macro]
pub fn flush_crud_logs(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    log::flush_crud_logs(_input)
}
