# Watchtower Setup Guide

This guide walks you through enabling zero-SSH deployments using Watchtower. With Watchtower running on your VPS, new images pushed to your public registry (GHCR) will be pulled automatically and your containers will be restarted safely.

Our repo already ships a Watchtower service in `docker-compose.prod.yml` and labels the `backend` for safe updates. Follow the steps below to finish the setup.

## What Watchtower does

- Periodically checks container images for updates in your registry.
- Pulls newer image tags and restarts only the labeled containers.
- Performs rolling restarts and cleans up old images (as configured).
  

## Prerequisites

- VPS with Docker and Docker Compose plugin installed (`docker compose version`).
- Your production stack cloned/copied to the VPS (e.g., `/opt/ruxlog`).
- Traefik stack running and sharing the `${PROJECT}_network` (per `docs/DEPLOY_STEPS.md`).
  

## Step 1 — Confirm labels and service

- `backend` is already labeled in `docker-compose.prod.yml`:
  - `com.centurylinklabs.watchtower.enable=true`
- A `watchtower` service is included with:
  - `--label-enable` (only update labeled containers)
  - `--interval 300` (check every 5 minutes) — adjustable
  - `--cleanup` (remove old/dangling images)
  - `--rolling-restart` (safer restarts)
  - Docker socket volume mounted (`/var/run/docker.sock`)
  - Attached to the same `network` as `backend`

No YAML edits are required—these are already in the repository.

## Step 2 — Prepare environment files

Ensure the following files exist on the VPS in your app directory (e.g., `/opt/ruxlog`):

- `deploy.env` — compose interpolation vars used by labels/Traefik (see `docs/DEPLOY_STEPS.md`).
- `.env.prod` — your production secrets used by the containers.

There is no webhook or external API exposure in this setup; Watchtower will simply poll every 5 minutes and update labeled containers when a new image tag is available.

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

There’s nothing else to configure beyond tagging releases (`vX.Y.Z`). The CI builds and pushes your image to GHCR, and within up to 5 minutes Watchtower will pull the new tag and restart the backend.

## Security best practices

- Prefer HTTPS via Traefik with a proper domain and certificate for your app traffic.
- Use pinned tags (e.g., `v1.2.3`) for predictable rollouts; avoid floating tags in production unless you understand the trade-offs.
  

## Troubleshooting

- Watchtower doesn’t update anything:
  - Ensure the container is labeled `com.centurylinklabs.watchtower.enable=true`.
  - Confirm Watchtower and the target container share a Docker network.
  - Verify your tag was actually pushed to GHCR and is visible.
  
- Too frequent checks:
  - Increase the `--interval` value (seconds) in the Watchtower command.

## Reference

- Compose file: `docker-compose.prod.yml` (backend label + watchtower service)
- Main deployment steps: `docs/DEPLOY_STEPS.md`
- Watchtower image: https://github.com/containrrr/watchtower
