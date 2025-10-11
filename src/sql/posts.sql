-- 1️⃣ 分类表
CREATE TABLE categories (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    slug VARCHAR(100) NOT NULL,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (slug)
);

COMMENT ON TABLE categories IS '文章分类';

COMMENT ON COLUMN categories.id IS '主键';

COMMENT ON COLUMN categories.name IS '分类名称';

COMMENT ON COLUMN categories.slug IS 'URL 用英文标识';

-- INSERT INTO
--     categories (
--         id,
--         name,
--         slug,
--         description,
--         created_at,
--         updated_at
--     )
-- VALUES
--     (
--         1,
--         '运维',
--         'ops',
--         '服务器、CI/CD、云原生相关',
--         '2024-11-06 06:40:00',
--         '2024-11-06 06:40:00'
--     ),
--     (
--         2,
--         '前端',
--         'frontend',
--         'Web 前端、小程序、移动端',
--         '2024-11-06 06:40:05',
--         '2024-11-06 06:40:05'
--     ),
--     (
--         3,
--         '后端',
--         'backend',
--         'API、数据库、微服务',
--         '2024-11-06 06:40:09',
--         '2024-11-06 06:40:09'
--     ),
--     (
--         4,
--         '嵌入式',
--         'embedded',
--         '单片机、RTOS、物联网',
--         '2025-01-04 07:38:13',
--         '2025-01-04 07:38:13'
--     ),
--     (
--         5,
--         '其他',
--         'other',
--         '随笔、工具、未分类',
--         '2025-01-07 06:22:00',
--         '2025-01-07 06:22:00'
--     ) ON CONFLICT (id) DO NOTHING;
-- 2️⃣ 标签表
CREATE TABLE tags (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (name)
);

COMMENT ON TABLE tags IS '文章标签';

COMMENT ON COLUMN tags.name IS '标签名称';

-- 初始数据（MySQL → PostgreSQL 语法）
-- INSERT INTO
--     tags (name)
-- VALUES
--     ('JavaScript'),
--     ('TypeScript'),
--     ('React'),
--     ('Vue'),
--     ('Node.js'),
--     ('Express'),
--     ('Nest.js'),
--     ('MySQL'),
--     ('MongoDB'),
--     ('Redis'),
--     ('Docker'),
--     ('Kubernetes'),
--     ('CI/CD'),
--     ('Git'),
--     ('GitHub'),
--     ('CSS'),
--     ('HTML'),
--     ('Webpack'),
--     ('Vite'),
--     ('Next.js'),
--     ('Tauri'),
--     ('Cesium'),
--     ('Liunx'),
--     ('Rust'),
--     ('Nginx'),
--     ('Bug') ON CONFLICT (id) DO NOTHING;
-- 3️⃣ 文章表
CREATE TABLE posts (
    id SERIAL PRIMARY KEY,
    uuid CHAR(36) NOT NULL,
    author_id INT NOT NULL,
    category_id INT,
    title VARCHAR(255) NOT NULL,
    summary TEXT,
    content TEXT NOT NULL,
    markdowncontent TEXT NOT NULL,
    cover_image VARCHAR(255),
    status SMALLINT NOT NULL DEFAULT 0,
    -- 0 草稿 1 发布 2 下线
    featured BOOLEAN NOT NULL DEFAULT FALSE,
    view_count INT NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    published_at TIMESTAMP WITH TIME ZONE,
    UNIQUE (uuid),
    FOREIGN KEY (author_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE
    SET
        NULL
);

CREATE INDEX idx_posts_author ON posts(author_id);

CREATE INDEX idx_posts_category ON posts(category_id);

CREATE INDEX idx_posts_status ON posts(status);

CREATE INDEX idx_posts_published ON posts(published_at DESC);

COMMENT ON TABLE posts IS '文章主表';

COMMENT ON COLUMN posts.uuid IS '全局唯一标识';

COMMENT ON COLUMN posts.author_id IS '作者 FK → users.id';

COMMENT ON COLUMN posts.category_id IS '分类 FK → categories.id';

COMMENT ON COLUMN posts.status IS '0 草稿 1 发布 2 下线';

COMMENT ON COLUMN posts.featured IS '是否置顶推荐';

COMMENT ON COLUMN posts.view_count IS '浏览量';

COMMENT ON COLUMN posts.published_at IS '首次发布时间';

-- 4️⃣ 文章-标签 多对多中间表
CREATE TABLE post_tags (
    post_id INT NOT NULL,
    tag_id INT NOT NULL,
    PRIMARY KEY (post_id, tag_id),
    FOREIGN KEY (post_id) REFERENCES posts(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

CREATE INDEX idx_post_tags_tag ON post_tags(tag_id);

COMMENT ON TABLE post_tags IS '文章与标签的多对多关系';
