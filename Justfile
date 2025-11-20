set shell := ["bash", "-euo", "pipefail", "-c"]

api_dir := "backend/api"
api_justfile := "backend/api/justfile"
admin_dir := "frontend/admin-dioxus"
dotenv_bin := "dotenv"

default:
    @just --list

# Docker orchestration ------------------------------------------------------

dev env_file='.env.dev':
    docker compose --env-file {{env_file}} --profile services --profile storage up -d

dev-full env_file='.env.dev':
    docker compose --env-file {{env_file}} --profile full --profile storage up -d --build

stage:
    just dev-full env_file='.env.stage'

prod:
    just dev-full env_file='.env.prod'

storage-init env_file='.env.dev':
    scripts/garage-bootstrap.sh {{env_file}}

logs env_file='.env.dev':
    docker compose --env-file {{env_file}} logs -f

ps env_file='.env.dev':
    docker compose --env-file {{env_file}} ps

down env_file='.env.dev':
    scripts/compose-down.sh {{env_file}}

reset env_file='.env.dev':
    scripts/compose-down.sh {{env_file}} --volumes

# Database / Garage helpers -------------------------------------------------

test-db env_file='.env.test':
    scripts/test-db-setup.sh {{env_file}}

frontend-env env_file='.env.dev':
    scripts/sync-admin-env.sh {{env_file}}

# Backend API (Axum) --------------------------------------------------------

api-dev env_file='.env.dev':
    {{dotenv_bin}} -e {{env_file}} -- just -f {{api_justfile}} dev

api-watch env_file='.env.dev':
    {{dotenv_bin}} -e {{env_file}} -- just -f {{api_justfile}} watch

api-dev-nohup env_file='.env.dev':
    {{dotenv_bin}} -e {{env_file}} -- just -f {{api_justfile}} dev-nohup

api-debug env_file='.env.dev':
    {{dotenv_bin}} -e {{env_file}} -- just -f {{api_justfile}} debug

api-debug-watch env_file='.env.dev':
    {{dotenv_bin}} -e {{env_file}} -- just -f {{api_justfile}} debug-w

api-debug-nohup env_file='.env.dev':
    {{dotenv_bin}} -e {{env_file}} -- just -f {{api_justfile}} debug-nohup

api-prod env_file='.env.dev':
    {{dotenv_bin}} -e {{env_file}} -- just -f {{api_justfile}} prod

api-prod-build env_file='.env.dev':
    {{dotenv_bin}} -e {{env_file}} -- just -f {{api_justfile}} prod-build

api-prod-nohup env_file='.env.dev':
    {{dotenv_bin}} -e {{env_file}} -- just -f {{api_justfile}} prod-nohup

api-kill env_file='.env.dev':
    {{dotenv_bin}} -e {{env_file}} -- just -f {{api_justfile}} kill

api-kill-nohup env_file='.env.dev':
    {{dotenv_bin}} -e {{env_file}} -- just -f {{api_justfile}} kill-nohup

api-kill-all env_file='.env.dev':
    {{dotenv_bin}} -e {{env_file}} -- just -f {{api_justfile}} kill-all

api-kill-port env_file='.env.dev':
    {{dotenv_bin}} -e {{env_file}} -- just -f {{api_justfile}} kill-port

api-logs env_file='.env.dev':
    {{dotenv_bin}} -e {{env_file}} -- just -f {{api_justfile}} logs

api-logs-debug env_file='.env.dev':
    {{dotenv_bin}} -e {{env_file}} -- just -f {{api_justfile}} logs-debug

api-logs-dev env_file='.env.dev':
    {{dotenv_bin}} -e {{env_file}} -- just -f {{api_justfile}} logs-dev

api-logs-prod env_file='.env.dev':
    {{dotenv_bin}} -e {{env_file}} -- just -f {{api_justfile}} logs-prod

api-clean-logs env_file='.env.dev':
    {{dotenv_bin}} -e {{env_file}} -- just -f {{api_justfile}} clean-logs

api-archive env_file='.env.dev':
    {{dotenv_bin}} -e {{env_file}} -- just -f {{api_justfile}} archive

api-restore zip_file env_file='.env.dev':
    {{dotenv_bin}} -e {{env_file}} -- just -f {{api_justfile}} restore {{zip_file}}

api-migrate env_file='.env.dev':
    {{dotenv_bin}} -e {{env_file}} -- just -f {{api_justfile}} migrate

api-lsof env_file='.env.dev':
    {{dotenv_bin}} -e {{env_file}} -- just -f {{api_justfile}} lsof

# Frontend Admin (Dioxus) ---------------------------------------------------

admin-dev env_file='.env.dev':
    {{dotenv_bin}} -e {{env_file}} -- bash -lc 'cd {{admin_dir}} && dx serve'

admin-desktop env_file='.env.dev':
    {{dotenv_bin}} -e {{env_file}} -- bash -lc 'cd {{admin_dir}} && dx serve --platform desktop'

admin-build env_file='.env.dev':
    {{dotenv_bin}} -e {{env_file}} -- bash -lc 'cd {{admin_dir}} && dx build --platform web --release'

admin-bundle env_file='.env.dev':
    {{dotenv_bin}} -e {{env_file}} -- bash -lc 'cd {{admin_dir}} && dx bundle --platform web --release'

admin-tailwind env_file='.env.dev':
    {{dotenv_bin}} -e {{env_file}} -- bash -lc 'cd {{admin_dir}} && bun run tailwind'

admin-tailwind-build env_file='.env.dev':
    {{dotenv_bin}} -e {{env_file}} -- bash -lc 'cd {{admin_dir}} && bun run tailwind:build'

admin-editor-build env_file='.env.dev':
    {{dotenv_bin}} -e {{env_file}} -- bash -lc 'cd {{admin_dir}} && bun run editor:build'

admin-editor-watch env_file='.env.dev':
    {{dotenv_bin}} -e {{env_file}} -- bash -lc 'cd {{admin_dir}} && bun run editor:watch'

admin-rpxy env_file='.env.dev':
    {{dotenv_bin}} -e {{env_file}} -- bash -lc 'cd {{admin_dir}} && bun run rpxy'

admin-install env_file='.env.dev':
    {{dotenv_bin}} -e {{env_file}} -- bash -lc 'cd {{admin_dir}} && bun install'

admin-clean env_file='.env.dev':
    {{dotenv_bin}} -e {{env_file}} -- bash -lc 'cd {{admin_dir}} && cargo clean'
