# Rebuild and restart only the backend service

docker compose -f docker-compose.prod.yml up -d --build backend

2. If you need to force recreate the container:

```bash
export PROJECT="rux_local"
```

```bash
docker compose --env-file .env.prod -f docker-compose.prod.yml up -d --build --force-recreate backend
```

3. To check the logs after deployment:

```bash
docker compose -f docker-compose.prod.yml logs -f backend
```

4. If you need to completely stop and remove everything before rebuilding:

```bash
# Stop all services
docker compose -f docker-compose.prod.yml down

# Rebuild and start all services
docker compose -f docker-compose.prod.yml up -d --build
```

5. To check the status of all services:

```bash
docker compose -f docker-compose.prod.yml ps
```
