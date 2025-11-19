# 博客系统架构图

## 系统整体架构

```mermaid
graph TB
    subgraph "客户端层"
        A[Web前端] --> B[移动端]
        A --> C[管理后台]
    end

    subgraph "API网关层"
        D[Actix-Web服务器]
    end

    subgraph "业务逻辑层"
        E[认证服务]
        F[用户服务]
        G[文章服务]
        H[分类服务]
        I[标签服务]
        J[聊天服务]
        K[文件服务]
    end

    subgraph "数据访问层"
        L[SeaORM]
    end

    subgraph "数据存储层"
        M[PostgreSQL数据库]
        N[七牛云存储]
    end

    subgraph "实时通信"
        O[WebSocket服务]
        P[SSE服务]
    end

    A --> D
    B --> D
    C --> D
    D --> E
    D --> F
    D --> G
    D --> H
    D --> I
    D --> J
    D --> K
    D --> O
    D --> P
    E --> L
    F --> L
    G --> L
    H --> L
    I --> L
    J --> L
    K --> N
    L --> M
    O --> J
    P --> E
```

## 核心模块关系图

```mermaid
graph LR
    subgraph "认证模块"
        A1[JWT认证]
        A2[权限中间件]
        A3[密码加密]
    end

    subgraph "用户模块"
        B1[用户注册]
        B2[用户登录]
        B3[用户管理]
        B4[角色权限]
    end

    subgraph "内容模块"
        C1[文章管理]
        C2[分类管理]
        C3[标签管理]
        C4[评论系统]
    end

    subgraph "通信模块"
        D1[WebSocket聊天]
        D2[服务器事件]
        D3[消息存储]
    end

    subgraph "存储模块"
        E1[数据库操作]
        E2[文件上传]
        E3[云存储]
    end

    A1 --> B2
    A2 --> B3
    A2 --> C1
    B1 --> B4
    B2 --> A1
    B3 --> B4
    C1 --> C2
    C1 --> C3
    D1 --> D3
    D2 --> A1
    E1 --> C1
    E2 --> E3
```

## 数据库设计图

```mermaid
erDiagram
    users ||--o{ posts : "作者拥有多篇文章"
    users ||--o{ user_roles : "用户拥有多个角色"
    roles ||--o{ user_roles : "角色被多个用户拥有"
    roles ||--o{ role_permissions : "角色拥有多个权限"
    permissions ||--o{ role_permissions : "权限被多个角色拥有"
    users ||--o{ user_permissions : "用户拥有特殊权限"
    permissions ||--o{ user_permissions : "权限被多个用户拥有"

    posts ||--o{ post_tags : "文章拥有多个标签"
    tags ||--o{ post_tags : "标签被多篇文章使用"
    categories ||--o{ posts : "分类包含多篇文章"

    rooms ||--o{ room_messages : "房间包含多条消息"

    users {
        int id PK
        string uuid UK
        string user_name UK
        string pass_word
        string email UK
        string image
        string phone UK
        string binding
        timestamp created_at
        timestamp updated_at
    }

    roles {
        int id PK
        string code UK
        string name
        text description
        boolean is_system
        timestamp created_at
    }

    permissions {
        int id PK
        string code UK
        string name
        text description
        string category
    }

    user_roles {
        int id PK
        int user_id FK
        int role_id FK
        boolean is_primary
        timestamp created_at
    }

    role_permissions {
        int id PK
        int role_id FK
        int permission_id FK
        timestamp created_at
    }

    user_permissions {
        int id PK
        int user_id FK
        int permission_id FK
        boolean granted
        timestamp expires_at
        timestamp created_at
    }

    posts {
        int id PK
        string uuid UK
        int author_id FK
        int category_id FK
        string title
        text summary
        text content
        text markdowncontent
        string cover_image
        smallint status
        boolean featured
        int view_count
        timestamp created_at
        timestamp updated_at
        timestamp published_at
        int size
    }

    categories {
        int id PK
        string name
        string slug UK
        text description
        timestamp created_at
        timestamp updated_at
    }

    tags {
        int id PK
        string name UK
        timestamp created_at
        timestamp updated_at
    }

    post_tags {
        int post_id FK
        int tag_id FK
    }

    rooms {
        int id PK
        string uuid
        string name
        text description
        int max_users
        timestamp created_at
        timestamp updated_at
    }

    room_messages {
        int id PK
        int room_id FK
        string message_type
        text content
        text file_url
        string file_name
        int file_size
        int retention_hours
        timestamp created_at
        timestamp expires_at
    }
```

## API架构图

```mermaid
graph TB
    subgraph "API网关"
        A[Actix-Web路由]
    end

    subgraph "认证API"
        B1[/api/v1/auth/register]
        B2[/api/v1/auth/login]
        B3[/api/v1/auth/email]
        B4[/api/v1/auth/phone]
        B5[/api/v1/auth/oauth]
    end

    subgraph "用户API"
        C1[/api/v1/users]
        C2[/api/v1/users/{uuid}]
    end

    subgraph "内容API"
        D1[/api/v1/posts]
        D2[/api/v1/posts/{uuid}]
        D3[/api/v1/categories]
        D4[/api/v1/categories/{id}]
        D5[/api/v1/tags]
        D6[/api/v1/tags/{id}]
    end

    subgraph "实时通信API"
        E1[/api/v1/ws]
        E2[/api/v1/sse/stream]
        E3[/api/v1/rooms]
        E4[/api/v1/rooms/{room_id}]
        E5[/api/v1/rooms/{room_id}/messages]
    end

    subgraph "文件API"
        F1[/api/v1/upload]
    end

    A --> B1
    A --> B2
    A --> B3
    A --> B4
    A --> B5
    A --> C1
    A --> C2
    A --> D1
    A --> D2
    A --> D3
    A --> D4
    A --> D5
    A --> D6
    A --> E1
    A --> E2
    A --> E3
    A --> E4
    A --> E5
    A --> F1
```
