# route-macros-types

Shared types for the `route-macros` crate. Contains the response types, error types, pagination types, validation types, and utility macros used by the generated CRUD handler code.

## When to use this

You typically only need this crate if you're using `route-macros` to auto-generate CRUD handlers. Add it alongside `route-macros`:

```toml
[dependencies]
route-macros = "0.1"
route-macros-types = "0.1"
```

## Provided Types

| Type | Description |
|------|-------------|
| `ApiResponse<T>` | Unified JSON API response `{ code, message, data }` |
| `EmptyResponse` | Unit struct for responses with no data (`data: null`) |
| `HttpResult<E>` | Type alias for `Result<HttpResponse, E>` |
| `PaginatedResp<T>` | Paginated list response wrapper |
| `Pagination` | Pagination metadata `{ page, limit, total }` |
| `PaginationQuery` | Query parameters for list endpoints with `#[derive(IntoParams)]` |
| `RouteInfo` | Route metadata collected by `route_permission` macro |
| `ValidationErrorJson` / `ValidationErrorItem` | Validation error structure |
| `AppError` | Default error enum (can be overridden via `error_type` config) |

## Provided Macros

| Macro | Description |
|-------|-------------|
| `impl_entity_unique_check!` | Adds `find_by_col` and `check_unique` methods to SeaORM entities |
| `impl_from_request!` | Implements `From<Request> for ActiveModel` for simple field mapping |
| `impl_from_request_with_default!` | Same as above but supports custom field transforms and defaults |
| `log_macro_info!` | Compile-time logging helper for proc macros |

## Provided Functions

| Function | Description |
|----------|-------------|
| `db_err_map(e: DbErr) -> &'static str` | Maps SeaORM database errors to user-friendly Chinese messages |

## Custom Error Type

If your project has its own error type, you can use `error_type` in `crud_entity!`:

```rust
crud_entity!({
    entity: my_entity,
    error_type: crate::MyAppError,
    // ...
});
```

Your custom error type must implement the variants listed in the `route-macros` README. Alternatively, you can use `route_macros_types::AppError` directly:

```rust
use route_macros_types::AppError;
```
