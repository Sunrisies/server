//! # route-macros
//!
//! Procedural macros for auto-generating Actix-web CRUD handlers with OpenAPI documentation.
//!
//! ## Quick Start
//!
//! ```rust
//! use route_macros::crud_entity;
//!
//! crud_entity!({
//!     entity: categories,
//!     route_prefix: "/api/v1/categories",
//!     permission_prefix: "categories",
//!     id_type: "id",
//!     operations: ["create", "read", "list", "delete"],
//!     create_request_type: CreateCategoryRequest,
//!     unique_field: Name,
//!     openapi_summary: "分类管理",
//! });
//! ```
//!
//! See the [README](https://github.com/Sunrisies/server/tree/main/route-macros) for full documentation.

mod args;
mod crud;
mod openapi;
mod route_permission;

/// Attribute macro for marking route permissions.
///
/// Collects `RouteInfo` via `inventory` for runtime route registry.
///
/// ```ignore
/// #[route_permission(path = "/api/v1/items", method = "get", permission = "items:read")]
/// pub async fn get_items_handler() -> HttpResult { ... }
/// ```
#[proc_macro_attribute]
pub fn route_permission(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    route_permission::route_permission(attr, item)
}

/// Macro for auto-generating CRUD handlers from a SeaORM entity.
///
/// Generates a module with handler functions and `#[utoipa::path]` annotations.
///
/// # Required dependencies in consumer crate
///
/// - `route-macros-types` (for `ApiResponse`, `EmptyResponse`, `HttpResult`, etc.)
/// - `actix-web` (for `web::Data`, `web::Path`, `HttpResponse`, etc.)
/// - `sea-orm` (for `DatabaseConnection`, `EntityTrait`, `PaginatorTrait`, etc.)
/// - `validator` (for request validation)
///
/// # Generated functions
///
/// For a entity named `Item` with operations `["create", "read", "list", "delete"]`:
///
/// | Generated function | HTTP | Path |
/// |---|---|------|
/// | `create_item_handler` | POST | `/api/v1/items` |
/// | `get_item_handler` | GET | `/api/v1/items/{id}` |
/// | `get_item_all_handler` | GET | `/api/v1/items` |
/// | `delete_item_handler` | DELETE | `/api/v1/items/{id}` |
///
/// All generated code is placed in a module named `item_routes`.
#[proc_macro]
pub fn crud_entity(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    crud::crud_entity(input)
}
