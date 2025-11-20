set shell := ["bash", "-euo", "pipefail", "-c"]

api_dir := "backend/api"
api_justfile := "backend/api/justfile"
admin_dir := "frontend/admin-dioxus"
consumer_dir := "frontend/consumer-dioxus"
dotenv_bin := "dotenv"

default:
    @just --list

# Docker orchestration ------------------------------------------------------

dev env='dev':
    docker compose --env-file .env.{{env}} --profile services --profile storage up -d

dev-full env='dev':
    docker compose --env-file .env.{{env}} --profile full --profile storage up -d --build

stage:
    just dev-full env=stage

prod:
    just dev-full env=prod

storage-init env='dev':
    scripts/garage-bootstrap.sh .env.{{env}}

logs env='dev':
    docker compose --env-file .env.{{env}} logs -f

ps env='dev':
    docker compose --env-file .env.{{env}} ps

down env='dev':
    scripts/compose-down.sh .env.{{env}}

reset env='dev':
    scripts/compose-down.sh .env.{{env}} --volumes

# Database / Garage helpers -------------------------------------------------

test-db env='test':
    scripts/test-db-setup.sh .env.{{env}}

frontend-env env='dev':
    scripts/sync-admin-env.sh .env.{{env}}

# Backend API (Axum) --------------------------------------------------------

api-dev env='dev':
    {{dotenv_bin}} -e .env.{{env}} -- just -f {{api_justfile}} dev

api-watch env='dev':
    {{dotenv_bin}} -e .env.{{env}} -- just -f {{api_justfile}} watch

api-dev-nohup env='dev':
    {{dotenv_bin}} -e .env.{{env}} -- just -f {{api_justfile}} dev-nohup

api-debug env='dev':
    {{dotenv_bin}} -e .env.{{env}} -- just -f {{api_justfile}} debug

api-debug-watch env='dev':
    {{dotenv_bin}} -e .env.{{env}} -- just -f {{api_justfile}} debug-w

api-debug-nohup env='dev':
    {{dotenv_bin}} -e .env.{{env}} -- just -f {{api_justfile}} debug-nohup

api-prod env='dev':
    {{dotenv_bin}} -e .env.{{env}} -- just -f {{api_justfile}} prod

api-prod-build env='dev':
    {{dotenv_bin}} -e .env.{{env}} -- just -f {{api_justfile}} prod-build

api-prod-nohup env='dev':
    {{dotenv_bin}} -e .env.{{env}} -- just -f {{api_justfile}} prod-nohup

api-kill env='dev':
    {{dotenv_bin}} -e .env.{{env}} -- just -f {{api_justfile}} kill

api-kill-nohup env='dev':
    {{dotenv_bin}} -e .env.{{env}} -- just -f {{api_justfile}} kill-nohup

api-kill-all env='dev':
    {{dotenv_bin}} -e .env.{{env}} -- just -f {{api_justfile}} kill-all

api-kill-port env='dev':
    {{dotenv_bin}} -e .env.{{env}} -- just -f {{api_justfile}} kill-port

api-logs env='dev':
    {{dotenv_bin}} -e .env.{{env}} -- just -f {{api_justfile}} logs

api-logs-debug env='dev':
    {{dotenv_bin}} -e .env.{{env}} -- just -f {{api_justfile}} logs-debug

api-logs-dev env='dev':
    {{dotenv_bin}} -e .env.{{env}} -- just -f {{api_justfile}} logs-dev

api-logs-prod env='dev':
    {{dotenv_bin}} -e .env.{{env}} -- just -f {{api_justfile}} logs-prod

api-clean-logs env='dev':
    {{dotenv_bin}} -e .env.{{env}} -- just -f {{api_justfile}} clean-logs

api-archive env='dev':
    {{dotenv_bin}} -e .env.{{env}} -- just -f {{api_justfile}} archive

api-restore zip_file env='dev':
    {{dotenv_bin}} -e .env.{{env}} -- just -f {{api_justfile}} restore {{zip_file}}

api-migrate env='dev':
    {{dotenv_bin}} -e .env.{{env}} -- just -f {{api_justfile}} migrate

api-lsof env='dev':
    {{dotenv_bin}} -e .env.{{env}} -- just -f {{api_justfile}} lsof

# Frontend Admin (Dioxus) ---------------------------------------------------

admin-dev env='dev':
    cd {{admin_dir}} && {{dotenv_bin}} -e ../../.env.{{env}} -- dx serve

admin-desktop env='dev':
    cd {{admin_dir}} && {{dotenv_bin}} -e ../../.env.{{env}} -- dx serve --platform desktop

admin-build env='dev':
    cd {{admin_dir}} && {{dotenv_bin}} -e ../../.env.{{env}} -- dx build --platform web --release

admin-bundle env='dev':
    cd {{admin_dir}} && {{dotenv_bin}} -e ../../.env.{{env}} -- dx bundle --platform web --release



admin-tailwind env='dev':
    {{dotenv_bin}} -e .env.{{env}} -- bash -lc 'cd {{admin_dir}} && bun run tailwind'

admin-tailwind-build env='dev':
    {{dotenv_bin}} -e .env.{{env}} -- bash -lc 'cd {{admin_dir}} && bun run tailwind:build'

admin-editor-build env='dev':
    {{dotenv_bin}} -e .env.{{env}} -- bash -lc 'cd {{admin_dir}} && bun run editor:build'

admin-editor-watch env='dev':
    {{dotenv_bin}} -e .env.{{env}} -- bash -lc 'cd {{admin_dir}} && bun run editor:watch'

admin-rpxy env='dev':
    {{dotenv_bin}} -e .env.{{env}} -- bash -lc 'cd {{admin_dir}} && bun run rpxy'

admin-install env='dev':
    {{dotenv_bin}} -e .env.{{env}} -- bash -lc 'cd {{admin_dir}} && bun install'

admin-clean env='dev':
    {{dotenv_bin}} -e .env.{{env}} -- bash -lc 'cd {{admin_dir}} && cargo clean'

# Frontend Consumer (Dioxus) ------------------------------------------------

consumer-dev env='dev':
    {{dotenv_bin}} -e .env.{{env}} -- bash -lc 'cd {{consumer_dir}} && dx serve'

consumer-desktop env='dev':
    {{dotenv_bin}} -e .env.{{env}} -- bash -lc 'cd {{consumer_dir}} && dx serve --platform desktop'

consumer-build env='dev':
    {{dotenv_bin}} -e .env.{{env}} -- bash -lc 'cd {{consumer_dir}} && dx build --platform web --release'

consumer-bundle env='dev':
    {{dotenv_bin}} -e .env.{{env}} -- bash -lc 'cd {{consumer_dir}} && dx bundle --platform web --release'

consumer-tailwind env='dev':
    {{dotenv_bin}} -e .env.{{env}} -- bash -lc 'cd {{consumer_dir}} && bun run tailwind'

consumer-tailwind-build env='dev':
    {{dotenv_bin}} -e .env.{{env}} -- bash -lc 'cd {{consumer_dir}} && bun run tailwind:build'

consumer-install env='dev':
    {{dotenv_bin}} -e .env.{{env}} -- bash -lc 'cd {{consumer_dir}} && bun install'

consumer-clean env='dev':
    {{dotenv_bin}} -e .env.{{env}} -- bash -lc 'cd {{consumer_dir}} && cargo clean'

# Tailwind watchers for both admin and consumer -------------------------------

tailwind-watch env='dev':
    #!/usr/bin/env bash
    set -euo pipefail
    {{dotenv_bin}} -e .env.{{env}} -- bash -lc 'cd {{admin_dir}} && bun run tailwind' &
    PID1=$!
    {{dotenv_bin}} -e .env.{{env}} -- bash -lc 'cd {{consumer_dir}} && bun run tailwind' &
    PID2=$!
    trap "kill $PID1 $PID2 2>/dev/null || true" EXIT
    wait

