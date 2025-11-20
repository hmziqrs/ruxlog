# Port Allocation Scheme

This document defines the standardized port allocation strategy for the ruxlog project to prevent port conflicts across different environments and services.

## Strategy Overview

**Base Port**: `1100` (assigned to ruxlog project)

**Environment Offsets**:
- **Dev**: Base + 0 (ports 1100-1199)
- **Stage**: Base + 100 (ports 1200-1299)
- **QA/Test**: Base + 200 (ports 1300-1399)

**Service Increments**: Within each environment, services increment by +1 from the base

## Port Assignments

### Development Environment (1100-1199)

| Service | Port | Internal | Description |
|---------|------|----------|-------------|
| API Backend | 1100 | 8888 | Main Rust API server |
| PostgreSQL | 1101 | 5432 | Database |
| Valkey (Redis) | 1102 | 6379 | Cache/session store |
| Garage RPC | 1103 | 3900 | S3-compatible storage RPC |
| Garage Admin | 1104 | 3901 | Garage admin API |
| Garage S3 | 1105 | 3902 | S3 API endpoint |
| Garage Web | 1106 | 3903 | Web interface |

**Access URLs**:
- API: `http://localhost:1100`
- PostgreSQL: `localhost:1101`
- Redis: `localhost:1102`
- S3: `http://127.0.0.1:1105`

### Staging Environment (1200-1299)

| Service | Port | Internal | Description |
|---------|------|----------|-------------|
| API Backend | 1200 | 8888 | Main Rust API server |
| PostgreSQL | 1201 | 5432 | Database |
| Valkey (Redis) | 1202 | 6379 | Cache/session store |
| Garage RPC | 1203 | 3900 | S3-compatible storage RPC |
| Garage Admin | 1204 | 3901 | Garage admin API |
| Garage S3 | 1205 | 3902 | S3 API endpoint |
| Garage Web | 1206 | 3903 | Web interface |

**Note**: Staging typically uses domain names with reverse proxy, so these ports are for direct local access to staging containers.

### QA/Test Environment (1300-1399)

| Service | Port | Internal | Description |
|---------|------|----------|-------------|
| API Backend | 1300 | 8888 | Main Rust API server |
| PostgreSQL | 1301 | 5432 | Database |
| Valkey (Redis) | 1302 | 6379 | Cache/session store |
| Garage RPC | 1303 | 3900 | S3-compatible storage RPC |
| Garage Admin | 1304 | 3901 | Garage admin API |
| Garage S3 | 1305 | 3902 | S3 API endpoint |
| Garage Web | 1306 | 3903 | Web interface |

**Access URLs**:
- API: `http://localhost:1300`
- PostgreSQL: `localhost:1301`
- Redis: `localhost:1302`
- S3: `http://127.0.0.1:1305`

## Benefits

1. **No Port Conflicts**: Different environments can run simultaneously without conflicts
2. **Predictable Ports**: Easy to remember and calculate (base + environment offset + service offset)
3. **Scalable**: 100 ports per environment allows for future service expansion
4. **Clear Organization**: Port number immediately identifies both environment and service

## Adding New Services

When adding a new service:
1. Use the next available increment (+7, +8, +9, etc.) from the environment base
2. Update this document with the new service mapping
3. Add the service to all relevant `.env.*` files with appropriate offsets

## Adding New Environments

For new environments (e.g., production):
1. Choose the next offset (+300, +400, etc.)
2. Apply the offset consistently across all services
3. Document the new environment in this file

## Configuration Files

Port configurations are maintained in:
- `.env.dev` - Development environment
- `.env.stage` - Staging environment
- `.env.test` - QA/Test environment
- `.env.example` - Template for development
- `docker-compose.yml` - Container port mappings

## Quick Reference

```bash
# Calculate port for any service
Port = BasePort + EnvironmentOffset + ServiceOffset

# Examples:
# Dev API: 1100 + 0 + 0 = 1100
# Stage PostgreSQL: 1100 + 100 + 1 = 1201
# Test Redis: 1100 + 200 + 2 = 1302
```

## External Services

The following services use their standard/external ports and are not part of this scheme:
- Supabase: 54321 (external service)
- Quickwit: 7280 (optional observability)
- Frontend Dev Server: 3000/5173 (Vite/npm dev servers)
- Traefik: 80/443 (HTTP/HTTPS reverse proxy)
