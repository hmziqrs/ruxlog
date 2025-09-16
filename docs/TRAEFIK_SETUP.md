# Traefik Deployment Guide

## Overview
Traefik sits in front of the containers, terminates HTTPS via Let's Encrypt, and routes requests to services that advertise themselves with labels. No manual template edits are required—add the right labels, and Traefik discovers the app automatically.

## Prerequisites
- Docker Engine 24+ with the Compose plugin (`docker compose version`).
- Public DNS pointing `BACKEND_DOMAIN` (and any other exposed hosts) to this server.
- Firewall rules that allow inbound TCP 80/443 (e.g. `ufw allow 80`, `ufw allow 443`).

## Configure Environment Variables
Compose interpolates label values at launch time. Provide them either by exporting variables in your shell or by creating an env file you pass to every `docker compose` command. Example `deploy.env`:

```env
PROJECT=ruxlog
BACKEND_DOMAIN=api.example.com
BACKEND_RATE_AVG=10
BACKEND_RATE_BURST=20
```

Load it whenever you orchestrate the stack:

```bash
docker compose --env-file deploy.env -f docker-compose.prod.yml up -d
```

## Prepare Traefik
1. Copy `traefik/.env.prod.example` to `traefik/.env.prod` and set `PROJECT` plus the Let's Encrypt contact email (`ACME_EMAIL`).
2. Create the ACME storage file once with the correct permissions:
   ```bash
   cd traefik
   mkdir -p data
   touch data/acme.json
   chmod 600 data/acme.json
   ```
3. Start the backend stack from the repository root (ensures the shared `${PROJECT}_network` exists):
   ```bash
   docker compose --env-file deploy.env -f docker-compose.prod.yml up -d
   ```
4. Launch Traefik from the repository root:
   ```bash
   docker compose --env-file traefik/.env.prod -f traefik/docker-compose.prod.yml up -d
   ```
   Traefik binds to ports 80/443, listens on the `${PROJECT}_network`, and uses the Docker provider to read service labels. Certificates are requested automatically through the HTTP-01 challenge and stored in `traefik/data/acme.json`.

## Adding More Services
- Attach the service container to `${PROJECT}_network` and add labels such as:
  ```yaml
  labels:
    - traefik.enable=true
    - traefik.http.routers.myapp.rule=Host(`app.example.com`)
    - traefik.http.routers.myapp.entrypoints=websecure
    - traefik.http.routers.myapp.tls.certresolver=letsencrypt
    - traefik.http.services.myapp.loadbalancer.server.port=3000
  ```
- Optional middlewares (rate limiting, redirects) are configured via labels as well; refer to `docker-compose.prod.yml` for examples.
- Changes take effect with `docker compose up -d`; Traefik hot-reloads without a container restart.

## Operational Tips
- Back up `traefik/data/acme.json` if you rebuild the host to avoid Let's Encrypt rate limits.
- Inspect routing and certificate events with `docker logs ${PROJECT}_traefik`.
- Enable the dashboard for troubleshooting by adding `--api.dashboard=true` to the Traefik command list and protecting it behind Basic Auth.
- For local work, use `docker compose --env-file deploy.env -f traefik/docker-compose.dev.yml up -d`; dev routing uses HTTP only and still relies on labels.

## Troubleshooting
- **ACME token errors**: confirm that port 80 is reachable from the internet and the DNS record resolves to this server.
- **404s from Traefik**: make sure the request host matches a router rule and that the target container is connected to `${PROJECT}_network`.
- **Certificates not persisting**: double-check the permissions and bind mount for `traefik/data/acme.json`.
