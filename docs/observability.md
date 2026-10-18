# 本地后端性能监测

不改变业务路由和响应体。新增响应头 `X-Request-ID`；请求、数据库指标按进程累积，重启清零。

## 开启与查看

设置 `METRICS_BIND=127.0.0.1:9090`，然后正常启动 Rust 服务。默认不启动指标 HTTP 服务；仅接受回环 IP，不允许 `0.0.0.0` / `::`。Prometheus 可由同机采集，或通过 SSH 隧道读取。不要通过业务反向代理暴露指标。

```sh
curl http://127.0.0.1:9090/metrics
```

指标服务独立于业务监听端口，单独一个 worker，不访问数据库。50ms 周期采样池大小、空闲连接与忙连接峰值；每个业务 worker 的50ms计时器记录事件循环延迟。

## 指标语义

- `backend_http_request_duration_seconds_*`：按 method、已匹配的路由模板、HTTP status 聚合的响应准备时间。包含鉴权、数据库获取、查询、业务处理等；不包含连接接受前等待、网络传输、流式响应传输。鉴权提前拒绝/未匹配路由归为 `unmatched`。
- `backend_http_sql_time_seconds_*` / `backend_http_db_acquire_time_seconds_*`：单请求 SQL 与成功获取连接耗时之和，用于与整个请求比较。
- `backend_http_queries_total`：请求内 SeaORM 已计时查询数。
- `backend_http_in_flight` / `backend_http_cancelled_total`：正在处理数与未完成即取消的请求数。
- `backend_db_acquire_seconds_*`：SQLx 成功 acquire 耗时，包含排队、配置启用时的连接健康检查，以及建立新连接；不是纯粹的排队时间。依赖当前锁定的 SQLx 0.8.6 timing event，升级时必须复测。
- `backend_sql_duration_seconds_*`：SeaORM 在成功 acquire 之后的语句执行/结果获取耗时，包含应用到数据库往返，不等于 PostgreSQL 内部CPU执行时间。按操作类型与SQL模板SHA256摘要聚合，绝不输出SQL正文或绑定参数。
- `backend_sql_errors_total`：语句执行失败数。连接获取失败不触发 SeaORM 语句回调，会体现在请求错误中。
- `backend_db_pool_size` / `backend_db_pool_idle` / `backend_db_pool_max` / `backend_db_pool_busy_peak`：当前连接数、空闲数、配置上限、进程生命周期内采样忙连接峰值。峰值为采样值，不是精确每时刻峰值。
- `backend_worker_event_loop_lag_seconds_*`：业务worker计时器调度延迟，用于发现同步工作、日志等造成的线程阻塞；不是客户端网络延迟。
- `backend_slow_logs_suppressed_total`：被限频抑制的慢请求日志数。

直方图包含累积 bucket/count/sum，单位秒；`*_max_seconds` 为进程生命周期最大值。使用 Prometheus `rate()`，或两次采样做差计算阶段结果；不要用从启动累计的均值冒充某次压测均值。

注意：并行 SQL/acquire 耗时相加可能大于请求墙钟时间，不应一律将相加值当作可相减的互斥阶段。指标并发采样存在小量读取偏差。SQL回调覆盖prepared execute/query以及事务中SeaORM已计时语句，不保证覆盖unprepared SQL、事务BEGIN/COMMIT、原生SQLx、独立spawn的新task。当前GET压测走已计时路径。SQLite有SQL回调，但没有这里的PostgreSQL pool计时配置和采样。

## 慢请求日志与隐私

响应准备超过500ms时，以 `performance` target 输出JSON：request_id、method、路由模板、HTTP status、总耗时、连接获取耗时和SQL耗时、查询次数。每秒最多约5条，避免监测自身大量写日志。没有原始URL、查询参数、Cookie、正文、SQL语句或参数。请求与查询标签表各最多256个常规组合，加一个overflow组合，避免无限基数。

本次使用专门的tracing subscriber只采集SQLx pool时长，不输出其他tracing字段。若将来接入OpenTelemetry/完整tracing，需要把这一事件采集整合到统一subscriber，不要再安装第二个全局subscriber。

## 诊断用配置

数据库使用现有配置：`DATABASE_MAX_CONNECTIONS`（默认10）、`DATABASE_MIN_CONNECTIONS`（默认1）、`DATABASE_TEST_BEFORE_ACQUIRE`（默认true），不要误用旧示例中的 `DB_MAX_CONNECTIONS`。

为了做控制变量对照，增加以下日志配置，默认保持现状：

- `HTTP_ACCESS_LOG_ENABLED=false`：关闭逐请求访问日志；默认true。
- `CONNECTION_LOG_ENABLED=false`：关闭新连接信息日志；默认true。
- `LOG_ROLL_BYTES=1048576`：日志文件轮转阈值；未设置时保持原有10240字节。最小1024。

关闭访问日志只适合明确选择的部署或诊断，不影响监测指标和限频慢请求日志。`DATABASE_SQLX_LOGGING` 默认false；本次监测不需要启用它，避免全量SQL日志和潜在敏感数据泄露。

## 验证

全量测试需配置隔离的本地 `DATABASE_URL_TEST` 和测试 `JWT_SECRET`，串行运行。新增测试验证：占用唯一连接时acquire时长与SQL时长分离；401响应状态保持；路由模板标签不泄露请求参数和SQL绑定值；指标端口拒绝公网监听。

指标是诊断工具，不直接证明表结构有问题。先对照获取连接耗时、SQL耗时和线程延迟，再针对具体查询使用只读 `EXPLAIN (ANALYZE, BUFFERS)`。

## PromQL 示例

每个接口近5分钟P95：

```promql
histogram_quantile(0.95, sum by (le, route) (rate(backend_http_request_duration_seconds_bucket[5m])))
```

平均成功获取连接耗时（秒）：

```promql
sum(rate(backend_db_acquire_seconds_sum[5m])) / sum(rate(backend_db_acquire_seconds_count[5m]))
```

HTTP 5xx比例：

```promql
sum(rate(backend_http_request_duration_seconds_count{status=~"5.."}[5m])) / sum(rate(backend_http_request_duration_seconds_count[5m]))
```
