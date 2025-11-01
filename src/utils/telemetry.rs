use opentelemetry::global;
use opentelemetry::metrics::{Counter, Histogram, Meter, ObservableGauge};
use opentelemetry::trace::TracerProvider as _;

use opentelemetry_otlp::{
    LogExporter, MetricExporter, SpanExporter, WithExportConfig, WithHttpConfig,
};
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::trace::SdkTracerProvider as TracerProvider;
use opentelemetry_sdk::{logs::SdkLoggerProvider as LoggerProvider, Resource};
use std::collections::HashMap;
use std::env;
use std::sync::OnceLock;
use std::time::Duration;
use tracing::{info, warn};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

pub struct TelemetryGuard {
    meter_provider: Option<SdkMeterProvider>,
}

impl Drop for TelemetryGuard {
    fn drop(&mut self) {
        tracing::info!("Shutting down OpenTelemetry pipelines");

        if let Some(provider) = self.meter_provider.take() {
            if let Err(err) = provider.shutdown() {
                eprintln!("Error shutting down meter provider: {:?}", err);
            }
        }
    }
}

fn env_u64(key: &str, default: u64) -> u64 {
    env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn env_f64(key: &str, default: f64) -> f64 {
    env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

struct TelemetryConfig {
    trace_batch_max_queue_size: usize,
    trace_batch_scheduled_delay_ms: u64,
    trace_batch_max_export_batch_size: usize,
    trace_batch_max_export_timeout_ms: u64,

    metrics_export_interval_ms: u64,
    metrics_export_timeout_ms: u64,

    logs_batch_max_queue_size: usize,
    logs_batch_scheduled_delay_ms: u64,
    logs_batch_max_export_batch_size: usize,
    logs_batch_max_export_timeout_ms: u64,

    trace_sample_ratio: f64,
}

impl TelemetryConfig {
    fn from_env() -> Self {
        Self {
            trace_batch_max_queue_size: env_u64("OTEL_BSP_MAX_QUEUE_SIZE", 2048) as usize,
            trace_batch_scheduled_delay_ms: env_u64("OTEL_BSP_SCHEDULE_DELAY", 5000),
            trace_batch_max_export_batch_size: env_u64("OTEL_BSP_MAX_EXPORT_BATCH_SIZE", 512)
                as usize,
            trace_batch_max_export_timeout_ms: env_u64("OTEL_BSP_EXPORT_TIMEOUT", 30000),

            metrics_export_interval_ms: env_u64("OTEL_METRIC_EXPORT_INTERVAL", 30000),
            metrics_export_timeout_ms: env_u64("OTEL_METRIC_EXPORT_TIMEOUT", 30000),

            logs_batch_max_queue_size: env_u64("OTEL_BLRP_MAX_QUEUE_SIZE", 2048) as usize,
            logs_batch_scheduled_delay_ms: env_u64("OTEL_BLRP_SCHEDULE_DELAY", 1000),
            logs_batch_max_export_batch_size: env_u64("OTEL_BLRP_MAX_EXPORT_BATCH_SIZE", 512)
                as usize,
            logs_batch_max_export_timeout_ms: env_u64("OTEL_BLRP_EXPORT_TIMEOUT", 30000),

            trace_sample_ratio: env_f64("OTEL_TRACES_SAMPLER_ARG", 1.0).clamp(0.0, 1.0),
        }
    }
}

fn build_resource() -> Resource {
    Resource::builder()
        .with_service_name(
            env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "ruxlog-api".to_string()),
        )
        .with_attribute(opentelemetry::KeyValue::new(
            "service.version",
            env::var("OTEL_SERVICE_VERSION").unwrap_or_else(|_| "unknown".to_string()),
        ))
        .with_attribute(opentelemetry::KeyValue::new(
            "deployment.environment",
            env::var("DEPLOYMENT_ENVIRONMENT").unwrap_or_else(|_| "development".to_string()),
        ))
        .build()
}

fn init_tracer(
    endpoint: &str,
    headers: HashMap<String, String>,
    config: &TelemetryConfig,
    resource: &Resource,
) -> Result<TracerProvider, Box<dyn std::error::Error>> {
    let trace_endpoint = format!("{}/api/v1/otlp/v1/traces", endpoint);
    eprintln!(
        "DEBUG: Initializing tracer with endpoint: {}",
        trace_endpoint
    );
    let exporter = SpanExporter::builder()
        .with_http()
        .with_endpoint(trace_endpoint)
        .with_headers(headers)
        .with_timeout(Duration::from_millis(
            config.trace_batch_max_export_timeout_ms,
        ))
        .build()?;

    let batch_config = opentelemetry_sdk::trace::BatchConfigBuilder::default()
        .with_max_queue_size(config.trace_batch_max_queue_size)
        .with_scheduled_delay(Duration::from_millis(config.trace_batch_scheduled_delay_ms))
        .with_max_export_batch_size(config.trace_batch_max_export_batch_size)
        .build();

    let batch_processor = opentelemetry_sdk::trace::BatchSpanProcessor::builder(exporter)
        .with_batch_config(batch_config)
        .build();

    let sampler = if config.trace_sample_ratio >= 1.0 {
        opentelemetry_sdk::trace::Sampler::AlwaysOn
    } else if config.trace_sample_ratio <= 0.0 {
        warn!("Trace sampling ratio is 0.0, using AlwaysOff sampler");
        opentelemetry_sdk::trace::Sampler::AlwaysOff
    } else {
        info!(
            "Using TraceIdRatioBased sampler with ratio: {}",
            config.trace_sample_ratio
        );
        opentelemetry_sdk::trace::Sampler::TraceIdRatioBased(config.trace_sample_ratio)
    };

    let provider = TracerProvider::builder()
        .with_span_processor(batch_processor)
        .with_sampler(sampler)
        .with_resource(resource.clone())
        .build();

    global::set_tracer_provider(provider.clone());

    Ok(provider)
}

fn init_metrics(
    endpoint: &str,
    headers: HashMap<String, String>,
    config: &TelemetryConfig,
    resource: &Resource,
) -> Result<SdkMeterProvider, Box<dyn std::error::Error>> {
    let metrics_endpoint = format!("{}/api/v1/otlp/v1/metrics", endpoint);
    eprintln!(
        "DEBUG: Initializing metrics with endpoint: {}",
        metrics_endpoint
    );
    let exporter = MetricExporter::builder()
        .with_http()
        .with_endpoint(metrics_endpoint)
        .with_headers(headers)
        .with_timeout(Duration::from_millis(config.metrics_export_timeout_ms))
        .build()?;

    let reader = opentelemetry_sdk::metrics::PeriodicReader::builder(exporter)
        .with_interval(Duration::from_millis(config.metrics_export_interval_ms))
        .build();

    let provider = SdkMeterProvider::builder()
        .with_reader(reader)
        .with_resource(resource.clone())
        .build();

    global::set_meter_provider(provider.clone());

    Ok(provider)
}

fn init_logs(
    endpoint: &str,
    headers: HashMap<String, String>,
    config: &TelemetryConfig,
    resource: &Resource,
) -> Result<LoggerProvider, Box<dyn std::error::Error>> {
    let logs_endpoint = format!("{}/api/v1/otlp/v1/logs", endpoint);
    eprintln!("DEBUG: Initializing logs with endpoint: {}", logs_endpoint);
    let exporter = LogExporter::builder()
        .with_http()
        .with_endpoint(logs_endpoint)
        .with_headers(headers)
        .with_timeout(Duration::from_millis(
            config.logs_batch_max_export_timeout_ms,
        ))
        .build()?;

    let batch_config = opentelemetry_sdk::logs::BatchConfigBuilder::default()
        .with_max_queue_size(config.logs_batch_max_queue_size)
        .with_scheduled_delay(Duration::from_millis(config.logs_batch_scheduled_delay_ms))
        .with_max_export_batch_size(config.logs_batch_max_export_batch_size)
        .build();

    let batch_processor = opentelemetry_sdk::logs::BatchLogProcessor::builder(exporter)
        .with_batch_config(batch_config)
        .build();

    let provider = LoggerProvider::builder()
        .with_log_processor(batch_processor)
        .with_resource(resource.clone())
        .build();

    Ok(provider)
}

fn parse_otlp_headers(headers_str: &str) -> HashMap<String, String> {
    headers_str
        .split(',')
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            let key = parts.next()?.trim();
            let value = parts.next()?.trim();
            Some((key.to_string(), value.to_string()))
        })
        .collect()
}

pub fn init() -> TelemetryGuard {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_line_number(true)
        .with_filter(env_filter);

    if false {
        let endpoint = env::var("QUICKWIT_INGEST_URL")
            .unwrap_or_else(|_| "http://localhost:7280".to_string())
            .trim_end_matches('/')
            .to_string();

        info!(
            "Initializing OpenTelemetry with Quickwit ingest endpoint: {}",
            endpoint
        );

        let config = TelemetryConfig::from_env();

        info!(
            trace_queue_size = config.trace_batch_max_queue_size,
            trace_delay_ms = config.trace_batch_scheduled_delay_ms,
            trace_batch_size = config.trace_batch_max_export_batch_size,
            metrics_interval_ms = config.metrics_export_interval_ms,
            sample_ratio = config.trace_sample_ratio,
            "OpenTelemetry configuration loaded"
        );

        let mut headers = HashMap::new();

        if let Ok(token) = env::var("QUICKWIT_ACCESS_TOKEN") {
            let trimmed = token.trim();
            if !trimmed.is_empty() {
                headers.insert("Authorization".to_string(), format!("Bearer {}", trimmed));
            }
        }

        if let Ok(headers_str) = env::var("OTEL_EXPORTER_OTLP_HEADERS") {
            headers.extend(parse_otlp_headers(&headers_str));
        }

        let resource = build_resource();

        let tracer_provider = init_tracer(&endpoint, headers.clone(), &config, &resource)
            .expect("Failed to initialize tracer");

        let meter_provider = init_metrics(&endpoint, headers.clone(), &config, &resource)
            .expect("Failed to initialize metrics");

        let logger_provider = init_logs(&endpoint, headers.clone(), &config, &resource)
            .expect("Failed to initialize logs");

        let tracer = tracer_provider.tracer("ruxlog-api");
        let otel_trace_layer = tracing_opentelemetry::layer().with_tracer(tracer);

        let otel_log_layer = opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge::new(
            &logger_provider,
        );

        tracing_subscriber::registry()
            .with(fmt_layer)
            .with(otel_trace_layer)
            .with(otel_log_layer)
            .init();

        info!("OpenTelemetry initialized successfully");

        TelemetryGuard {
            meter_provider: Some(meter_provider),
        }
    } else {
        info!("ENABLE_QUICKWIT_OTEL is disabled, skipping OpenTelemetry initialization");

        tracing_subscriber::registry().with(fmt_layer).init();

        TelemetryGuard {
            meter_provider: None,
        }
    }
}

pub fn global_meter() -> Meter {
    global::meter("ruxlog-api")
}

/// Shared HTTP metrics to avoid recreating on every request
pub struct HttpMetrics {
    pub request_duration: Histogram<f64>,
    pub request_count: Counter<u64>,
    pub response_status: Counter<u64>,
}

/// Shared observable gauges for pool metrics
pub struct PoolMetrics {
    _redis_gauge: ObservableGauge<u64>,
    _db_gauge: ObservableGauge<u64>,
}

impl PoolMetrics {
    pub fn new(meter: &Meter) -> Self {
        let redis_gauge = meter
            .u64_observable_gauge("redis.pool.connections")
            .with_description("Number of active Redis pool connections")
            .build();

        let db_gauge = meter
            .u64_observable_gauge("db.pool.connections")
            .with_description("Number of active database pool connections")
            .build();

        Self {
            _redis_gauge: redis_gauge,
            _db_gauge: db_gauge,
        }
    }
}

/// Shared authentication metrics
pub struct AuthMetrics {
    pub login_attempts: Counter<u64>,
    pub login_success: Counter<u64>,
    pub login_failure: Counter<u64>,
    pub session_created: Counter<u64>,
    pub password_verification_duration: Histogram<f64>,
}

impl AuthMetrics {
    pub fn new(meter: &Meter) -> Self {
        Self {
            login_attempts: meter
                .u64_counter("auth.login.attempts")
                .with_description("Total login attempts")
                .build(),
            login_success: meter
                .u64_counter("auth.login.success")
                .with_description("Successful login attempts")
                .build(),
            login_failure: meter
                .u64_counter("auth.login.failure")
                .with_description("Failed login attempts")
                .build(),
            session_created: meter
                .u64_counter("auth.session.created")
                .with_description("Total sessions created")
                .build(),
            password_verification_duration: meter
                .f64_histogram("auth.password.verification.duration")
                .with_description("Password verification duration in milliseconds")
                .with_unit("ms")
                .build(),
        }
    }
}

/// Shared image optimization metrics
pub struct ImageMetrics {
    pub optimization_requests: Counter<u64>,
    pub optimization_success: Counter<u64>,
    pub optimization_skipped: Counter<u64>,
    pub bytes_saved: Counter<u64>,
    pub variants_generated: Counter<u64>,
    pub optimization_duration: Histogram<f64>,
}

impl ImageMetrics {
    pub fn new(meter: &Meter) -> Self {
        Self {
            optimization_requests: meter
                .u64_counter("image.optimization.requests")
                .with_description("Total image optimization requests")
                .build(),
            optimization_success: meter
                .u64_counter("image.optimization.success")
                .with_description("Successful optimizations")
                .build(),
            optimization_skipped: meter
                .u64_counter("image.optimization.skipped")
                .with_description("Optimizations skipped")
                .build(),
            bytes_saved: meter
                .u64_counter("image.optimization.bytes_saved")
                .with_description("Total bytes saved through optimization")
                .build(),
            variants_generated: meter
                .u64_counter("image.optimization.variants")
                .with_description("Image variants generated")
                .build(),
            optimization_duration: meter
                .f64_histogram("image.optimization.duration")
                .with_description("Image optimization duration in milliseconds")
                .with_unit("ms")
                .build(),
        }
    }
}

/// Shared abuse limiter metrics
pub struct LimiterMetrics {
    pub checks: Counter<u64>,
    pub allowed: Counter<u64>,
    pub blocked: Counter<u64>,
    pub temp_blocks: Counter<u64>,
    pub long_blocks: Counter<u64>,
}

impl LimiterMetrics {
    pub fn new(meter: &Meter) -> Self {
        Self {
            checks: meter
                .u64_counter("limiter.checks")
                .with_description("Total limiter checks")
                .build(),
            allowed: meter
                .u64_counter("limiter.allowed")
                .with_description("Requests allowed by limiter")
                .build(),
            blocked: meter
                .u64_counter("limiter.blocked")
                .with_description("Requests blocked by limiter")
                .build(),
            temp_blocks: meter
                .u64_counter("limiter.blocked.temp")
                .with_description("Temporary blocks issued")
                .build(),
            long_blocks: meter
                .u64_counter("limiter.blocked.long")
                .with_description("Long-term blocks issued")
                .build(),
        }
    }
}

/// Shared mail service metrics
pub struct MailMetrics {
    pub emails_sent: Counter<u64>,
    pub emails_failed: Counter<u64>,
    pub send_duration: Histogram<f64>,
}

impl MailMetrics {
    pub fn new(meter: &Meter) -> Self {
        Self {
            emails_sent: meter
                .u64_counter("mail.sent")
                .with_description("Total emails sent successfully")
                .build(),
            emails_failed: meter
                .u64_counter("mail.failed")
                .with_description("Total emails failed to send")
                .build(),
            send_duration: meter
                .f64_histogram("mail.send.duration")
                .with_description("Email send duration in milliseconds")
                .with_unit("ms")
                .build(),
        }
    }
}

impl HttpMetrics {
    pub fn new(meter: &Meter) -> Self {
        let request_duration = meter
            .f64_histogram("http.server.duration")
            .with_description("HTTP request duration in milliseconds")
            .with_unit("ms")
            .build();

        let request_count = meter
            .u64_counter("http.server.request.count")
            .with_description("Total number of HTTP requests")
            .build();

        let response_status = meter
            .u64_counter("http.server.response.status")
            .with_description("HTTP response status codes")
            .build();

        Self {
            request_duration,
            request_count,
            response_status,
        }
    }
}

static HTTP_METRICS: OnceLock<HttpMetrics> = OnceLock::new();
static POOL_METRICS: OnceLock<PoolMetrics> = OnceLock::new();
static AUTH_METRICS: OnceLock<AuthMetrics> = OnceLock::new();
static IMAGE_METRICS: OnceLock<ImageMetrics> = OnceLock::new();
static LIMITER_METRICS: OnceLock<LimiterMetrics> = OnceLock::new();
static MAIL_METRICS: OnceLock<MailMetrics> = OnceLock::new();

pub fn http_metrics() -> &'static HttpMetrics {
    HTTP_METRICS.get_or_init(|| HttpMetrics::new(&global_meter()))
}

pub fn auth_metrics() -> &'static AuthMetrics {
    AUTH_METRICS.get_or_init(|| AuthMetrics::new(&global_meter()))
}

pub fn image_metrics() -> &'static ImageMetrics {
    IMAGE_METRICS.get_or_init(|| ImageMetrics::new(&global_meter()))
}

pub fn limiter_metrics() -> &'static LimiterMetrics {
    LIMITER_METRICS.get_or_init(|| LimiterMetrics::new(&global_meter()))
}

pub fn mail_metrics() -> &'static MailMetrics {
    MAIL_METRICS.get_or_init(|| MailMetrics::new(&global_meter()))
}

pub fn init_pool_metrics() {
    POOL_METRICS.get_or_init(|| PoolMetrics::new(&global_meter()));
}
