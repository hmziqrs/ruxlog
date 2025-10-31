use opentelemetry::global;
use opentelemetry::metrics::{Counter, Histogram, Meter};
use opentelemetry::trace::TracerProvider as _;
use opentelemetry::KeyValue;
use opentelemetry_otlp::{
    LogExporter, MetricExporter, SpanExporter, WithExportConfig, WithHttpConfig,
};
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::runtime;
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_sdk::{logs::LoggerProvider, Resource};
use std::collections::HashMap;
use std::env;
use std::sync::OnceLock;
use std::time::Duration;
use tracing::info;
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

        global::shutdown_tracer_provider();
    }
}

fn build_resource() -> Resource {
    let service_name = env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "ruxlog-api".to_string());
    let service_version =
        env::var("OTEL_SERVICE_VERSION").unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_string());
    let deployment_env =
        env::var("DEPLOYMENT_ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

    Resource::new(vec![
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::SERVICE_NAME,
            service_name,
        ),
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::SERVICE_VERSION,
            service_version,
        ),
        KeyValue::new("deployment.environment", deployment_env),
    ])
}

fn init_tracer(
    resource: Resource,
    endpoint: &str,
    headers: HashMap<String, String>,
) -> Result<TracerProvider, opentelemetry::trace::TraceError> {
    let exporter = SpanExporter::builder()
        .with_http()
        .with_endpoint(format!("{}/v1/traces", endpoint))
        .with_headers(headers)
        .build()?;

    let provider = TracerProvider::builder()
        .with_batch_exporter(exporter, runtime::Tokio)
        .with_resource(resource)
        .with_sampler(opentelemetry_sdk::trace::Sampler::AlwaysOn)
        .build();

    global::set_tracer_provider(provider.clone());

    Ok(provider)
}

fn init_metrics(
    resource: Resource,
    endpoint: &str,
    headers: HashMap<String, String>,
) -> Result<SdkMeterProvider, Box<dyn std::error::Error>> {
    let exporter = MetricExporter::builder()
        .with_http()
        .with_endpoint(format!("{}/v1/metrics", endpoint))
        .with_headers(headers)
        .build()?;

    let reader = opentelemetry_sdk::metrics::PeriodicReader::builder(exporter, runtime::Tokio)
        .with_interval(Duration::from_secs(30))
        .build();

    let provider = SdkMeterProvider::builder()
        .with_resource(resource)
        .with_reader(reader)
        .build();

    global::set_meter_provider(provider.clone());

    Ok(provider)
}

fn init_logs(
    resource: Resource,
    endpoint: &str,
    headers: HashMap<String, String>,
) -> Result<LoggerProvider, Box<dyn std::error::Error>> {
    let exporter = LogExporter::builder()
        .with_http()
        .with_endpoint(format!("{}/v1/logs", endpoint))
        .with_headers(headers)
        .build()?;

    let provider = LoggerProvider::builder()
        .with_resource(resource)
        .with_batch_exporter(exporter, runtime::Tokio)
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

    let otlp_endpoint = env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok();

    if let Some(endpoint) = otlp_endpoint {
        info!("Initializing OpenTelemetry with endpoint: {}", endpoint);

        let headers_str = env::var("OTEL_EXPORTER_OTLP_HEADERS").unwrap_or_default();
        let headers = parse_otlp_headers(&headers_str);

        let resource = build_resource();

        let tracer_provider = init_tracer(resource.clone(), &endpoint, headers.clone())
            .expect("Failed to initialize tracer");

        let meter_provider = init_metrics(resource.clone(), &endpoint, headers.clone())
            .expect("Failed to initialize metrics");

        let logger_provider = init_logs(resource.clone(), &endpoint, headers.clone())
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
        info!("OTEL_EXPORTER_OTLP_ENDPOINT not set, skipping OpenTelemetry initialization");

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
    _redis_gauge: opentelemetry::metrics::ObservableGauge<u64>,
    _db_gauge: opentelemetry::metrics::ObservableGauge<u64>,
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

pub fn http_metrics() -> &'static HttpMetrics {
    HTTP_METRICS.get_or_init(|| HttpMetrics::new(&global_meter()))
}

pub fn init_pool_metrics() {
    POOL_METRICS.get_or_init(|| PoolMetrics::new(&global_meter()));
}
