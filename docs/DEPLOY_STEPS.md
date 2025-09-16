# Production Deployment Steps

Follow these commands from the repository root the first time you bring the stack online. Replace placeholder domains and secrets with your real values.

```bash
# 1. Base application environment
cp .env.example .env
# Edit .env with database, Redis, SMTP, and secret values

# 2. Traefik environment (ACME email + project slug)
cp traefik/.env.prod.example traefik/.env.prod
# Edit traefik/.env.prod and set ACME_EMAIL plus any overrides

# 3. Compose interpolation variables for labels and rate limits
cat > deploy.env <<'EOF_ENV'
PROJECT=ruxlog
BACKEND_DOMAIN=api.example.com
BACKEND_RATE_AVG=10
BACKEND_RATE_BURST=20
EOF_ENV

# 4. Start backend dependencies and API (creates the shared network)
docker compose --env-file deploy.env -f docker-compose.prod.yml up -d

# 5. Prepare Traefik's ACME storage directory
mkdir -p traefik/data
touch traefik/data/acme.json
chmod 600 traefik/data/acme.json

# 6. Launch Traefik with automatic Let's Encrypt certificates
docker compose --env-file traefik/.env.prod -f traefik/docker-compose.prod.yml up -d

# 7. Inspect proxy logs (ACME + routing) and test HTTPS endpoint
docker logs ruxlog_traefik
open https://api.example.com
```

## Optional: Local Label-Based Routing
```bash
docker compose --env-file deploy.env -f traefik/docker-compose.dev.yml up -d
# Traefik now serves backend.localhost → backend container
```
