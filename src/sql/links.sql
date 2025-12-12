-- 外部链接表
CREATE TABLE external_links (
    id SERIAL PRIMARY KEY,
    uuid CHAR(36) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    url TEXT NOT NULL,
    protocol VARCHAR(10) NOT NULL CHECK (protocol IN ('http', 'https')),
    icon_url TEXT,
    category VARCHAR(100) NOT NULL,
    tags JSONB NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

COMMENT ON TABLE external_links IS '第三方外部链接管理表';

COMMENT ON COLUMN external_links.uuid IS '链接唯一标识';

COMMENT ON COLUMN external_links.protocol IS '协议限定: http/https';

COMMENT ON COLUMN external_links.tags IS '分类标签(JSON数组)';

-- 创建必要的索引
CREATE INDEX idx_external_links_category ON external_links(category);

CREATE INDEX idx_external_links_tags_gin ON external_links USING GIN (tags jsonb_ops);
