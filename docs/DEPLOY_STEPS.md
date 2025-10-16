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
curl -I https://api.example.com/healthz
```

## Optional: Local Label-Based Routing
```bash
docker compose --env-file deploy.env -f traefik/docker-compose.dev.yml up -d
# Traefik now serves backend.localhost → backend container
```

## Automated CI/CD with GitHub Actions

This repo includes a workflow at `.github/workflows/cicd.yml` that:

- Builds and pushes a container image to GHCR when you push a Git tag like `v1.2.3`.
- Deployment is handled by Watchtower polling on your VPS (no SSH, no webhooks).

### Prerequisites on the VPS

- Docker and Docker Compose Plugin installed (`docker compose version`).
- A non-root user with passwordless sudo or access to the Docker daemon.
- A directory for the app, for example `/opt/ruxlog`, containing:
	- `docker-compose.prod.yml` (place the repo files on the server, e.g. via git clone or scp)
	- `.env.prod` (your production secrets; never stored in CI)
	- `deploy.env` (compose interpolation variables used by labels and Traefik)
	- `docker/redis/prod.acl`
- Traefik stack running (see steps above) and sharing the `${PROJECT}_network`.

If your GHCR images are private, ensure the VPS can pull them:

```bash
docker login ghcr.io  # use a PAT with read:packages
```

### GitHub Repository Secrets (optional)

The workflow builds and pushes to GHCR using `GITHUB_TOKEN` and requires no secrets for the build step.

Optional secret for instant rollout via webhook (if you expose Watchtower’s API):

- `WATCHTOWER_WEBHOOK_URL`: The HTTPS URL to Watchtower’s update endpoint, e.g. `https://watchtower.example.com/v1/update?token=...`.

Note: GHCR pull credentials (if needed) should be configured on the VPS with `docker login ghcr.io`, not as GitHub secrets.

### How it works

On push of a version tag (e.g., `v1.2.3`):

1. Build & push image with tags:
	- `ghcr.io/<owner>/<repo>:v1.2.3` (full tag)
	- `ghcr.io/<owner>/<repo>:1.2.3` (raw semver)
	- `ghcr.io/<owner>/<repo>:latest` (only for stable semver x.y.z)
2. Your VPS auto-detects and rolls out the update via Watchtower polling.

### Default: SSH-less auto-deploy (Watchtower)

You can avoid SSH entirely and let the server auto-update containers when new images are available:

- The compose file includes a `watchtower` service that checks for new images every 5 minutes and restarts only containers labeled with `com.centurylinklabs.watchtower.enable=true` (already set on `backend`).
- Make sure your server can pull from GHCR:
	- Either make the package public, or
	- Run `docker login ghcr.io` once on the VPS (use a PAT with `read:packages`).
There is no webhook in this setup; Watchtower will pick up new images on its polling interval (5 minutes by default).

For a step-by-step Watchtower setup guide, see `docs/WATCHTOWER_SETUP.md`.

### Optional: SSH-based deploy (legacy)

If you prefer explicit SSH-driven deployments (to run DB migrations, coordinated multi-service changes, or custom health gates), we can keep a separate workflow that connects to your VPS and runs `docker compose up -d backend`. The default pipeline no longer uses SSH.

### Manual run

From the Actions tab, use “Run workflow” and set `version` to a tag (e.g. `v1.2.3`) if you want to rebuild/redeploy that exact version.

### Pre-releases

If you push a tag that doesn’t match strict `x.y.z` (like `v1.2.3-rc.1`), images are pushed with `v1.2.3-rc.1` and `1.2.3-rc.1` tags, but `latest` is not updated.

### Rollback

To rollback, re-run the workflow with a previous tag (use the SHA from the prior successful run) or update `BACKEND_IMAGE` on the server and run compose:

```bash
BACKEND_IMAGE=ghcr.io/<owner>/<repo>:<tag> IMAGE_TAG=<tag> \
	docker compose --env-file deploy.env -f docker-compose.prod.yml up -d backend
```
