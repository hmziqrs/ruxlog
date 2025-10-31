#!/bin/bash

# Test script for Observability API endpoints
# Usage: ./scripts/test_observability.sh

set -e

BASE_URL="${BASE_URL:-http://localhost:3000}"
ADMIN_EMAIL="${ADMIN_EMAIL:-admin@example.com}"
ADMIN_PASSWORD="${ADMIN_PASSWORD:-your-admin-password}"

echo "🧪 Testing Observability API Endpoints"
echo "======================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Login as admin to get session cookie
echo -e "${YELLOW}1. Logging in as admin...${NC}"
LOGIN_RESPONSE=$(curl -s -c /tmp/ruxlog_cookies.txt -X POST "$BASE_URL/auth/v1/log_in" \
  -H "Content-Type: application/json" \
  -d "{\"email\":\"$ADMIN_EMAIL\",\"password\":\"$ADMIN_PASSWORD\"}")

if echo "$LOGIN_RESPONSE" | grep -q "error"; then
  echo -e "${RED}❌ Login failed. Make sure admin credentials are correct.${NC}"
  echo "$LOGIN_RESPONSE"
  exit 1
fi

echo -e "${GREEN}✅ Logged in successfully${NC}"
echo ""

# Test 1: Health Check
echo -e "${YELLOW}2. Testing health check...${NC}"
HEALTH_RESPONSE=$(curl -s -b /tmp/ruxlog_cookies.txt -X POST "$BASE_URL/observability/v1/health")
echo "$HEALTH_RESPONSE" | jq '.'

if echo "$HEALTH_RESPONSE" | grep -q "enabled"; then
  echo -e "${GREEN}✅ Observability is enabled${NC}"
elif echo "$HEALTH_RESPONSE" | grep -q "disabled"; then
  echo -e "${YELLOW}⚠️  Observability is disabled. Set OTEL_EXPORTER_OTLP_ENDPOINT to enable.${NC}"
  echo -e "${YELLOW}Skipping remaining tests.${NC}"
  rm -f /tmp/ruxlog_cookies.txt
  exit 0
else
  echo -e "${RED}❌ Unexpected health check response${NC}"
  rm -f /tmp/ruxlog_cookies.txt
  exit 1
fi
echo ""

# Calculate time ranges (microseconds)
NOW_MICROS=$(($(date +%s) * 1000000))
ONE_HOUR_AGO_MICROS=$(($NOW_MICROS - 3600000000))
TWENTY_FOUR_HOURS_AGO_MICROS=$(($NOW_MICROS - 86400000000))

# Test 2: Recent Logs
echo -e "${YELLOW}3. Testing recent logs (last hour, ERROR level)...${NC}"
RECENT_LOGS=$(curl -s -b /tmp/ruxlog_cookies.txt -X POST "$BASE_URL/observability/v1/logs/recent" \
  -H "Content-Type: application/json" \
  -d '{"level":"ERROR","limit":10,"hours_ago":1}')

echo "$RECENT_LOGS" | jq '{total: .total, took_ms: .took_ms, sample_count: (.data | length)}'

if echo "$RECENT_LOGS" | grep -q "total"; then
  echo -e "${GREEN}✅ Recent logs retrieved successfully${NC}"
else
  echo -e "${RED}❌ Failed to retrieve recent logs${NC}"
  echo "$RECENT_LOGS"
fi
echo ""

# Test 3: Search Logs with Custom SQL
echo -e "${YELLOW}4. Testing log search with custom SQL...${NC}"
SEARCH_LOGS=$(curl -s -b /tmp/ruxlog_cookies.txt -X POST "$BASE_URL/observability/v1/logs/search" \
  -H "Content-Type: application/json" \
  -d "{
    \"sql\": \"SELECT * FROM {stream} ORDER BY _timestamp DESC\",
    \"start_time\": $ONE_HOUR_AGO_MICROS,
    \"end_time\": $NOW_MICROS,
    \"from\": 0,
    \"size\": 10,
    \"stream\": \"default\"
  }")

echo "$SEARCH_LOGS" | jq '{total: .total, from: .from, size: .size, took_ms: .took_ms, scan_size_mb: .scan_size_mb}'

if echo "$SEARCH_LOGS" | grep -q "total"; then
  echo -e "${GREEN}✅ Log search completed successfully${NC}"
else
  echo -e "${RED}❌ Log search failed${NC}"
  echo "$SEARCH_LOGS"
fi
echo ""

# Test 4: Error Statistics
echo -e "${YELLOW}5. Testing error statistics (last 24h)...${NC}"
ERROR_STATS=$(curl -s -b /tmp/ruxlog_cookies.txt -X POST "$BASE_URL/observability/v1/stats/errors" \
  -H "Content-Type: application/json" \
  -d '{"hours_ago":24,"top_n":10}')

echo "$ERROR_STATS" | jq '{total: .total, took_ms: .took_ms, top_errors: (.data[:3] | length)}'

if echo "$ERROR_STATS" | grep -q "total"; then
  echo -e "${GREEN}✅ Error statistics retrieved successfully${NC}"

  # Show top 3 errors if available
  TOP_ERRORS=$(echo "$ERROR_STATS" | jq -r '.data[:3] | .[] | "\(.http_route) (\(.http_method)): \(.error_count) errors"' 2>/dev/null || echo "No errors found")
  if [ "$TOP_ERRORS" != "No errors found" ]; then
    echo "Top errors:"
    echo "$TOP_ERRORS"
  fi
else
  echo -e "${RED}❌ Failed to retrieve error statistics${NC}"
  echo "$ERROR_STATS"
fi
echo ""

# Test 5: Latency Statistics
echo -e "${YELLOW}6. Testing latency statistics (last 24h)...${NC}"
LATENCY_STATS=$(curl -s -b /tmp/ruxlog_cookies.txt -X POST "$BASE_URL/observability/v1/stats/latency" \
  -H "Content-Type: application/json" \
  -d '{"hours_ago":24}')

echo "$LATENCY_STATS" | jq '{total: .total, took_ms: .took_ms}'

if echo "$LATENCY_STATS" | grep -q "total"; then
  echo -e "${GREEN}✅ Latency statistics retrieved successfully${NC}"

  # Show top 3 slowest routes if available
  SLOW_ROUTES=$(echo "$LATENCY_STATS" | jq -r '.data[:3] | .[] | "\(.http_route): avg=\(.avg_latency_ms)ms, max=\(.max_latency_ms)ms"' 2>/dev/null || echo "No data available")
  if [ "$SLOW_ROUTES" != "No data available" ]; then
    echo "Top routes by latency:"
    echo "$SLOW_ROUTES"
  fi
else
  echo -e "${RED}❌ Failed to retrieve latency statistics${NC}"
  echo "$LATENCY_STATS"
fi
echo ""

# Test 6: Authentication Statistics
echo -e "${YELLOW}7. Testing authentication statistics (last 24h)...${NC}"
AUTH_STATS=$(curl -s -b /tmp/ruxlog_cookies.txt -X POST "$BASE_URL/observability/v1/stats/auth")

echo "$AUTH_STATS" | jq '{total: .total, took_ms: .took_ms}'

if echo "$AUTH_STATS" | grep -q "total"; then
  echo -e "${GREEN}✅ Authentication statistics retrieved successfully${NC}"

  # Show auth event breakdown if available
  AUTH_EVENTS=$(echo "$AUTH_STATS" | jq -r '.data | .[] | "\(.event_type): \(.count)"' 2>/dev/null || echo "No auth events found")
  if [ "$AUTH_EVENTS" != "No auth events found" ]; then
    echo "Authentication events:"
    echo "$AUTH_EVENTS"
  fi
else
  echo -e "${RED}❌ Failed to retrieve authentication statistics${NC}"
  echo "$AUTH_STATS"
fi
echo ""

# Test 7: Metrics Summary
echo -e "${YELLOW}8. Testing metrics summary (last 24h)...${NC}"
METRICS_SUMMARY=$(curl -s -b /tmp/ruxlog_cookies.txt -X POST "$BASE_URL/observability/v1/metrics/summary" \
  -H "Content-Type: application/json" \
  -d '{"hours_ago":24}')

echo "$METRICS_SUMMARY" | jq '{total: .total, took_ms: .took_ms}'

if echo "$METRICS_SUMMARY" | grep -q "total"; then
  echo -e "${GREEN}✅ Metrics summary retrieved successfully${NC}"

  # Show top metrics if available
  TOP_METRICS=$(echo "$METRICS_SUMMARY" | jq -r '.data[:5] | .[] | "\(.metric_name // .time_bucket): \(.count // .avg_value)"' 2>/dev/null || echo "No metrics found")
  if [ "$TOP_METRICS" != "No metrics found" ]; then
    echo "Top metrics:"
    echo "$TOP_METRICS"
  fi
else
  echo -e "${RED}❌ Failed to retrieve metrics summary${NC}"
  echo "$METRICS_SUMMARY"
fi
echo ""

# Cleanup
rm -f /tmp/ruxlog_cookies.txt

echo "======================================="
echo -e "${GREEN}✅ All tests completed!${NC}"
echo ""
echo "Tips:"
echo "  - View detailed responses by piping through 'jq'"
echo "  - Check OpenObserve UI at http://localhost:5080"
echo "  - Adjust time ranges for more/less data"
echo ""
