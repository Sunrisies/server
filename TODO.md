# TODO

## 🔴 高优先级（功能缺失 / 阻塞）

- [ ] **分类删除路由未注册** — `src/routes/categories.rs` 缺少 DELETE 路由，但 `api_doc.rs` 中有注册，前后不一致
- [ ] **ImageService 三个接口是空壳** — `get_images`、`get_image_by_id`、`delete_image` 返回假数据，需补齐
- [ ] **云剪贴板前端实现** — `pages/admin/clipboard.vue` + 组件尚未开发（后端已就绪）

## 🟡 中优先级（优化 / 补齐）

- [ ] **route-macros 实现 Update 操作** — `CrudOperation::Update` 枚举已加，但 `generate_update_code` 未实现
- [ ] **generate_delete_doc 缺少 deprecated 属性** — create/read/list 都有，唯独 delete 没有
- [ ] **路由匹配问题** — `routes/posts.rs` 中 `/prevNext/{uuid:.*}` 可能抢 `/uploadTime`
- [ ] **CI 未执行测试** — GitHub Actions 安装了 nextest 但没有 `cargo test` 步骤
- [ ] **UploadManager / ImageService 代码重复** — 两套七牛上传逻辑，应合并

## 🟢 低优先级（新功能 / 增强）

- [ ] **文章内容搜索** — 目前仅支持按分类/标签筛选，不支持关键词搜索
- [ ] **验证码持久化** — `EmailVerificationManager` 使用 `HashMap` 存验证码，重启丢失
- [ ] **密码重置** — 目前没有忘记密码/重置密码流程
- [ ] **接口限流** — 登录接口无防暴力破解限制
- [ ] **管理员仪表盘 API** — 统计数据（文章数、评论数等）
- [ ] **数据库连接信息输出到日志** — `main.rs` 中 `log::info!("app config: {:#?}", CONFIG.jwt)` 可能泄露敏感信息（JWT_SECRET）
