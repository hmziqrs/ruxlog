# Observability API - Usage Guide

Complete guide for querying logs, metrics, and traces from your Ruxlog dashboard.

## Quick Start

### 1. Enable OpenObserve

**Local Development:**
```bash
docker-compose -f docker-compose.observability.yml up -d
```

**Production:** Point to your OpenObserve instance
```bash
OTEL_EXPORTER_OTLP_ENDPOINT=https://openobserve.yourcompany.com/api/default
OTEL_EXPORTER_OTLP_HEADERS=Authorization=Basic <base64-encoded-credentials>
```

### 2. Login as Admin

All observability endpoints require **Admin** role.

```bash
curl -X POST http://localhost:3000/auth/v1/log_in \
  -H "Content-Type: application/json" \
  -d '{
    "email": "admin@example.com",
    "password": "your-admin-password"
  }'
```

### 3. Check Observability Health

```bash
curl -X POST http://localhost:3000/observability/v1/health \
  -H "Cookie: your-session-cookie"
```

**Expected Response:**
```json
{
  "observability": "enabled",
  "backend": "openobserve"
}
```

---

## Common Use Cases

### 1. Dashboard: Error Rate (Last 24h)

**Endpoint:** `/observability/v1/stats/errors`

```bash
curl -X POST http://localhost:3000/observability/v1/stats/errors \
  -H "Content-Type: application/json" \
  -H "Cookie: session=..." \
  -d '{
    "hours_ago": 24,
    "top_n": 10
  }'
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

**Use in Dashboard:**
- Show top 10 failing endpoints
- Calculate error rate: `error_count / total_requests`
- Alert if error rate > 5%

---

### 2. Dashboard: Request Latency (p95, p99)

**Endpoint:** `/observability/v1/stats/latency`

```bash
curl -X POST http://localhost:3000/observability/v1/stats/latency \
  -H "Content-Type: application/json" \
  -H "Cookie: session=..." \
  -d '{
    "hours_ago": 1
  }'
```

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
    },
    {
      "http_route": "/post/v1/list/query",
      "request_count": 3421,
      "avg_latency_ms": 23.8,
      "min_latency_ms": 8.3,
      "max_latency_ms": 234.1
    }
  ],
  "total": 2,
  "took_ms": 28
}
```

**Use in Dashboard:**
- Show slowest endpoints
- Graph avg_latency_ms over time
- Alert if max_latency_ms > 1000ms

---

### 3. Dashboard: Authentication Events

**Endpoint:** `/observability/v1/stats/auth`

```bash
curl -X POST http://localhost:3000/observability/v1/stats/auth \
  -H "Cookie: session=..."
```

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
    },
    {
      "event_type": "auth.logout",
      "count": 298
    }
  ],
  "total": 4,
  "took_ms": 19
}
```

**Use in Dashboard:**
- Show login success rate: `success / (success + failed)`
- Alert on failed login spike
- Track 2FA adoption rate

---

### 4. Dashboard: Recent Critical Errors

**Endpoint:** `/observability/v1/logs/recent`

```bash
curl -X POST http://localhost:3000/observability/v1/logs/recent \
  -H "Content-Type: application/json" \
  -H "Cookie: session=..." \
  -d '{
    "level": "ERROR",
    "limit": 50,
    "hours_ago": 1
  }'
```

**Response:**
```json
{
  "data": [
    {
      "_timestamp": 1674789786006000,
      "level": "ERROR",
      "message": "Database connection timeout",
      "http_route": "/post/v1/create",
      "http_method": "POST",
      "user_id": 123,
      "trace_id": "abc123..."
    }
  ],
  "total": 23,
  "took_ms": 18
}
```

**Use in Dashboard:**
- Live feed of errors
- Link trace_id to distributed traces
- Show user impact (user_id)

---

### 5. Advanced: Custom SQL Query

**Endpoint:** `/observability/v1/logs/search`

**Example: Find slow database queries**
```bash
curl -X POST http://localhost:3000/observability/v1/logs/search \
  -H "Content-Type: application/json" \
  -H "Cookie: session=..." \
  -d '{
    "sql": "SELECT db_query, duration_ms, _timestamp FROM {stream} WHERE db_query IS NOT NULL AND duration_ms > 100 ORDER BY duration_ms DESC",
    "start_time": 1674789786006000,
    "end_time": 1674879786006000,
    "size": 20
  }'
```

**Example: Find users hitting rate limits**
```bash
curl -X POST http://localhost:3000/observability/v1/logs/search \
  -H "Content-Type: application/json" \
  -H "Cookie: session=..." \
  -d '{
    "sql": "SELECT user_id, client_ip, COUNT(*) as violations FROM {stream} WHERE event_type = 'rate_limit.exceeded' GROUP BY user_id, client_ip ORDER BY violations DESC",
    "hours_ago": 24,
    "size": 50
  }'
```

**Example: Image optimization stats**
```bash
curl -X POST http://localhost:3000/observability/v1/logs/search \
  -H "Content-Type: application/json" \
  -H "Cookie: session=..." \
  -d '{
    "sql": "SELECT AVG(original_size_kb) as avg_original, AVG(optimized_size_kb) as avg_optimized, COUNT(*) as count FROM {stream} WHERE event_type = 'image.optimized'",
    "hours_ago": 168
  }'
```

---

## Dashboard Widget Examples

### Widget 1: Error Rate Gauge

**Data Source:** `/observability/v1/stats/errors`

```javascript
// Fetch every 60 seconds
const errorStats = await fetch('/observability/v1/stats/errors', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ hours_ago: 1, top_n: 100 })
});

const data = await errorStats.json();
const totalErrors = data.data.reduce((sum, item) => sum + item.error_count, 0);

// Display as gauge: 0-50 green, 50-100 yellow, 100+ red
renderGauge(totalErrors, { max: 200, thresholds: [50, 100] });
```

### Widget 2: Latency Heatmap

**Data Source:** `/observability/v1/stats/latency`

```javascript
const latencyStats = await fetch('/observability/v1/stats/latency', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ hours_ago: 24 })
});

const data = await latencyStats.json();

// Transform to heatmap format
const heatmapData = data.data.map(route => ({
  route: route.http_route,
  requests: route.request_count,
  p50: route.avg_latency_ms,
  p95: route.max_latency_ms * 0.95, // Approximate
  color: route.avg_latency_ms > 100 ? 'red' : 'green'
}));

renderHeatmap(heatmapData);
```

### Widget 3: Live Error Feed

**Data Source:** `/observability/v1/logs/recent` (polling)

```javascript
// Poll every 10 seconds
setInterval(async () => {
  const recentErrors = await fetch('/observability/v1/logs/recent', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      level: 'ERROR',
      limit: 10,
      hours_ago: 1
    })
  });

  const data = await recentErrors.json();
  
  // Update feed (newest first)
  updateErrorFeed(data.data.map(log => ({
    timestamp: new Date(log._timestamp / 1000), // Convert from microseconds
    message: log.message,
    route: log.http_route,
    userId: log.user_id
  })));
}, 10000);
```

### Widget 4: Auth Success Rate (Donut Chart)

**Data Source:** `/observability/v1/stats/auth`

```javascript
const authStats = await fetch('/observability/v1/stats/auth', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' }
});

const data = await authStats.json();

const successCount = data.data.find(e => e.event_type === 'auth.login.success')?.count || 0;
const failedCount = data.data.find(e => e.event_type === 'auth.login.failed')?.count || 0;

const successRate = (successCount / (successCount + failedCount)) * 100;

renderDonutChart([
  { label: 'Success', value: successCount, color: 'green' },
  { label: 'Failed', value: failedCount, color: 'red' }
]);
```

---

## Time Range Helpers

### Convert to OpenObserve Microseconds

```javascript
// Last hour
const now = Date.now() * 1000; // Convert ms to microseconds
const oneHourAgo = (Date.now() - 3600000) * 1000;

// Last 24 hours
const twentyFourHoursAgo = (Date.now() - 86400000) * 1000;

// Last 7 days
const sevenDaysAgo = (Date.now() - 604800000) * 1000;

// Custom range
const customStart = new Date('2024-01-01T00:00:00Z').getTime() * 1000;
const customEnd = new Date('2024-01-31T23:59:59Z').getTime() * 1000;
```

### SQL Date Filters

```sql
-- Last hour (using _timestamp field)
SELECT * FROM {stream}
WHERE _timestamp >= now() - interval '1 hour'

-- Specific date range
SELECT * FROM {stream}
WHERE _timestamp BETWEEN 1674789786006000 AND 1674879786006000

-- Today only
SELECT * FROM {stream}
WHERE _timestamp >= date_trunc('day', now())
```

---

## Performance Tips

### 1. Use Time Ranges
Always specify `start_time` and `end_time` or `hours_ago`. Scanning all data is expensive.

```javascript
// ✅ Good - scoped query
{ hours_ago: 1, limit: 100 }

// ❌ Bad - scans everything
{ limit: 100 } // Defaults to 1 hour, but be explicit
```

### 2. Limit Result Size
Don't fetch more than you need.

```javascript
// ✅ Good - dashboard widget
{ size: 20, top_n: 10 }

// ❌ Bad - too much data
{ size: 10000 }
```

### 3. Cache Responses
Use Redis or in-memory cache for frequently accessed data.

```javascript
// Cache error stats for 60 seconds
const cacheKey = `observability:errors:${hoursAgo}`;
let data = await redis.get(cacheKey);

if (!data) {
  const response = await fetch('/observability/v1/stats/errors', ...);
  data = await response.json();
  await redis.setex(cacheKey, 60, JSON.stringify(data));
}
```

### 4. Use Aggregations
Let OpenObserve do the heavy lifting.

```sql
-- ✅ Good - aggregation in DB
SELECT http_route, COUNT(*) as count, AVG(duration_ms) as avg_latency
FROM {stream}
GROUP BY http_route

-- ❌ Bad - fetching all rows to aggregate client-side
SELECT http_route, duration_ms FROM {stream}
```

---

## Error Handling

### Check if Observability is Enabled

```javascript
const health = await fetch('/observability/v1/health', { method: 'POST' });
const status = await health.json();

if (status.observability === 'disabled') {
  console.warn('Observability not configured');
  showPlaceholder('Configure OTEL_EXPORTER_OTLP_ENDPOINT to enable');
  return;
}
```

### Handle API Errors

```javascript
try {
  const response = await fetch('/observability/v1/logs/search', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload)
  });

  if (response.status === 503) {
    // OpenObserve not configured
    showError('Observability service unavailable');
    return;
  }

  if (response.status === 403) {
    // Not admin
    showError('Admin access required');
    return;
  }

  if (!response.ok) {
    throw new Error(`HTTP ${response.status}`);
  }

  const data = await response.json();
  renderDashboard(data);

} catch (error) {
  console.error('Failed to fetch observability data:', error);
  showError('Failed to load metrics');
}
```

---

## Production Deployment

### Environment Variables

```bash
# Production OpenObserve endpoint
OTEL_EXPORTER_OTLP_ENDPOINT=https://openobserve.prod.yourcompany.com/api/production

# Generate secure credentials
echo -n "admin@yourcompany.com:SecurePassword123!" | base64
# Output: YWRtaW5AeW91cmNvbXBhbnkuY29tOlNlY3VyZVBhc3N3b3JkMTIzIQ==

OTEL_EXPORTER_OTLP_HEADERS=Authorization=Basic YWRtaW5AeW91cmNvbXBhbnkuY29tOlNlY3VyZVBhc3N3b3JkMTIzIQ==

# Service metadata
OTEL_SERVICE_NAME=ruxlog-api
DEPLOYMENT_ENVIRONMENT=production
```

### Security Considerations

1. **Admin-only access**: All endpoints protected by `user_permission::admin`
2. **No SQL injection**: Queries parameterized via OpenObserve
3. **HTTPS required**: Use TLS for production endpoints
4. **Rate limiting**: Apply to observability endpoints (expensive queries)

### Monitoring the Monitor

Add alerts for:
- OpenObserve API failures (circuit breaker pattern)
- Slow observability queries (> 5s)
- High scan sizes (> 100MB indicates inefficient query)

```javascript
if (data.took_ms > 5000) {
  logWarning('Slow observability query', { sql, took_ms: data.took_ms });
}

if (data.scan_size_mb > 100) {
  logWarning('Large scan size', { sql, scan_size: data.scan_size_mb });
}
```

---

## Next Steps

1. **Build Dashboard**: Create React/Vue components using examples above
2. **Add Caching**: Implement Redis caching for expensive queries
3. **Set Up Alerts**: Use OpenObserve alert rules for critical thresholds
4. **Optimize Queries**: Monitor `took_ms` and `scan_size_mb` in responses
5. **Export Data**: Add CSV/JSON export endpoints for reports

For more details, see:
- [OpenObserve API Docs](https://openobserve.ai/docs/api/)
- [Module README](../src/modules/observability_v1/README.md)
- [OpenTelemetry Setup](./OPENOBSERVE_SETUP.md)