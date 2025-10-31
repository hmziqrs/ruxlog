# Observability Module v1

Dashboard endpoints for querying OpenObserve logs, metrics, and traces.

## Features

- ✅ Search logs with custom SQL queries
- ✅ Fetch recent logs with filters (level, service, time range)
- ✅ Error statistics (top failing endpoints)
- ✅ Latency statistics (p50/p95/p99 by route)
- ✅ Authentication event tracking
- ✅ Metrics summaries with time-series aggregation
- ✅ Admin-only access via permission middleware

## Configuration

Set these environment variables to enable:

```bash
# OpenObserve endpoint (required to enable observability)
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:5080/api/default

# Base64 encoded "email:password"
OTEL_EXPORTER_OTLP_HEADERS=Authorization=Basic cm9vdEBleGFtcGxlLmNvbTpDb21wbGV4cGFzcyMxMjM=

# Optional: Service identification
OTEL_SERVICE_NAME=ruxlog-api
DEPLOYMENT_ENVIRONMENT=development
```

If `OTEL_EXPORTER_OTLP_ENDPOINT` is not set, all endpoints return `ServiceUnavailable`.

## API Endpoints

All routes require **Admin** role.

### Health Check
```http
POST /observability/v1/health
```

**Response:**
```json
{
  "observability": "enabled",
  "backend": "openobserve"
}
```

---

### Search Logs
```http
POST /observability/v1/logs/search
Content-Type: application/json

{
  "sql": "SELECT * FROM {stream} WHERE level = 'ERROR' ORDER BY _timestamp DESC",
  "start_time": 1674789786006000,
  "end_time": 1674799786006000,
  "from": 0,
  "size": 100,
  "stream": "default"
}
```

**Fields:**
- `sql` (optional): Custom SQL query. Use `{stream}` placeholder. Default: `SELECT * FROM {stream} ORDER BY _timestamp DESC`
- `start_time` (optional): Microseconds since epoch. Default: 1 hour ago
- `end_time` (optional): Microseconds since epoch. Default: now
- `from` (optional): Offset for pagination. Default: 0
- `size` (optional): Limit results (1-1000). Default: 100
- `stream` (optional): OpenObserve stream name. Default: `default`

**Response:**
```json
{
  "data": [
    {
      "_timestamp": 1674789786006000,
      "level": "ERROR",
      "message": "Database connection failed",
      "http_route": "/api/posts",
      "http_method": "POST"
    }
  ],
  "total": 142,
  "from": 0,
  "size": 100,
  "took_ms": 45,
  "scan_size_mb": 12
}
```

---

### Recent Logs
```http
POST /observability/v1/logs/recent
Content-Type: application/json

{
  "limit": 50,
  "level": "ERROR",
  "service": "ruxlog-api",
  "hours_ago": 24
}
```

**Fields:**
- `limit` (optional): Max results (1-1000). Default: 100
- `level` (optional): Filter by log level (ERROR, WARN, INFO, DEBUG)
- `service` (optional): Filter by service name
- `hours_ago` (optional): Time range in hours. Default: 1

**Response:**
```json
{
  "data": [...],
  "total": 23,
  "took_ms": 18
}
```

---

### Error Statistics
```http
POST /observability/v1/stats/errors
Content-Type: application/json

{
  "hours_ago": 24,
  "top_n": 20
}
```

**Response:**
```json
{
  "data": [
    {
      "http_route": "/post/v1/create",
      "http_method": "POST",
      "error_count": 142
    },
    {
      "http_route": "/auth/v1/log_in",
      "http_method": "POST",
      "error_count": 89
    }
  ],
  "total": 2,
  "took_ms": 32
}
```

---

### Latency Statistics
```http
POST /observability/v1/stats/latency
Content-Type: application/json

{
  "hours_ago": 24,
  "route": "/post/v1/create"
}
```

**Fields:**
- `hours_ago` (optional): Time range in hours. Default: 24
- `route` (optional): Filter by specific route

**Response:**
```json
{
  "data": [
    {
      "http_route": "/post/v1/create",
      "request_count": 1523,
      "avg_latency_ms": 45.2,
      "min_latency_ms": 12.1,
      "max_latency_ms": 892.4
    }
  ],
  "total": 1,
  "took_ms": 28
}
```

---

### Authentication Statistics
```http
POST /observability/v1/stats/auth
```

Aggregates auth events from the last 24 hours.

**Response:**
```json
{
  "data": [
    {
      "event_type": "auth.login.success",
      "count": 342
    },
    {
      "event_type": "auth.login.failed",
      "count": 23
    },
    {
      "event_type": "auth.2fa.verified",
      "count": 156
    }
  ],
  "total": 3,
  "took_ms": 19
}
```

---

### Metrics Summary
```http
POST /observability/v1/metrics/summary
Content-Type: application/json

{
  "hours_ago": 24,
  "metric_name": "http.server.duration"
}
```

**Fields:**
- `hours_ago` (optional): Time range in hours. Default: 24
- `metric_name` (optional): Specific metric to query. If omitted, returns all metrics grouped by name

**Response (with metric_name):**
```json
{
  "data": [
    {
      "time_bucket": 1674789600000000,
      "count": 1234,
      "avg_value": 42.5
    }
  ],
  "total": 288,
  "took_ms": 67
}
```

**Response (without metric_name):**
```json
{
  "data": [
    {
      "metric_name": "http.server.duration",
      "count": 45231
    },
    {
      "metric_name": "auth.login.attempts",
      "count": 892
    }
  ],
  "total": 2,
  "took_ms": 34
}
```

## SQL Query Examples

### Find slow requests (> 1 second)
```sql
SELECT http_route, http_method, duration_ms, _timestamp
FROM {stream}
WHERE duration_ms > 1000
ORDER BY duration_ms DESC
LIMIT 20
```

### Failed logins by IP
```sql
SELECT client_ip, COUNT(*) as attempts
FROM {stream}
WHERE event_type = 'auth.login.failed'
GROUP BY client_ip
ORDER BY attempts DESC
LIMIT 10
```

### Database errors in last hour
```sql
SELECT message, COUNT(*) as occurrences
FROM {stream}
WHERE str_match(message, 'database') AND level = 'ERROR'
GROUP BY message
ORDER BY occurrences DESC
```

### HTTP status codes distribution
```sql
SELECT http_status_code, COUNT(*) as count
FROM {stream}
WHERE http_status_code IS NOT NULL
GROUP BY http_status_code
ORDER BY count DESC
```

## Implementation Details

### Architecture
- **Service Layer**: `OpenObserveClient` handles HTTP communication with OpenObserve API
- **Controller Layer**: Handlers validate requests, build SQL, call service, format responses
- **Validator Layer**: DTOs with field validation and helper methods for SQL building

### Time Handling
- All timestamps in **microseconds** (OpenObserve default)
- Use `chrono::Utc::now().timestamp_micros()` for current time
- Time ranges default to sensible values (1h, 24h) if not provided

### Error Handling
- Returns `ServiceUnavailable` if OpenObserve not configured
- Returns `InternalServerError` for API/network failures
- Logs all errors with context (SQL, stream, time range)

### Security
- All endpoints protected by `user_permission::admin` middleware
- Only admins can query observability data
- No raw SQL injection possible (parameterized via OpenObserve)

### Caching
Not implemented yet. Consider adding Redis caching for:
- Recent error stats (TTL: 60s)
- Latency summaries (TTL: 5min)
- Auth stats (TTL: 5min)

## Development

**Start OpenObserve locally:**
```bash
docker-compose -f docker-compose.observability.yml up -d
```

**Access UI:**
- URL: http://localhost:5080
- Email: `root@example.com`
- Password: `Complexpass#123`

**Test endpoint:**
```bash
# Login as admin first
curl -X POST http://localhost:3000/auth/v1/log_in \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@example.com","password":"yourpass"}'

# Query recent errors
curl -X POST http://localhost:3000/observability/v1/logs/recent \
  -H "Content-Type: application/json" \
  -H "Cookie: your-session-cookie" \
  -d '{"level":"ERROR","hours_ago":1}'
```

## Future Enhancements

- [ ] Add Redis caching layer
- [ ] Support multiple streams/organizations
- [ ] Real-time log streaming via WebSockets
- [ ] Pre-built dashboard queries (common patterns)
- [ ] Export to CSV/JSON
- [ ] Alert rule management
- [ ] Trace correlation (logs → traces → metrics)