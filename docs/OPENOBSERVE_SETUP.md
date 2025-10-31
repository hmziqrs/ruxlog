# OpenObserve + OpenTelemetry Setup

## Quick Start

### 1. Start OpenObserve
```sh
docker-compose -f docker-compose.observability.yml up -d
```

### 2. Configure Environment

**Local Setup:**
```sh
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:5080/api/default
export OTEL_EXPORTER_OTLP_HEADERS="Authorization=Basic cm9vdEBleGFtcGxlLmNvbTpDb21wbGV4cGFzcyMxMjM="
export OTEL_SERVICE_NAME=ruxlog-api
export DEPLOYMENT_ENVIRONMENT=development
```

**Remote Server (192.168.0.23):**
```sh
export OTEL_EXPORTER_OTLP_ENDPOINT=http://192.168.0.23:5080/api/default
export OTEL_EXPORTER_OTLP_HEADERS="Authorization=Basic cm9vdEBleGFtcGxlLmNvbTpDb21wbGV4cGFzcyMxMjM="
export OTEL_SERVICE_NAME=ruxlog-api
export DEPLOYMENT_ENVIRONMENT=production
```

### 3. Run Application
```sh
cargo run
```

### 4. Access OpenObserve
- **Local**: http://localhost:5080
- **Remote**: http://192.168.0.23:5080
- Email: `root@example.com`
- Password: `Complexpass#123`

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `OTEL_EXPORTER_OTLP_ENDPOINT` | - | Base endpoint (e.g., `http://localhost:5080/api/default`) |
| `OTEL_EXPORTER_OTLP_HEADERS` | - | Auth headers (e.g., `Authorization=Basic <base64>`) |
| `OTEL_SERVICE_NAME` | `ruxlog-api` | Service identifier |
| `OTEL_SERVICE_VERSION` | From Cargo.toml | Service version |
| `DEPLOYMENT_ENVIRONMENT` | `development` | Deployment environment |
| `RUST_LOG` | `info` | Log level |

## Available Metrics

- **HTTP**: `http.server.duration`, request/response counters
- **Auth**: Login success/failure, session operations
- **Image**: Optimization, upload, resize operations  
- **Mail**: Send success/failure, queue depth
- **Abuse**: Rate limit hits, violations
- **Pool**: Active connections (DB, Redis, S3)

## Notes

- Telemetry gracefully disabled if `OTEL_EXPORTER_OTLP_ENDPOINT` not set
- OpenObserve auto-appends signal paths: `/v1/traces`, `/v1/metrics`, `/v1/logs`
- Basic auth format: `base64(email:password)` for default credentials above