set shell := ["bash", "-euo", "pipefail", "-c"]

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
