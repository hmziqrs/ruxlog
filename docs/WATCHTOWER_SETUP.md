# Watchtower Setup Guide

This guide walks you through enabling zero-SSH deployments using Watchtower. With Watchtower running on your VPS, new images pushed to your registry (GHCR) will be pulled automatically and your containers will be restarted safely.

Our repo already ships a Watchtower service in `docker-compose.prod.yml` and labels the `backend` for safe updates. Follow the steps below to finish the setup.

## What Watchtower does

- Periodically checks container images for updates in your registry.
- Pulls newer image tags and restarts only the labeled containers.
- Performs rolling restarts and cleans up old images (as configured).
- Optional HTTP API lets you trigger an immediate update (webhook) after a CI push.

## Prerequisites

- VPS with Docker and Docker Compose plugin installed (`docker compose version`).
- Your production stack cloned/copied to the VPS (e.g., `/opt/ruxlog`).
- Traefik stack running and sharing the `${PROJECT}_network` (per `docs/DEPLOY_STEPS.md`).
- GHCR access:
  - If your package is public, nothing else to do.
  - If private, log in on the VPS once with a PAT that has `read:packages`.

```bash
# On the VPS
docker login ghcr.io  # use a GitHub Personal Access Token with read:packages
```

## Step 1 — Confirm labels and service

- `backend` is already labeled in `docker-compose.prod.yml`:
  - `com.centurylinklabs.watchtower.enable=true`
- A `watchtower` service is included with:
  - `--label-enable` (only update labeled containers)
  - `--interval 300` (check every 5 minutes) — adjustable
  - `--cleanup` (remove old/dangling images)
  - `--rolling-restart` (safer restarts)
  - `--http-api-update` (enables optional webhook)
  - Docker socket volume mounted (`/var/run/docker.sock`)
  - Attached to the same `network` as `backend`

No YAML edits are required—these are already in the repository.

## Step 2 — Prepare environment files

Ensure the following files exist on the VPS in your app directory (e.g., `/opt/ruxlog`):

- `deploy.env` — compose interpolation vars used by labels/Traefik (see `docs/DEPLOY_STEPS.md`).
- `.env.prod` — your production secrets used by the containers.

Optional (only if you want webhook-triggered instant updates): add to `.env.prod`:

```env
# Expose Watchtower HTTP API through Traefik (disabled by default)
WATCHTOWER_EXPOSE=true
WATCHTOWER_DOMAIN=watchtower.example.com
# Long random token; required to access the API safely
WATCHTOWER_TOKEN=replace-with-a-long-random-string
```

Notes:
- If you do not set `WATCHTOWER_EXPOSE=true`, the API won’t be reachable externally (safer default). Watchtower will still poll every 5 minutes.
- The Traefik labels for Watchtower are already in `docker-compose.prod.yml` and honor these variables.

## Step 3 — Start or restart Watchtower

Bring up (or bounce) only the Watchtower service:

```bash
# From your app directory on the VPS
docker compose --env-file deploy.env -f docker-compose.prod.yml up -d watchtower
```

Verify logs:

```bash
docker logs -f ruxlog_watchtower  # container name uses ${PROJECT}_watchtower
```

You should see it scanning containers and reporting nothing to update initially. After you push a new image tag, it will show pull and restart activity.

## Step 4 — Release and observe rollout

1. Push a Git tag like `v1.2.3` to your GitHub repo.
2. The GitHub Actions workflow builds and pushes images:
   - `ghcr.io/<owner>/<repo>:v1.2.3`
   - `ghcr.io/<owner>/<repo>:1.2.3`
   - `:latest` (only for stable `x.y.z`)
3. Within up to 5 minutes, Watchtower detects the updated tag, pulls it, and restarts the `backend` container.
4. Check:
   - `docker ps` (new image tag on `backend`)
   - `docker logs ruxlog_backend` or your app health endpoint (`/healthz`).

## Step 5 — Optional webhook for instant updates

If you configured `WATCHTOWER_EXPOSE=true`, `WATCHTOWER_DOMAIN`, and `WATCHTOWER_TOKEN` in `.env.prod`, Traefik exposes the Watchtower API at:

```
https://<WATCHTOWER_DOMAIN>/v1/update?token=<WATCHTOWER_TOKEN>
```

- Manual trigger from your machine:

```bash
curl -fsSL "https://watchtower.example.com/v1/update?token=your-long-token"
```

- CI trigger (GitHub Actions):
  - Add a repo secret `WATCHTOWER_WEBHOOK_URL` set to the full HTTPS URL above.
  - The workflow includes a job that curls this URL after a successful push (only runs if the secret is present).

## Security best practices

- Keep the Watchtower API disabled by default. When enabling, always require a strong token.
- Prefer HTTPS via Traefik with a proper domain and certificate.
- Consider IP allowlists, Traefik rate limits, or BasicAuth in front of the webhook if exposed publicly.
- Use pinned tags (e.g., `v1.2.3`) for predictable rollouts; avoid floating tags in production unless you understand the trade-offs.
- Limit your PAT scopes for `docker login ghcr.io` to `read:packages` on the VPS.

## Troubleshooting

- Watchtower doesn’t update anything:
  - Ensure the container is labeled `com.centurylinklabs.watchtower.enable=true`.
  - Confirm Watchtower and the target container share a Docker network.
  - Verify your tag was actually pushed to GHCR and is visible.
- Pulls fail with 401/403:
  - Run `docker login ghcr.io` on the VPS (private packages need a PAT with `read:packages`).
- Webhook 404/403:
  - Confirm `WATCHTOWER_EXPOSE=true`, correct `WATCHTOWER_DOMAIN`, DNS/Traefik routing, and a valid `WATCHTOWER_TOKEN`.
- Too frequent checks:
  - Increase the `--interval` value (seconds) in the Watchtower command.

## Reference

- Compose file: `docker-compose.prod.yml` (backend label + watchtower service)
- Main deployment steps: `docs/DEPLOY_STEPS.md`
- Watchtower image: https://github.com/containrrr/watchtower
