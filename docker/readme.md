Docker for windows:

```bash
$env:PROJECT="rux_local"; docker compose up -d
```

when changing configs or name else it will not set up new database
```bash
docker-compose down --volumes
```


 docker run -p 8888:8888 ruxlog-backend --env-file .env
 docker build -t ruxlog-backend .
docker run -p 8888:8888 ruxlog-backend

docker run --env-file .\.env  -p 8888:8888 ruxlog-backend
