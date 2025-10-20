-- 房间表 - 存储聊天房间的基本信息
CREATE TABLE rooms (
    id VARCHAR(50) PRIMARY KEY,
    -- 房间ID，主键，用户自定义的唯一标识
    name VARCHAR(100) NOT NULL,
    -- 房间名称，必填
    description TEXT,
    -- 房间描述，可选
    max_users INTEGER DEFAULT 100,
    -- 房间最大用户数限制，默认100人
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    -- 房间创建时间，自动设置为当前时间
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() -- 房间最后更新时间，自动设置为当前时间
);

COMMENT ON TABLE rooms IS '聊天房间表，存储所有聊天房间的基本信息';

COMMENT ON COLUMN rooms.id IS '房间唯一标识符，用户自定义的字符串ID';

COMMENT ON COLUMN rooms.name IS '房间显示名称，用于界面展示';

COMMENT ON COLUMN rooms.description IS '房间描述信息，可选的详细说明';

COMMENT ON COLUMN rooms.max_users IS '房间允许的最大同时在线用户数';

COMMENT ON COLUMN rooms.created_at IS '房间创建时间戳';

COMMENT ON COLUMN rooms.updated_at IS '房间最后更新时间戳';

-- 房间消息表 - 存储所有房间内的聊天消息
CREATE TABLE room_messages (
    id SERIAL PRIMARY KEY,
    -- 消息ID，自增主键
    room_id VARCHAR(50) REFERENCES rooms(id) ON DELETE CASCADE,
    -- 外键，关联到房间表，级联删除
    message_type VARCHAR(20) NOT NULL CHECK (
        -- 消息类型，限制为指定类型
        message_type IN ('text', 'image', 'file', 'system')
    ),
    content TEXT,
    -- 消息内容，对于文本消息存储文本内容
    file_url TEXT,
    -- 文件URL，对于图片和文件消息存储文件地址
    file_name VARCHAR(255),
    -- 文件名，原始文件名
    file_size INTEGER,
    -- 文件大小，单位字节
    retention_hours INTEGER DEFAULT 24,
    -- 消息保留时间，单位小时，默认24小时
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    -- 消息创建时间
    expires_at TIMESTAMP WITH TIME ZONE -- 消息过期时间，根据retention_hours计算
);

COMMENT ON TABLE room_messages IS '房间消息表，存储所有聊天房间内的消息记录';

COMMENT ON COLUMN room_messages.id IS '消息唯一标识符，自增整数';

COMMENT ON COLUMN room_messages.room_id IS '外键，关联到所属的房间';

COMMENT ON COLUMN room_messages.message_type IS '消息类型：text-文本, image-图片, file-文件, system-系统消息';

COMMENT ON COLUMN room_messages.content IS '消息内容，文本消息存储具体内容，其他类型可为空';

COMMENT ON COLUMN room_messages.file_url IS '文件访问URL，对于图片和文件类型消息有效';

COMMENT ON COLUMN room_messages.file_name IS '原始文件名，用户上传时的文件名';

COMMENT ON COLUMN room_messages.file_size IS '文件大小，单位为字节';

COMMENT ON COLUMN room_messages.retention_hours IS '消息保留时长，单位小时，0表示永久保存';

COMMENT ON COLUMN room_messages.created_at IS '消息创建时间戳';

COMMENT ON COLUMN room_messages.expires_at IS '消息过期时间戳，用于自动清理过期消息';

-- 创建索引
-- 按房间和创建时间排序的索引，用于快速获取房间内的最新消息
CREATE INDEX idx_room_messages_room_created ON room_messages(room_id, created_at DESC);

COMMENT ON INDEX idx_room_messages_room_created IS '房间消息按房间ID和创建时间排序索引，优化消息查询性能';

-- 过期消息索引，用于快速查找和清理过期消息
CREATE INDEX idx_room_messages_expires ON room_messages(expires_at)
WHERE
    expires_at IS NOT NULL;

COMMENT ON INDEX idx_room_messages_expires IS '过期消息过滤索引，只对设置了过期时间的消息建立索引，优化清理操作';
