# ⚡ 快速配置向导

本文档提供最快速的配置方法，让您在5分钟内完成基本配置并启动服务。

## 🎯 配置目标

根据您的需求选择配置方案：

### 方案A: 最小配置（仅核心功能）⏱️ 2分钟
- ✅ 用户认证
- ✅ 文章管理
- ✅ 分类标签
- ❌ 文件上传
- ❌ 邮件功能

### 方案B: 完整配置（所有功能）⏱️ 5分钟
- ✅ 所有核心功能
- ✅ 文件上传（七牛云）
- ✅ 邮件发送
- ✅ 实时通信

---

## 📋 方案A: 最小配置（2分钟）

### Step 1: 准备数据库 (1分钟)

**Option 1: 使用Docker（推荐）**
```bash
# 启动PostgreSQL容器
docker run --name blog-postgres \
  -e POSTGRES_PASSWORD=yourpassword \
  -e POSTGRES_DB=blog_db \
  -p 5432:5432 \
  -d postgres:14

# 等待5秒让数据库启动
sleep 5
```

**Option 2: 使用本地PostgreSQL**
```bash
# 登录PostgreSQL
psql -U postgres

# 创建数据库
CREATE DATABASE blog_db;

# 退出
\q
```

### Step 2: 生成JWT密钥 (30秒)

```bash
# 生成随机密钥
openssl rand -base64 32

# 输出示例（复制此输出）:
# Kx9vR2mN8pQ4wL7yF6hJ3sT5nC1dB0aEiH2jK5lM7nP0qR3sT
```

### Step 3: 创建配置文件 (30秒)

```bash
# 复制模板
cp .env.example .env

# 编辑配置文件
nano .env  # Linux/Mac
# 或
notepad .env  # Windows
```

**只需填写这两项**:
```env
# 替换为您的数据库信息
DATABASE_URL=postgresql://postgres:yourpassword@localhost:5432/blog_db

# 粘贴上一步生成的密钥
JWT_SECRET=Kx9vR2mN8pQ4wL7yF6hJ3sT5nC1dB0aEiH2jK5lM7nP0qR3sT
```

**保存文件**: Ctrl+O (nano) 或 Ctrl+S (notepad)

### Step 4: 启动服务 (完成!)

```bash
# 运行数据库迁移
cargo run -- migrate

# 启动服务
cargo run

# 看到以下信息表示成功:
# ✅ Database connected
# ✅ Server running on http://127.0.0.1:8080
```

**测试**:
```bash
# 打开浏览器访问
http://localhost:8080/api-doc/openapi.json

# 或使用curl测试
curl http://localhost:8080/api/v1/health
```

✅ **完成！现在可以使用核心功能了。**

---

## 📋 方案B: 完整配置（5分钟）

### Step 1-3: 同方案A (2分钟)

执行方案A的Step 1-3，完成数据库和JWT配置。

### Step 4: 配置七牛云存储 (2分钟)

**4.1 注册并获取密钥**
1. 访问 https://portal.qiniu.com/signup 注册账号
2. 进入"密钥管理" https://portal.qiniu.com/user/key
3. 复制 AccessKey 和 SecretKey

**4.2 创建存储空间**
1. 进入"对象存储" https://portal.qiniu.com/kodo/bucket
2. 点击"新建空间"
3. 填写信息:
   - 空间名称: `blog-uploads`（或自定义）
   - 存储区域: 选择最近的区域
   - 访问控制: 公开
4. 创建后，查看"域名管理"获取CDN域名

**4.3 添加配置**
```env
# 在 .env 文件中添加（替换为您的实际值）
QINIU_ACCESS_KEY=你的AccessKey
QINIU_SECRET_KEY=你的SecretKey
QINIU_BUCKET=blog-uploads
QINIU_DOMAIN=你的CDN域名.bkt.clouddn.com
```

### Step 5: 配置邮件服务 (1分钟)

**Option 1: 使用Gmail（推荐）**

1. 启用两步验证: https://myaccount.google.com/security
2. 生成应用专用密码: https://myaccount.google.com/apppasswords
3. 选择"应用"→"邮件"，生成密码

```env
# 在 .env 文件中添加
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=生成的16位密码
SMTP_FROM_NAME=我的博客
```

**Option 2: 使用QQ邮箱**

1. 登录QQ邮箱 → 设置 → 账户
2. 开启"POP3/SMTP服务"
3. 获取授权码

```env
SMTP_HOST=smtp.qq.com
SMTP_PORT=587
SMTP_USERNAME=your-qq-number@qq.com
SMTP_PASSWORD=授权码
SMTP_FROM_NAME=我的博客
```

### Step 6: 启动服务 (完成!)

```bash
# 启动服务
cargo run

# 测试文件上传
curl -X POST http://localhost:8080/api/v1/upload \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -F "file=@test.jpg"

# 测试邮件发送（需要先登录获取token）
curl -X POST http://localhost:8080/api/v1/email/verify \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com"}'
```

✅ **完成！所有功能已就绪。**

---

## 🔍 配置验证

### 检查配置是否正确

```bash
# 检查环境变量
cat .env

# 应该至少包含:
# ✅ DATABASE_URL
# ✅ JWT_SECRET
# 🟡 QINIU_* (如果需要文件上传)
# 🟡 SMTP_* (如果需要邮件)
```

### 测试数据库连接

```bash
# 使用psql测试
psql $DATABASE_URL

# 如果成功连接，输入:
\l  # 列出数据库
\q  # 退出
```

### 测试JWT密钥

```bash
# 检查密钥长度
echo -n "你的JWT_SECRET" | wc -c

# 应该 >= 32
```

---

## 🐛 常见问题快速修复

### 问题1: 数据库连接失败
```
❌ Error: database connection failed
```

**解决方案**:
```bash
# 1. 检查PostgreSQL是否运行
docker ps  # 或 systemctl status postgresql

# 2. 测试连接
psql "postgresql://postgres:password@localhost:5432/blog_db"

# 3. 检查DATABASE_URL格式
# 格式: postgresql://用户名:密码@地址:端口/数据库名
```

### 问题2: JWT密钥太短
```
❌ Error: JWT secret must be at least 32 characters
```

**解决方案**:
```bash
# 重新生成密钥
openssl rand -base64 32

# 更新 .env 文件中的 JWT_SECRET
```

### 问题3: 端口被占用
```
❌ Error: Address already in use (os error 98)
```

**解决方案**:
```bash
# 查找占用8080端口的进程
lsof -i :8080  # Linux/Mac
netstat -ano | findstr :8080  # Windows

# 杀死进程或更改端口
# 在 .env 中添加:
SERVER_PORT=8081
```

### 问题4: 七牛云上传失败
```
❌ Error: Qiniu authentication failed
```

**解决方案**:
```bash
# 1. 检查密钥是否正确（不要有空格）
# 2. 确认存储空间名称正确
# 3. 测试密钥:
curl -u "AccessKey:SecretKey" \
  http://rs.qiniu.com/bucket/你的空间名
```

### 问题5: 邮件发送失败
```
❌ Error: SMTP authentication failed
```

**解决方案**:
```bash
# Gmail: 确认使用应用专用密码
# QQ邮箱: 确认使用授权码（不是登录密码）
# 检查端口（587用TLS，465用SSL）
```

---

## 📝 配置文件完整示例

### 最小配置 .env
```env
# 数据库配置
DATABASE_URL=postgresql://postgres:mypassword@localhost:5432/blog_db

# JWT密钥（使用 openssl rand -base64 32 生成）
JWT_SECRET=Kx9vR2mN8pQ4wL7yF6hJ3sT5nC1dB0aEiH2jK5lM7nP0qR3sT

# 日志级别（可选）
RUST_LOG=info
```

### 完整配置 .env
```env
# ===================
# 核心配置（必填）
# ===================

# 数据库配置
DATABASE_URL=postgresql://postgres:mypassword@localhost:5432/blog_db

# JWT密钥（必须≥32字符）
JWT_SECRET=Kx9vR2mN8pQ4wL7yF6hJ3sT5nC1dB0aEiH2jK5lM7nP0qR3sT

# ===================
# 服务器配置（可选）
# ===================

# 日志级别
RUST_LOG=info

# 监听地址和端口
SERVER_HOST=127.0.0.1
SERVER_PORT=8080

# CORS配置（生产环境请指定具体域名）
ALLOWED_ORIGINS=*

# ===================
# 七牛云配置（可选）
# ===================

# 如果需要文件上传功能，请配置以下参数
QINIU_ACCESS_KEY=你的AccessKey
QINIU_SECRET_KEY=你的SecretKey
QINIU_BUCKET=blog-uploads
QINIU_DOMAIN=你的域名.bkt.clouddn.com

# ===================
# 邮件配置（可选）
# ===================

# 如果需要邮件功能，请配置以下参数
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-app-password
SMTP_FROM_NAME=我的博客系统
SMTP_FROM_EMAIL=noreply@myblog.com

# ===================
# 高级配置（可选）
# ===================

# 数据库连接池
DB_MAX_CONNECTIONS=100
DB_MIN_CONNECTIONS=5
DB_CONNECT_TIMEOUT=10
DB_IDLE_TIMEOUT=600
DB_MAX_LIFETIME=1800

# 文件上传
MAX_FILE_SIZE=10485760
TEMP_UPLOAD_DIR=./temp_uploads

# 会话
JWT_EXPIRATION=86400

# 权限缓存
PERM_CACHE_TTL=3600

# WebSocket
WS_HEARTBEAT_INTERVAL=5
WS_CLIENT_TIMEOUT=10

# 环境
ENVIRONMENT=development
DEBUG=true
ENABLE_SWAGGER=true
```

---

## 🎯 下一步

配置完成后，您可以：

1. **阅读API文档**: http://localhost:8080/swagger-ui/
2. **测试API接口**: 使用Postman或curl
3. **查看架构设计**: [ARCHITECTURE_OVERVIEW.md](./ARCHITECTURE_OVERVIEW.md)
4. **开始开发**: [开发指南](./README.md#开发指南)

---

## 📞 需要帮助？

- 📖 **详细配置说明**: [CONFIGURATION_GUIDE.md](./CONFIGURATION_GUIDE.md)
- 📚 **完整文档**: [DOCUMENTATION_INDEX.md](./DOCUMENTATION_INDEX.md)
- 🐛 **提交问题**: [创建Issue](../../issues)

---

<div align="center">

**快速开始，立即体验！**

⚡ 只需2分钟，即可运行核心功能

Made with ❤️ using Rust

</div>
