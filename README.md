# Rust Server
- `web框架:` Actix-web 框架编写的Rust后端服务。
- `序列化:` Serde - 高效的序列化/反序列化
- `检验:` validator - 用于校验用户输入的库



$env:CRUD_MACRO_DEBUG=1
echo $env:CRUD_MACRO_DEBUG
watchexec -w src -w route-macros -r cargo run
