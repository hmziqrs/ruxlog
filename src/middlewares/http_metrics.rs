use axum::{extract::Request, middleware::Next, response::Response};
use opentelemetry::metrics::{Counter, Histogram};
use std::sync::Arc;
use std::time::Instant;

pub struct HttpMetrics {
    pub request_duration: Histogram<f64>,
    pub request_count: Counter<u64>,
    pub response_status: Counter<u64>,
}

impl HttpMetrics {
    pub fn new(meter: &opentelemetry::metrics::Meter) -> Self {
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

pub async fn track_metrics(request: Request, next: Next) -> Response {
    let meter = crate::utils::telemetry::global_meter();
    let start = Instant::now();
    let method = request.method().to_string();
    let path = request.uri().path().to_string();

    let metrics = Arc::new(HttpMetrics::new(&meter));

    metrics.request_count.add(
        1,
        &[
            opentelemetry::KeyValue::new("http.method", method.clone()),
            opentelemetry::KeyValue::new("http.route", path.clone()),
        ],
    );

    let response = next.run(request).await;

    let duration = start.elapsed().as_millis() as f64;
    let status = response.status().as_u16();

    metrics.request_duration.record(
        duration,
        &[
            opentelemetry::KeyValue::new("http.method", method.clone()),
            opentelemetry::KeyValue::new("http.route", path.clone()),
            opentelemetry::KeyValue::new("http.status_code", status.to_string()),
        ],
    );

    metrics.response_status.add(
        1,
        &[
            opentelemetry::KeyValue::new("http.method", method),
            opentelemetry::KeyValue::new("http.route", path),
            opentelemetry::KeyValue::new("http.status_code", status.to_string()),
        ],
    );

    response
}
