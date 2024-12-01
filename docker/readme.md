Docker for windows:

```bash
$env:PROJECT="rux_local"; docker compose up -d
```

```bash
export PROJECT="rux_local" && docker compose up -d
```

```bash
docker compose --env-file .env.prod -f docker-compose.prod.yml up -d
```

```bash
docker compose down --rmi all; docker compose build --no-cache; docker compose up -d
```

```bash
$env:PROJECT="rux_local"; docker compose down --rmi all; docker compose build --no-cache; docker compose up -d
```

when changing configs or name else it will not set up new database

```bash
docker-compose down --volumes
```

docker run -p 8888:8888 ruxlog --env-file .env

docker build -t ruxlog:v1.0 .

docker run -p 8888:8888 ruxlog:v1.0

docker run --env-file .\.env -p 8888:8888 ruxlog:v1.0

docker run --env-file .\.env.docker -p 8888:8888 ruxlog:v1.0

For creating admin users on project setup

```bash
docker exec -it rux_local_postgres psql -U root -d ruxlog -f ./docker/postgres/admin_users.sql
```

```bash
docker exec -i rux_local_postgres psql -U rroot -d ruxlog < ./docker/postgres/admin_users.sql
```

Nginx:

```

```
