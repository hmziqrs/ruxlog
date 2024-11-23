Docker for windows:

```bash
$env:PROJECT="rux_local"; docker compose up -d
```

```bash
export PROJECT="rux_local" && docker compose up -d
```

when changing configs or name else it will not set up new database
```bash
docker-compose down --volumes
```


docker run -p 8888:8888 ruxlog --env-file .env

docker build -t ruxlog:v1.0 .

docker run -p 8888:8888 ruxlog:v1.0

docker run --env-file .\.env  -p 8888:8888 ruxlog:v1.0

docker run --env-file .\.env.docker  -p 8888:8888 ruxlog:v1.0
