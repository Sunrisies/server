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
