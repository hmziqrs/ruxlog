set shell := ["bash", "-euo", "pipefail", "-c"]

api_dir := "backend/api"
api_justfile := "backend/api/justfile"
admin_dir := "frontend/admin-dioxus"
dotenv_bin := "dotenv"

default:
    @just --list

# Docker orchestration ------------------------------------------------------

dev env='dev':
    docker compose --env-file .env.{{env}} --profile services --profile storage up -d
    just storage-init {{env}}

dev-full env='dev':
    docker compose --env-file .env.{{env}} --profile full --profile storage up -d --build
    just storage-init {{env}}

stage:
    just dev-full env=stage

prod:
    just dev-full env=prod

storage-init env='dev':
    scripts/rustfs-bootstrap.sh .env.{{env}}

storage-console env='dev':
    @echo "RustFS Console: http://localhost:$(grep RUSTFS_CONSOLE_PORT .env.{{env}} | cut -d= -f2)"

storage-reset env='dev':
    docker compose --env-file .env.{{env}} stop rustfs
    docker volume rm -f $(grep PROJECT .env.{{env}} | cut -d= -f2)_rustfs_data || true
    just storage-init {{env}}

logs env='dev':
    docker compose --env-file .env.{{env}} logs -f

ps env='dev':
    docker compose --env-file .env.{{env}} ps

down env='dev':
    scripts/compose-down.sh .env.{{env}}

reset env='dev':
    scripts/compose-down.sh .env.{{env}} --volumes

# Database helpers ----------------------------------------------------------

test-db env='test':
    scripts/test-db-setup.sh .env.{{env}}

# Backend API (Axum) --------------------------------------------------------

# Delegate any command to the backend API justfile
api cmd env='dev' *args='':
    {{dotenv_bin}} -e .env.{{env}} -- just -f {{api_justfile}} {{cmd}} {{args}}

api-dev env='dev':
    just dev {{env}}
    {{dotenv_bin}} -e .env.{{env}} -- just -f {{api_justfile}} dev

tui env='dev' *args='':
    cd {{api_dir}} && set -a && source ../../.env.{{env}} && set +a && cargo run --bin ruxlog_tui -- {{args}}

# Frontend (Dioxus) ---------------------------------------------------------

[private]
_fe app cmd env:
    #!/usr/bin/env bash
    set -euo pipefail
    dir="frontend/{{app}}-dioxus"
    case "{{cmd}}" in
        dev)            cd "$dir" && {{dotenv_bin}} -e "../../.env.{{env}}" -- bash -c 'dx serve --platform web --port ${{uppercase(app)}}_PORT' ;;
        desktop)        cd "$dir" && {{dotenv_bin}} -e "../../.env.{{env}}" -- bash -c 'dx serve --platform desktop --port ${{uppercase(app)}}_PORT' ;;
        desktop-native) cd "$dir" && {{dotenv_bin}} -e "../../.env.{{env}}" -- bash -c 'dx serve --platform desktop --renderer native --port ${{uppercase(app)}}_PORT' ;;
        build)          cd "$dir" && {{dotenv_bin}} -e "../../.env.{{env}}" -- dx build --platform web --release ;;
        bundle)         cd "$dir" && {{dotenv_bin}} -e "../../.env.{{env}}" -- dx bundle --platform web --release ;;
        tailwind)       cd "$dir" && bun run tailwind ;;
        tailwind-build) cd "$dir" && bun run tailwind:build ;;
        install)        {{dotenv_bin}} -e ".env.{{env}}" -- bash -lc "cd '$dir' && bun install" ;;
        clean)          {{dotenv_bin}} -e ".env.{{env}}" -- bash -lc "cd '$dir' && cargo clean" ;;
        *)              echo "Unknown frontend command: {{cmd}}" && exit 1 ;;
    esac

# Admin frontend
admin cmd='dev' env='dev':
    just _fe admin {{cmd}} {{env}}

# Consumer frontend
consumer cmd='dev' env='dev':
    just _fe consumer {{cmd}} {{env}}

# Admin-specific recipes -----------------------------------------------------

admin-editor-build env='dev':
    {{dotenv_bin}} -e .env.{{env}} -- bash -lc 'cd {{admin_dir}} && bun run editor:build'

admin-editor-watch env='dev':
    {{dotenv_bin}} -e .env.{{env}} -- bash -lc 'cd {{admin_dir}} && bun run editor:watch'

admin-rpxy env='dev':
    {{dotenv_bin}} -e .env.{{env}} -- bash -lc 'cd {{admin_dir}} && bun run rpxy'
