# TODO

## 🔴 高优先级

- [ ] **ImageService 三个接口是空壳** — 已路由注册，需补齐 DB 查询逻辑
- [ ] **云剪贴板前端实现** — 后端已就绪，前端待开发

## 🟡 中优先级

- [ ] **main.rs 启动时打印 JWT_SECRET 到日志** — `log::info!("app config: {:#?}", CONFIG.jwt)` 会泄露密钥
- [ ] **route-macros 实现 Update 操作** — `CrudOperation::Update` 枚举已加，但 `generate_update_code` 未实现
- [ ] **generate_delete_doc 缺少 deprecated 属性** — create/read/list 都有，唯独 delete 没有
- [ ] **路由匹配问题** — `routes/posts.rs` 中 `/prevNext/{uuid:.*}` 可能抢 `/uploadTime`
- [ ] **CI 未执行测试** — GitHub Actions 安装了 nextest，需配置 `cargo test -- --test-threads=1`
- [ ] **验证码存在 HashMap 里** — 服务重启验证码丢失

## 🟢 低优先级

- [ ] **文章内容搜索** — 支持关键词搜索文章正文
- [ ] **密码重置** — 忘记密码/重置密码流程
- [ ] **接口限流** — 登录接口防暴力破解
- [ ] **管理员仪表盘 API** — 统计数据
