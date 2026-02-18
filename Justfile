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
        dev)                cd "$dir" && {{dotenv_bin}} -e "../../.env.{{env}}" -- bash -c 'dx serve --platform web --port ${{uppercase(app)}}_PORT' ;;
        desktop)            cd "$dir" && {{dotenv_bin}} -e "../../.env.{{env}}" -- bash -c 'dx serve --platform desktop --port ${{uppercase(app)}}_PORT' ;;
        desktop-native)     cd "$dir" && {{dotenv_bin}} -e "../../.env.{{env}}" -- bash -c 'dx serve --platform desktop --renderer native --port ${{uppercase(app)}}_PORT' ;;
        mobile)             cd "$dir" && {{dotenv_bin}} -e "../../.env.{{env}}" -- bash -c 'dx serve --platform android --port ${{uppercase(app)}}_PORT' ;;
        mobile-native)      cd "$dir" && {{dotenv_bin}} -e "../../.env.{{env}}" -- bash -c 'dx serve --platform android --renderer native --port ${{uppercase(app)}}_PORT' ;;
        build)              cd "$dir" && {{dotenv_bin}} -e "../../.env.{{env}}" -- dx build --platform web --release ;;
        build-desktop)      cd "$dir" && {{dotenv_bin}} -e "../../.env.{{env}}" -- dx build --platform desktop --release ;;
        build-desktop-native) cd "$dir" && {{dotenv_bin}} -e "../../.env.{{env}}" -- dx build --platform desktop --renderer native --release ;;
        build-mobile)       cd "$dir" && {{dotenv_bin}} -e "../../.env.{{env}}" -- dx build --platform android --release ;;
        build-mobile-native) cd "$dir" && {{dotenv_bin}} -e "../../.env.{{env}}" -- dx build --platform android --renderer native --release ;;
        bundle)             cd "$dir" && {{dotenv_bin}} -e "../../.env.{{env}}" -- dx bundle --platform web --release ;;
        bundle-desktop)     cd "$dir" && {{dotenv_bin}} -e "../../.env.{{env}}" -- dx bundle --platform desktop --release ;;
        bundle-mobile)      cd "$dir" && {{dotenv_bin}} -e "../../.env.{{env}}" -- dx bundle --platform android --release ;;
        tailwind)           cd "$dir" && bun run tailwind ;;
        tailwind-build)     cd "$dir" && bun run tailwind:build ;;
        install)            {{dotenv_bin}} -e ".env.{{env}}" -- bash -lc "cd '$dir' && bun install" ;;
        clean)              {{dotenv_bin}} -e ".env.{{env}}" -- bash -lc "cd '$dir' && cargo clean" ;;
        *)                  echo "Unknown frontend command: {{cmd}}" && exit 1 ;;
    esac

# Admin frontend
admin cmd='dev' env='dev':
    just _fe admin {{cmd}} {{env}}

# Consumer frontend
consumer cmd='dev' env='dev':
    just _fe consumer {{cmd}} {{env}}

# Consumer static demo SSG builds (public routes + markdown content)
consumer-demo-build env='dev' base_path='/':
    #!/usr/bin/env bash
    set -euo pipefail
    env_name="{{env}}"
    env_name="${env_name#env=}"
    base_path_arg="{{base_path}}"
    base_path_arg="${base_path_arg#base_path=}"
    cd frontend/consumer-dioxus
    BASE_PATH_OVERRIDE="$base_path_arg" {{dotenv_bin}} -e "../../.env.${env_name}" -- bash -lc '
      set -euo pipefail
      base_path="${CONSUMER_BASE_PATH:-$BASE_PATH_OVERRIDE}"
      unset PORT
      dx build \
        --fullstack \
        --ssg \
        --release \
        --no-default-features \
        --base-path "$base_path" \
        @client --platform web --features "web demo-static-content basic analytics" \
        @server --platform server --features "server demo-static-content basic analytics"

      # dx can leave root index.html as a shell page even when SSG renders "/".
      # Regenerate "/" HTML from the built server so static hosting hydrates correctly.
      out_root="target/dx/consumer-dioxus/release/web"
      server_bin="$out_root/consumer-dioxus"
      public_dir="$out_root/public"
      root_html="$public_dir/index.html"
      if [ -x "$server_bin" ]; then
        tmp_html="$(mktemp)"
        ssg_pid=""
        cleanup_root_render() {
          if [ -n "$ssg_pid" ]; then
            kill "$ssg_pid" >/dev/null 2>&1 || true
            wait "$ssg_pid" 2>/dev/null || true
          fi
          rm -f "$tmp_html"
        }
        trap cleanup_root_render EXIT

        PORT=39999 "$server_bin" >/tmp/consumer_demo_root_ssg.log 2>&1 &
        ssg_pid=$!
        for _ in $(seq 1 100); do
          if curl -fsS "http://127.0.0.1:39999/" -o "$tmp_html"; then
            mv "$tmp_html" "$root_html"
            tmp_html=""
            break
          fi
          sleep 0.1
        done

        cleanup_root_render
        trap - EXIT
      fi
    '

consumer-demo-bundle env='dev' base_path='/':
    #!/usr/bin/env bash
    set -euo pipefail
    env_name="{{env}}"
    env_name="${env_name#env=}"
    base_path_arg="{{base_path}}"
    base_path_arg="${base_path_arg#base_path=}"
    cd frontend/consumer-dioxus
    BASE_PATH_OVERRIDE="$base_path_arg" {{dotenv_bin}} -e "../../.env.${env_name}" -- bash -lc '
      set -euo pipefail
      base_path="${CONSUMER_BASE_PATH:-$BASE_PATH_OVERRIDE}"
      unset PORT
      dx bundle \
        --fullstack \
        --ssg \
        --release \
        --no-default-features \
        --base-path "$base_path" \
        @client --platform web --features "web demo-static-content basic analytics" \
        @server --platform server --features "server demo-static-content basic analytics"

      # dx can leave root index.html as a shell page even when SSG renders "/".
      # Regenerate "/" HTML from the built server so static hosting hydrates correctly.
      out_root="target/dx/consumer-dioxus/release/web"
      server_bin="$out_root/consumer-dioxus"
      public_dir="$out_root/public"
      root_html="$public_dir/index.html"
      if [ -x "$server_bin" ]; then
        tmp_html="$(mktemp)"
        ssg_pid=""
        cleanup_root_render() {
          if [ -n "$ssg_pid" ]; then
            kill "$ssg_pid" >/dev/null 2>&1 || true
            wait "$ssg_pid" 2>/dev/null || true
          fi
          rm -f "$tmp_html"
        }
        trap cleanup_root_render EXIT

        PORT=39999 "$server_bin" >/tmp/consumer_demo_root_ssg.log 2>&1 &
        ssg_pid=$!
        for _ in $(seq 1 100); do
          if curl -fsS "http://127.0.0.1:39999/" -o "$tmp_html"; then
            mv "$tmp_html" "$root_html"
            tmp_html=""
            break
          fi
          sleep 0.1
        done

        cleanup_root_render
        trap - EXIT
      fi
    '

# Desktop builds (with and without native renderer)
admin-desktop env='dev':
    just _fe admin desktop {{env}}

admin-desktop-native env='dev':
    just _fe admin desktop-native {{env}}

consumer-desktop env='dev':
    just _fe consumer desktop {{env}}

consumer-desktop-native env='dev':
    just _fe consumer desktop-native {{env}}

# Mobile builds (Android only - with and without native renderer)
admin-mobile env='dev':
    just _fe admin mobile {{env}}

admin-mobile-native env='dev':
    just _fe admin mobile-native {{env}}

consumer-mobile env='dev':
    just _fe consumer mobile {{env}}

consumer-mobile-native env='dev':
    just _fe consumer mobile-native {{env}}

# Production builds for desktop and mobile
admin-build-desktop env='dev':
    just _fe admin build-desktop {{env}}

admin-build-desktop-native env='dev':
    just _fe admin build-desktop-native {{env}}

admin-build-mobile env='dev':
    just _fe admin build-mobile {{env}}

admin-build-mobile-native env='dev':
    just _fe admin build-mobile-native {{env}}

consumer-build-desktop env='dev':
    just _fe consumer build-desktop {{env}}

consumer-build-desktop-native env='dev':
    just _fe consumer build-desktop-native {{env}}

consumer-build-mobile env='dev':
    just _fe consumer build-mobile {{env}}

consumer-build-mobile-native env='dev':
    just _fe consumer build-mobile-native {{env}}

# Bundling for distribution
admin-bundle-desktop env='dev':
    just _fe admin bundle-desktop {{env}}

admin-bundle-mobile env='dev':
    just _fe admin bundle-mobile {{env}}

consumer-bundle-desktop env='dev':
    just _fe consumer bundle-desktop {{env}}

consumer-bundle-mobile env='dev':
    just _fe consumer bundle-mobile {{env}}

# Admin-specific recipes -----------------------------------------------------

admin-editor-build env='dev':
    {{dotenv_bin}} -e .env.{{env}} -- bash -lc 'cd {{admin_dir}} && bun run editor:build'

admin-editor-watch env='dev':
    {{dotenv_bin}} -e .env.{{env}} -- bash -lc 'cd {{admin_dir}} && bun run editor:watch'

admin-rpxy env='dev':
    {{dotenv_bin}} -e .env.{{env}} -- bash -lc 'cd {{admin_dir}} && bun run rpxy'
