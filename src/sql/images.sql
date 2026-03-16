-- 图片表
CREATE TABLE images (
    id SERIAL PRIMARY KEY,
    url TEXT NOT NULL,
    key VARCHAR(255) NOT NULL,
    filename VARCHAR(255) NOT NULL,
    size BIGINT NOT NULL,
    human_readable_size VARCHAR(50) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

COMMENT ON TABLE images IS '图片管理表';

COMMENT ON COLUMN images.id IS '主键';

COMMENT ON COLUMN images.url IS '图片访问URL';

COMMENT ON COLUMN images.key IS '存储键值';

COMMENT ON COLUMN images.filename IS '原始文件名';

COMMENT ON COLUMN images.size IS '文件大小(字节)';

COMMENT ON COLUMN images.human_readable_size IS '可读的文件大小(如: 1.5MB)';

COMMENT ON COLUMN images.created_at IS '创建时间';

COMMENT ON COLUMN images.updated_at IS '更新时间';

-- 创建必要的索引
CREATE INDEX idx_images_key ON images(key);

CREATE INDEX idx_images_filename ON images(filename);
