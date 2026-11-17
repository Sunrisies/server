-- 生产启动和部署前均可重复执行。记录已由数据库本身保证幂等。
CREATE TABLE IF NOT EXISTS post_views (
  id BIGSERIAL PRIMARY KEY,
  post_id INTEGER NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
  ip VARCHAR(45) NOT NULL,
  viewed_date DATE NOT NULL,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(post_id, ip, viewed_date)
);
CREATE INDEX IF NOT EXISTS post_views_daily_unique ON post_views(post_id, ip, viewed_date);

-- 文章 CSDN 阅读量：代码 entity 已有该字段，旧库缺列会导致所有 posts 查询报错。
ALTER TABLE posts ADD COLUMN IF NOT EXISTS csdn_view_count INTEGER NOT NULL DEFAULT 0;

-- 云剪贴板：频道与条目。历史库已存在时由 IF NOT EXISTS 跳过，owner_uuid 列随后兼容补齐。
CREATE TABLE IF NOT EXISTS clipboard_channels (
  id SERIAL PRIMARY KEY,
  name VARCHAR NOT NULL UNIQUE,
  password_hash VARCHAR NOT NULL,
  owner_uuid VARCHAR(64),
  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TABLE IF NOT EXISTS clipboard_entries (
  id BIGSERIAL PRIMARY KEY,
  uuid VARCHAR NOT NULL UNIQUE,
  channel_id INTEGER NOT NULL REFERENCES clipboard_channels(id) ON DELETE CASCADE,
  type VARCHAR NOT NULL,
  content TEXT,
  file_url VARCHAR,
  file_name VARCHAR,
  file_size BIGINT,
  mime_type VARCHAR,
  pinned BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS clipboard_entries_channel ON clipboard_entries(channel_id);

-- 跨设备文件盒：账号登录可重新打开自己创建的空间；历史频道继续兼容访问码。
ALTER TABLE clipboard_channels ADD COLUMN IF NOT EXISTS owner_uuid VARCHAR(64);
CREATE INDEX IF NOT EXISTS clipboard_channels_owner_uuid ON clipboard_channels(owner_uuid, updated_at DESC);
