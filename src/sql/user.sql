-- 如果当前表存在就删除
DROP TABLE IF EXISTS users;
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    uuid CHAR(36) NOT NULL,
    user_name VARCHAR(255) NOT NULL,
    pass_word VARCHAR(255) NOT NULL,
    email VARCHAR(255),
    image VARCHAR(255),
    phone VARCHAR(20),
    binding VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (uuid),
    UNIQUE (email),
    UNIQUE (user_name),
    UNIQUE (phone)
);
-- 如果存在删除
DROP TABLE IF EXISTS roles;
-- 角色表
CREATE TABLE roles (
    id SERIAL PRIMARY KEY,
    code VARCHAR(50) UNIQUE NOT NULL,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    is_system BOOLEAN DEFAULT FALSE,  -- 是否为系统内置角色
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

DROP TABLE IF EXISTS user_roles;
-- 用户角色关联表（支持多角色）
CREATE TABLE user_roles (
    id SERIAL PRIMARY KEY,
    user_id INT REFERENCES users(id) ON DELETE CASCADE,
    role_id INT REFERENCES roles(id) ON DELETE CASCADE,
    is_primary BOOLEAN DEFAULT FALSE,  -- 是否为主要角色
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, role_id)
);

DROP TABLE IF EXISTS permissions;
-- 权限表（基于操作而非路由）
CREATE TABLE permissions (
    id SERIAL PRIMARY KEY,
    code VARCHAR(100) UNIQUE NOT NULL,    -- 权限代码：post:create, user:delete等
    name VARCHAR(200) NOT NULL,
    description TEXT,
    category VARCHAR(50)                  -- 权限分类：content, user, system等
);

DROP TABLE IF EXISTS role_permissions;
-- 角色权限关联表
CREATE TABLE role_permissions (
    id SERIAL PRIMARY KEY,
    role_id INT REFERENCES roles(id) ON DELETE CASCADE,
    permission_id INT REFERENCES permissions(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(role_id, permission_id)
);

DROP TABLE IF EXISTS user_permissions;
-- 用户特殊权限表（覆盖角色权限）
CREATE TABLE user_permissions (
    id SERIAL PRIMARY KEY,
    user_id INT REFERENCES users(id) ON DELETE CASCADE,
    permission_id INT REFERENCES permissions(id) ON DELETE CASCADE,
    granted BOOLEAN DEFAULT TRUE,  -- TRUE表示授予，FALSE表示拒绝
    expires_at TIMESTAMP WITH TIME ZONE,  -- 权限过期时间
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- 初始化数据
INSERT INTO roles (code, name, description, is_system) VALUES
('SUPER_ADMIN', '超级管理员', '系统最高权限', TRUE),
('ADMIN', '管理员', '系统管理权限', TRUE),
('EDITOR', '编辑', '内容编辑权限', TRUE),
('AUTHOR', '作者', '文章作者权限', TRUE),
('VIEWER', '查看者', '只读权限', TRUE);

-- 初始化权限
INSERT INTO permissions (code, name, description, category) VALUES
-- 内容权限
('post:create', '创建文章', '创建新文章', 'content'),
('post:read', '查看文章', '查看文章内容', 'content'),
('post:update', '更新文章', '更新任何文章', 'content'),
('post:update:own', '更新自己的文章', '只能更新自己创建的文章', 'content'),
('post:delete', '删除文章', '删除任何文章', 'content'),
('post:delete:own', '删除自己的文章', '只能删除自己创建的文章', 'content'),
('post:publish', '发布文章', '发布或撤回文章', 'content'),

-- 用户权限
('user:read', '查看用户', '查看用户信息', 'user'),
('user:create', '创建用户', '创建新用户', 'user'),
('user:update', '更新用户', '更新用户信息', 'user'),
('user:delete', '删除用户', '删除用户账户', 'user'),

-- 系统权限
('system:settings', '系统设置', '修改系统设置', 'system'),
('system:backup', '系统备份', '执行系统备份', 'system');

-- 为角色分配权限
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.code = 'SUPER_ADMIN';

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.code = 'ADMIN' AND p.code NOT LIKE 'system:%';

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.code = 'EDITOR' AND p.category = 'content';

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.code = 'AUTHOR' AND (
    p.code LIKE 'post:create' OR
    p.code LIKE 'post:update:own' OR
    p.code LIKE 'post:delete:own' OR
    p.code LIKE 'post:read'
);

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.code = 'VIEWER' AND p.code = 'post:read';


-------------------- 表注释 --------------------
COMMENT ON TABLE users IS '用户基础信息表';
COMMENT ON TABLE roles IS '角色定义表（系统内置或自定义）';
COMMENT ON TABLE user_roles IS '用户-角色多对多关联（支持多角色）';
COMMENT ON TABLE permissions IS '权限定义表（基于操作颗粒度）';
COMMENT ON TABLE role_permissions IS '角色-权限多对多关联';
COMMENT ON TABLE user_permissions IS '用户特殊权限表（可覆盖角色权限，支持拒绝与过期）';

-------------------- users 字段注释 --------------------
COMMENT ON COLUMN users.id IS '主键';
COMMENT ON COLUMN users.uuid IS '全局唯一用户编号';
COMMENT ON COLUMN users.user_name IS '登录用户名';
COMMENT ON COLUMN users.pass_word IS '密码哈希（bcrypt/argon2）';
COMMENT ON COLUMN users.email IS '邮箱（可空，唯一）';
COMMENT ON COLUMN users.image IS '头像 URL（可空）';
COMMENT ON COLUMN users.phone IS '手机号（可空，唯一）';
COMMENT ON COLUMN users.binding IS '第三方绑定信息（可空，JSON）';
COMMENT ON COLUMN users.created_at IS '记录创建时间（UTC）';
COMMENT ON COLUMN users.updated_at IS '记录最后更新时间（UTC）';

-------------------- roles 字段注释 --------------------
COMMENT ON COLUMN roles.id IS '主键';
COMMENT ON COLUMN roles.code IS '角色代码（全局唯一，程序硬编码引用）';
COMMENT ON COLUMN roles.name IS '角色显示名称';
COMMENT ON COLUMN roles.description IS '角色描述';
COMMENT ON COLUMN roles.is_system IS '是否系统内置（禁止删除）';
COMMENT ON COLUMN roles.created_at IS '创建时间（UTC）';

-------------------- user_roles 字段注释 --------------------
COMMENT ON COLUMN user_roles.id IS '主键';
COMMENT ON COLUMN user_roles.user_id IS '用户外键';
COMMENT ON COLUMN user_roles.role_id IS '角色外键';
COMMENT ON COLUMN user_roles.is_primary IS '是否为主要角色（用于默认展示）';
COMMENT ON COLUMN user_roles.created_at IS '关联创建时间（UTC）';

-------------------- permissions 字段注释 --------------------
COMMENT ON COLUMN permissions.id IS '主键';
COMMENT ON COLUMN permissions.code IS '权限代码（全局唯一，程序硬编码引用）';
COMMENT ON COLUMN permissions.name IS '权限显示名称';
COMMENT ON COLUMN permissions.description IS '权限详细描述';
COMMENT ON COLUMN permissions.category IS '权限分类（content/user/system 等）';

-------------------- role_permissions 字段注释 --------------------
COMMENT ON COLUMN role_permissions.id IS '主键';
COMMENT ON COLUMN role_permissions.role_id IS '角色外键';
COMMENT ON COLUMN role_permissions.permission_id IS '权限外键';
COMMENT ON COLUMN role_permissions.created_at IS '关联创建时间（UTC）';

-------------------- user_permissions 字段注释 --------------------
COMMENT ON COLUMN user_permissions.id IS '主键';
COMMENT ON COLUMN user_permissions.user_id IS '用户外键';
COMMENT ON COLUMN user_permissions.permission_id IS '权限外键';
COMMENT ON COLUMN user_permissions.granted IS 'TRUE=授予，FALSE=拒绝（覆盖角色）';
COMMENT ON COLUMN user_permissions.expires_at IS '权限过期时间（NULL=永不过期）';
COMMENT ON COLUMN user_permissions.created_at IS '记录创建时间（UTC）';
