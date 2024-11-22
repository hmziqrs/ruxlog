Docker for windows:

```bash
$env:PROJECT="rux_local"; docker compose up -d
```

when changing configs or name else it will not set up new database
```bash
docker-compose down --volumes
```