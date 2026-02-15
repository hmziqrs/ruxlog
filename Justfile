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

api-dev env='dev':
    just dev {{env}}
    {{dotenv_bin}} -e .env.{{env}} -- just -f {{api_justfile}} dev

api-remote:
    cd {{api_dir}} && set -a && source ../../.env.remote && set +a && just dev

api-tui env='dev' *args:
    cd {{api_dir}} && set -a && source ../../.env.{{env}} && set +a && cargo run --bin ruxlog_tui -- {{args}}

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
    cd {{admin_dir}} && {{dotenv_bin}} -e ../../.env.{{env}} -- bash -c 'dx serve --port $ADMIN_PORT'

admin-remote:
    cd {{admin_dir}} && set -a && source ../../.env.remote && set +a && dx serve --port $ADMIN_PORT

admin-desktop env='dev':
    cd {{admin_dir}} && {{dotenv_bin}} -e ../../.env.{{env}} -- bash -c 'dx serve --platform desktop --port $ADMIN_PORT'

admin-build env='dev':
    cd {{admin_dir}} && {{dotenv_bin}} -e ../../.env.{{env}} -- dx build --platform web --release

admin-bundle env='dev':
    cd {{admin_dir}} && {{dotenv_bin}} -e ../../.env.{{env}} -- dx bundle --platform web --release



admin-tailwind:
    cd {{admin_dir}} && bun run tailwind

admin-tailwind-build:
    cd {{admin_dir}} && bun run tailwind:build


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
    cd {{consumer_dir}} && {{dotenv_bin}} -e ../../.env.{{env}} -- bash -c 'dx serve --port $CONSUMER_PORT'

consumer-remote:
    cd {{consumer_dir}} && set -a && source ../../.env.remote && set +a && dx serve --port $CONSUMER_PORT

consumer-desktop env='dev':
    cd {{consumer_dir}} && {{dotenv_bin}} -e ../../.env.{{env}} -- bash -c 'dx serve --platform desktop --port $CONSUMER_PORT'

consumer-build env='dev':
    cd {{consumer_dir}} && {{dotenv_bin}} -e ../../.env.{{env}} -- dx build --platform web --release

consumer-bundle env='dev':
    cd {{consumer_dir}} && {{dotenv_bin}} -e ../../.env.{{env}} -- dx bundle --platform web --release

consumer-build-ssg env='prod':
    #!/usr/bin/env bash
    set -euo pipefail
    echo "=========================================="
    echo "Building Consumer Frontend with SEO"
    echo "=========================================="
    echo "Environment: {{env}}"
    echo ""

    # Load environment variables from .env file
    ENV_FILE=".env.{{env}}"
    if [ ! -f "$ENV_FILE" ]; then
        echo "Error: $ENV_FILE not found"
        exit 1
    fi

    set -a
    source "$ENV_FILE"
    set +a

    echo "API URL: $SITE_URL"
    echo "Consumer URL: $CONSUMER_SITE_URL"
    echo ""

    # Build the optimized production bundle
    echo "Building optimized production bundle..."
    cd {{consumer_dir}} && dx bundle --platform web --release

    echo ""
    echo "=========================================="
    echo "✓ Build complete!"
    echo "=========================================="
    echo ""
    echo "Output directory: {{consumer_dir}}/dist/"
    echo ""
    echo "SEO Features Included:"
    echo "  ✓ Dynamic meta tags per page"
    echo "  ✓ Open Graph tags for social media"
    echo "  ✓ Twitter Cards"
    echo "  ✓ JSON-LD structured data"
    echo "  ✓ Canonical URLs"
    echo "  ✓ robots.txt"
    echo ""
    echo "Next steps:"
    echo "  1. Test locally: cd {{consumer_dir}}/dist && python3 -m http.server 8000"
    echo "  2. Deploy to static hosting (Netlify, Vercel, Cloudflare Pages, etc.)"
    echo "  3. Verify SEO with Facebook Debugger and Twitter Card Validator"
    echo ""

consumer-tailwind:
    cd {{consumer_dir}} && bun run tailwind

consumer-tailwind-build:
    cd {{consumer_dir}} && bun run tailwind:build


consumer-install env='dev':
    {{dotenv_bin}} -e .env.{{env}} -- bash -lc 'cd {{consumer_dir}} && bun install'

consumer-clean env='dev':
    {{dotenv_bin}} -e .env.{{env}} -- bash -lc 'cd {{consumer_dir}} && cargo clean'
