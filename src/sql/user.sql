-- 创建 users 表
CREATE TABLE users (
    -- 主键ID，自增
    id SERIAL PRIMARY KEY,
    -- 全局唯一标识符（字符串格式）
    uuid CHAR(36) NOT NULL,
    -- 用户名
    user_name VARCHAR(255) NOT NULL,
    -- 密码
    pass_word VARCHAR(255) NOT NULL,
    -- 电子邮箱
    email VARCHAR(255),
    -- 头像图片路径
    image VARCHAR(255),
    -- 手机号码
    phone VARCHAR(20),
    -- 用户角色
    role VARCHAR(50),
    -- 用户权限
    permissions TEXT,
    -- 绑定信息
    binding VARCHAR(255),
    -- 创建时间，默认为当前时间
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    -- 更新时间，默认为当前时间
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    -- 确保 uuid 是唯一的
    UNIQUE (uuid),
    -- 确保 email 是唯一的
    UNIQUE (email),
    -- 确保 user_name 是唯一的
    UNIQUE (user_name),
    -- 确保 phone 是唯一的
    UNIQUE (phone)
);

-- 为表添加注释
COMMENT ON TABLE users IS '用户信息表';
COMMENT ON COLUMN users.id IS '主键ID，自增';
COMMENT ON COLUMN users.uuid IS '全局唯一标识符（字符串格式）';
COMMENT ON COLUMN users.user_name IS '用户名';
COMMENT ON COLUMN users.pass_word IS '密码';
COMMENT ON COLUMN users.email IS '电子邮箱';
COMMENT ON COLUMN users.image IS '头像图片路径';
COMMENT ON COLUMN users.phone IS '手机号码';
COMMENT ON COLUMN users.role IS '用户角色';
COMMENT ON COLUMN users.permissions IS '用户权限';
COMMENT ON COLUMN users.binding IS '绑定信息';
COMMENT ON COLUMN users.created_at IS '创建时间，默认为当前时间';
COMMENT ON COLUMN users.updated_at IS '更新时间，默认为当前时间';
