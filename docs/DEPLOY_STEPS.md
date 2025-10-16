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

- Builds and pushes a container image to GHCR on every push to `master`.
- SSHes into your VPS to pull the image and restart the `backend` service using `docker-compose.prod.yml`.

### Prerequisites on the VPS

- Docker and Docker Compose Plugin installed (`docker compose version`).
- A non-root user with passwordless sudo or access to the Docker daemon.
- A directory for the app, for example `/opt/ruxlog`, containing:
	- `docker-compose.prod.yml` (the workflow syncs this by default)
	- `.env.prod` (your production secrets; not synced by CI)
	- `deploy.env` (compose interpolation variables; CI can create if absent)
	- `docker/redis/prod.acl` (synced by CI)
- Traefik stack running (see steps above) and sharing the `${PROJECT}_network`.

### GitHub Repository Secrets

Create these in GitHub → Settings → Secrets and variables → Actions:

- `SSH_PRIVATE_KEY`: Private key for SSH to the VPS (the matching public key must be in `~/.ssh/authorized_keys`).
- `VPS_HOST`: Hostname or IP of your VPS.
- `VPS_USER`: SSH username.
- `VPS_APP_DIR`: Absolute path on the VPS, e.g. `/opt/ruxlog`.
- `PROJECT`: Project slug, e.g. `ruxlog`.
- `BACKEND_DOMAIN`: Your backend domain used by Traefik.
- `BACKEND_RATE_AVG` and `BACKEND_RATE_BURST`: Optional rate limit overrides.
- Optional if your GHCR images are private (recommended):
	- `GHCR_USER`: Typically your GitHub username or a bot account.
	- `GHCR_TOKEN`: A classic PAT with `read:packages` scope for pulling on VPS.

Note: The workflow uses the GitHub-provided `GITHUB_TOKEN` to push to GHCR during build. For the VPS to pull, you can either make the GHCR package public or provide `GHCR_USER` and `GHCR_TOKEN` secrets for login on the VPS.

### How it works

On push to `master`:

1. Build & push image to `ghcr.io/<owner>/<repo>:<short-sha>` and `:latest`.
2. SSH to VPS, ensure app directory exists and required files are present.
3. Write `deploy.env` if missing with `PROJECT` and Traefik label vars.
4. Log in to GHCR (if credentials provided), pull the new image, and run:

	 `docker compose --env-file deploy.env -f docker-compose.prod.yml up -d backend`

Compose picks up `BACKEND_IMAGE` from the deploy step, so the backend uses the pre-built image instead of building on the server.

### Manual run with a custom tag

From the Actions tab, use “Run workflow” and set `image_tag` to any tag (e.g. `staging-123`). This will build/push and deploy that tag.

### Rollback

To rollback, re-run the workflow with a previous tag (use the SHA from the prior successful run) or update `BACKEND_IMAGE` on the server and run compose:

```bash
BACKEND_IMAGE=ghcr.io/<owner>/<repo>:<tag> IMAGE_TAG=<tag> \
	docker compose --env-file deploy.env -f docker-compose.prod.yml up -d backend
```
