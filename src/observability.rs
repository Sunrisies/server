//! Bounded, in-process Prometheus metrics. Never retain URLs, SQL text or bind values.
use actix_web::{
    Error, HttpResponse,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
    http::header::{HeaderName, HeaderValue},
    web,
};
use sea_orm::{DatabaseConnection, metric::Info};
use sha2::{Digest, Sha256};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    fmt::Write,
    future::{Ready, ready},
    net::SocketAddr,
    pin::Pin,
    rc::Rc,
    sync::{
        Arc, LazyLock, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};
use tracing::{
    Event, Metadata, Subscriber,
    field::{Field, Visit},
    span::{Attributes, Id, Record},
};

const BOUNDS_US: [u64; 15] = [
    100, 500, 1000, 3000, 5000, 10000, 25000, 50000, 100000, 250000, 500000, 1000000, 2500000,
    5000000, 10000000,
];
const LABEL_LIMIT: usize = 256;
static METRICS: LazyLock<Metrics> = LazyLock::new(Metrics::default);
tokio::task_local! { static REQUEST: RefCell<RequestStats>; }

#[derive(Default, Debug, Clone)]
struct RequestStats {
    sql_us: u64,
    acquire_us: u64,
    queries: u64,
    acquires: u64,
}

#[derive(Default)]
struct Histogram {
    buckets: [AtomicU64; 15],
    count: AtomicU64,
    sum_us: AtomicU64,
    max_us: AtomicU64,
}
impl Histogram {
    fn observe(&self, us: u64) {
        self.count.fetch_add(1, Ordering::Relaxed);
        self.sum_us.fetch_add(us, Ordering::Relaxed);
        self.max_us.fetch_max(us, Ordering::Relaxed);
        for (bound, bucket) in BOUNDS_US.iter().zip(&self.buckets) {
            if us <= *bound {
                bucket.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
    fn render(&self, out: &mut String, name: &str, labels: &str) {
        let comma = if labels.is_empty() { "" } else { "," };
        for (bound, bucket) in BOUNDS_US.iter().zip(&self.buckets) {
            let _ = writeln!(
                out,
                "{name}_seconds_bucket{{{labels}{comma}le=\"{}\"}} {}",
                *bound as f64 / 1e6,
                bucket.load(Ordering::Relaxed)
            );
        }
        let _ = writeln!(
            out,
            "{name}_seconds_bucket{{{labels}{comma}le=\"+Inf\"}} {}",
            self.count.load(Ordering::Relaxed)
        );
        for (suffix, value) in [
            ("count", self.count.load(Ordering::Relaxed) as f64),
            ("sum", self.sum_us.load(Ordering::Relaxed) as f64 / 1e6),
        ] {
            let _ = writeln!(out, "{name}_seconds_{suffix}{{{labels}}} {value}");
        }
        let _ = writeln!(
            out,
            "{name}_max_seconds{{{labels}}} {}",
            self.max_us.load(Ordering::Relaxed) as f64 / 1e6
        );
    }
}
#[derive(Default)]
struct RouteMetrics {
    duration: Histogram,
    sql: Histogram,
    acquire: Histogram,
    queries: AtomicU64,
}
#[derive(Default)]
struct QueryMetrics {
    duration: Histogram,
    failures: AtomicU64,
}
#[derive(Default)]
struct Metrics {
    routes: Mutex<BTreeMap<(String, String, u16), Arc<RouteMetrics>>>,
    queries: Mutex<BTreeMap<(String, String), Arc<QueryMetrics>>>,
    acquire: Histogram,
    worker_lag: Histogram,
    in_flight: AtomicU64,
    cancelled: AtomicU64,
    pool_size: AtomicU64,
    pool_idle: AtomicU64,
    pool_busy_peak: AtomicU64,
    pool_max: AtomicU64,
    log_window: AtomicU64,
    logs_in_window: AtomicU64,
    suppressed_logs: AtomicU64,
}
fn micros(duration: Duration) -> u64 {
    duration.as_micros().min(u64::MAX as u128) as u64
}
fn quote(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('"', "\\\"")
}
fn operation(sql: &str) -> &'static str {
    match sql
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_ascii_uppercase()
        .as_str()
    {
        "SELECT" => "select",
        "INSERT" => "insert",
        "UPDATE" => "update",
        "DELETE" => "delete",
        _ => "other",
    }
}
fn query_identity(sql: &str) -> (String, String) {
    // Hash the parameterized template; never format Statement (which expands bind values).
    let hash = Sha256::digest(sql.as_bytes());
    let fingerprint = hash[..8].iter().map(|byte| format!("{byte:02x}")).collect();
    (operation(sql).into(), fingerprint)
}
fn observe_query(info: &Info<'_>) {
    let elapsed = micros(info.elapsed);
    let _ = REQUEST.try_with(|stats| {
        let mut s = stats.borrow_mut();
        s.sql_us += elapsed;
        s.queries += 1;
    });
    let key = query_identity(&info.statement.sql);
    let metric = {
        let mut all = METRICS.queries.lock().unwrap_or_else(|e| e.into_inner());
        let key = if all.contains_key(&key) || all.len() < LABEL_LIMIT {
            key
        } else {
            ("other".into(), "overflow".into())
        };
        all.entry(key.clone()).or_default().clone()
    };
    metric.duration.observe(elapsed);
    if info.failed {
        metric.failures.fetch_add(1, Ordering::Relaxed);
    }
}
fn observe_acquire(seconds: f64) {
    if !seconds.is_finite() || seconds < 0.0 {
        return;
    }
    let us = micros(Duration::from_secs_f64(seconds.min(86400.0)));
    METRICS.acquire.observe(us);
    let _ = REQUEST.try_with(|stats| {
        let mut s = stats.borrow_mut();
        s.acquire_us += us;
        s.acquires += 1;
    });
}

/// Capture only SQLx's timing event, not SQLx query text or arbitrary tracing fields.
struct PoolTimingSubscriber;
impl Subscriber for PoolTimingSubscriber {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.target() == "sqlx::pool::acquire"
    }
    fn max_level_hint(&self) -> Option<tracing::metadata::LevelFilter> {
        Some(tracing::metadata::LevelFilter::INFO)
    }
    fn new_span(&self, _: &Attributes<'_>) -> Id {
        Id::from_u64(1)
    }
    fn record(&self, _: &Id, _: &Record<'_>) {}
    fn record_follows_from(&self, _: &Id, _: &Id) {}
    fn enter(&self, _: &Id) {}
    fn exit(&self, _: &Id) {}
    fn event(&self, event: &Event<'_>) {
        #[derive(Default)]
        struct Timing(Option<f64>);
        impl Visit for Timing {
            fn record_f64(&mut self, field: &Field, value: f64) {
                // SQLx 0.8 uses the misspelling "aquired" in this public event.
                if field.name() == "aquired_after_secs" {
                    self.0 = Some(value);
                }
            }
            fn record_debug(&mut self, _: &Field, _: &dyn std::fmt::Debug) {}
        }
        let mut timing = Timing::default();
        event.record(&mut timing);
        if let Some(value) = timing.0 {
            observe_acquire(value);
        }
    }
}
pub fn init_pool_timing() -> Result<(), tracing::subscriber::SetGlobalDefaultError> {
    tracing::subscriber::set_global_default(PoolTimingSubscriber)
}
pub fn instrument_database(db: &mut DatabaseConnection) {
    db.set_metric_callback(observe_query);
}

pub fn sample_pool(db: &DatabaseConnection) {
    if !matches!(db, DatabaseConnection::SqlxPostgresPoolConnection(_)) {
        return;
    }
    let pool = db.get_postgres_connection_pool().clone();
    METRICS.pool_max.store(
        pool.options().get_max_connections() as u64,
        Ordering::Relaxed,
    );
    actix_web::rt::spawn(async move {
        let mut timer = tokio::time::interval(Duration::from_millis(50));
        timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            timer.tick().await;
            if pool.is_closed() {
                break;
            }
            let size = pool.size() as u64;
            let idle = pool.num_idle() as u64;
            METRICS.pool_size.store(size, Ordering::Relaxed);
            METRICS.pool_idle.store(idle, Ordering::Relaxed);
            METRICS
                .pool_busy_peak
                .fetch_max(size.saturating_sub(idle), Ordering::Relaxed);
        }
    });
}
/// A per-worker timer reveals event-loop stalls; not a measurement of network latency.
pub fn start_worker_probe() {
    actix_web::rt::spawn(async {
        let period = Duration::from_millis(50);
        let mut interval = tokio::time::interval_at(tokio::time::Instant::now() + period, period);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            let scheduled = interval.tick().await;
            let delay = tokio::time::Instant::now().saturating_duration_since(scheduled);
            METRICS.worker_lag.observe(micros(delay));
        }
    });
}
fn allow_slow_log() -> bool {
    let second = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let old = METRICS.log_window.load(Ordering::Relaxed);
    if old != second
        && METRICS
            .log_window
            .compare_exchange(old, second, Ordering::Relaxed, Ordering::Relaxed)
            .is_ok()
    {
        METRICS.logs_in_window.store(0, Ordering::Relaxed);
    }
    if METRICS.logs_in_window.fetch_add(1, Ordering::Relaxed) < 5 {
        true
    } else {
        METRICS.suppressed_logs.fetch_add(1, Ordering::Relaxed);
        false
    }
}
struct ActiveRequest {
    completed: bool,
}
impl ActiveRequest {
    fn new() -> Self {
        METRICS.in_flight.fetch_add(1, Ordering::Relaxed);
        Self { completed: false }
    }
}
impl Drop for ActiveRequest {
    fn drop(&mut self) {
        METRICS.in_flight.fetch_sub(1, Ordering::Relaxed);
        if !self.completed {
            METRICS.cancelled.fetch_add(1, Ordering::Relaxed);
        }
    }
}

#[derive(Clone, Default)]
pub struct RequestMetrics;
impl<S, B> Transform<S, ServiceRequest> for RequestMetrics
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequestMetricsMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;
    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestMetricsMiddleware {
            service: Rc::new(service),
        }))
    }
}
pub struct RequestMetricsMiddleware<S> {
    service: Rc<S>,
}
impl<S, B> Service<ServiceRequest> for RequestMetricsMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;
    forward_ready!(service);
    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        Box::pin(async move {
            let mut active = ActiveRequest::new();
            let start = Instant::now();
            let request_id = uuid::Uuid::new_v4().to_string();
            let method = req.method().as_str().to_owned();
            let result = REQUEST.scope(RefCell::new(RequestStats::default()), async move {
                let response = service.call(req).await;
                let duration_us = micros(start.elapsed());
                let status = match &response { Ok(r) => r.status().as_u16(), Err(e) => e.as_response_error().status_code().as_u16() };
                // Only matched patterns become labels; never raw paths or query strings.
                let route = response.as_ref().ok().and_then(|r| r.request().match_pattern()).unwrap_or_else(|| "unmatched".into());
                let stats = REQUEST.with(|s| s.borrow().clone());
                let metric = {
                    let mut all = METRICS.routes.lock().unwrap_or_else(|e| e.into_inner());
                    let key = (method.clone(), route.clone(), status);
                    let key = if all.contains_key(&key) || all.len() < LABEL_LIMIT { key } else { ("OTHER".into(), "overflow".into(), 0) };
                    all.entry(key).or_default().clone()
                };
                metric.duration.observe(duration_us); metric.sql.observe(stats.sql_us); metric.acquire.observe(stats.acquire_us);
                metric.queries.fetch_add(stats.queries, Ordering::Relaxed);
                if duration_us >= 500000 && allow_slow_log() {
                    log::warn!(target: "performance", "{}", serde_json::json!({"event":"slow_request","request_id":request_id,"method":method,"route":route,"status":status,"duration_ms":duration_us as f64/1000.0,"db_acquire_ms":stats.acquire_us as f64/1000.0,"sql_ms":stats.sql_us as f64/1000.0,"queries":stats.queries,"acquires":stats.acquires}));
                }
                response.map(|mut r| {
                    r.headers_mut().insert(HeaderName::from_static("x-request-id"), HeaderValue::from_str(&request_id).expect("UUID header")); r
                })
            }).await;
            active.completed = true;
            result
        })
    }
}

pub fn render_metrics() -> String {
    let mut out = String::from(
        "# HTTP durations stop when response headers/body object is ready; exclude streaming/body transfer.\n# Pool acquire includes queueing, optional health check and new connection setup; successful acquires only.\n# SQL durations exclude pool acquire; sum of concurrent operations can exceed wall time.\n",
    );
    let routes = METRICS
        .routes
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect::<Vec<_>>();
    for ((method, route, status), metric) in routes {
        let labels = format!(
            "method=\"{}\",route=\"{}\",status=\"{status}\"",
            quote(&method),
            quote(&route)
        );
        metric
            .duration
            .render(&mut out, "backend_http_request_duration", &labels);
        metric
            .sql
            .render(&mut out, "backend_http_sql_time", &labels);
        metric
            .acquire
            .render(&mut out, "backend_http_db_acquire_time", &labels);
        let _ = writeln!(
            out,
            "backend_http_queries_total{{{labels}}} {}",
            metric.queries.load(Ordering::Relaxed)
        );
    }
    let queries = METRICS
        .queries
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect::<Vec<_>>();
    for ((operation, fingerprint), metric) in queries {
        let labels = format!("operation=\"{operation}\",fingerprint=\"{fingerprint}\"");
        metric
            .duration
            .render(&mut out, "backend_sql_duration", &labels);
        let _ = writeln!(
            out,
            "backend_sql_errors_total{{{labels}}} {}",
            metric.failures.load(Ordering::Relaxed)
        );
    }
    METRICS.acquire.render(&mut out, "backend_db_acquire", "");
    METRICS
        .worker_lag
        .render(&mut out, "backend_worker_event_loop_lag", "");
    for (name, value) in [
        ("backend_http_in_flight", &METRICS.in_flight),
        ("backend_http_cancelled_total", &METRICS.cancelled),
        ("backend_db_pool_size", &METRICS.pool_size),
        ("backend_db_pool_idle", &METRICS.pool_idle),
        ("backend_db_pool_busy_peak", &METRICS.pool_busy_peak),
        ("backend_db_pool_max", &METRICS.pool_max),
        (
            "backend_slow_logs_suppressed_total",
            &METRICS.suppressed_logs,
        ),
    ] {
        let _ = writeln!(out, "{name} {}", value.load(Ordering::Relaxed));
    }
    out
}

pub fn metrics_bind(value: &str) -> Result<SocketAddr, &'static str> {
    let addr: SocketAddr = value
        .parse()
        .map_err(|_| "METRICS_BIND must be a loopback IP:port")?;
    if !addr.ip().is_loopback() {
        return Err("METRICS_BIND must use a loopback address");
    }
    Ok(addr)
}
pub async fn serve_metrics(addr: SocketAddr) -> std::io::Result<actix_web::dev::Server> {
    Ok(actix_web::HttpServer::new(|| {
        actix_web::App::new().route(
            "/metrics",
            web::get().to(|| async {
                HttpResponse::Ok()
                    .insert_header(("Cache-Control", "no-store"))
                    .content_type("text/plain; version=0.0.4; charset=utf-8")
                    .body(render_metrics())
            }),
        )
    })
    .workers(1)
    .bind(addr)?
    .run())
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{App, test};
    use sea_orm::{ConnectionTrait, Statement};

    #[actix_web::test]
    async fn metrics_listener_rejects_public_addresses() {
        assert!(metrics_bind("0.0.0.0:9090").is_err());
        assert!(metrics_bind("[::]:9090").is_err());
        assert!(metrics_bind("127.0.0.1:9090").is_ok());
        assert!(metrics_bind("[::1]:9090").is_ok());
    }
    #[actix_web::test]
    async fn labels_do_not_contain_request_paths_queries_or_binds() {
        let app = test::init_service(App::new().wrap(RequestMetrics).route(
            "/test/{id}",
            web::get().to(|| async { HttpResponse::Ok().finish() }),
        ))
        .await;
        let response = test::call_service(
            &app,
            test::TestRequest::get()
                .uri("/test/private-marker?password=secret-marker")
                .to_request(),
        )
        .await;
        assert_eq!(response.status(), 200);
        assert!(response.headers().contains_key("x-request-id"));
        let stmt = Statement::from_sql_and_values(
            sea_orm::DbBackend::Postgres,
            "SELECT $1",
            ["sql-bind-secret".into()],
        );
        observe_query(&Info {
            elapsed: Duration::from_millis(2),
            statement: &stmt,
            failed: false,
        });
        let text = render_metrics();
        assert!(text.contains("/test/{id}"));
        for private in [
            "private-marker",
            "secret-marker",
            "sql-bind-secret",
            "SELECT",
        ] {
            assert!(!text.contains(private));
        }
    }
    #[actix_web::test]
    async fn failure_responses_are_counted_without_changing_status() {
        let app = test::init_service(App::new().wrap(RequestMetrics).route(
            "/test/denied",
            web::get().to(|| async {
                Err::<HttpResponse, _>(actix_web::error::ErrorUnauthorized("denied"))
            }),
        ))
        .await;
        let response = test::call_service(
            &app,
            test::TestRequest::get().uri("/test/denied").to_request(),
        )
        .await;
        assert_eq!(response.status(), 401);
        assert!(render_metrics().contains("route=\"/test/denied\",status=\"401\""));
    }

    #[actix_web::test]
    async fn pool_wait_is_separate_from_query_execution() {
        let _ = init_pool_timing();
        let db = crate::test_helpers::setup_test_db().await;
        let url = std::env::var("DATABASE_URL_TEST").unwrap();
        let pool = sea_orm::sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .acquire_time_level(log::LevelFilter::Info)
            .connect(&url)
            .await
            .unwrap();
        let mut instrumented =
            sea_orm::SqlxPostgresConnector::from_sqlx_postgres_pool(pool.clone());
        instrument_database(&mut instrumented);
        let held = pool.acquire().await.unwrap();
        REQUEST
            .scope(RefCell::new(RequestStats::default()), async {
                let query = instrumented.query_one(Statement::from_string(
                    sea_orm::DbBackend::Postgres,
                    "SELECT 1",
                ));
                let release = async {
                    tokio::time::sleep(Duration::from_millis(60)).await;
                    drop(held);
                };
                let (row, _) = tokio::join!(query, release);
                assert!(row.unwrap().is_some());
                let stats = REQUEST.with(|s| s.borrow().clone());
                assert_eq!(stats.queries, 1);
                assert!(stats.acquire_us >= 50000, "{stats:?}");
                assert!(stats.sql_us < stats.acquire_us, "{stats:?}");
            })
            .await;
        instrumented.close().await.unwrap();
        db.close().await.unwrap();
    }
}
