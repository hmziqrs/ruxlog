#!/bin/bash
set -e

#!/bin/bash

# Wait for PostgreSQL to be ready
until pg_isready -h postgres -p 5432 -U "$POSTGRES_USER"; do
  echo "Waiting for PostgreSQL..."
  sleep 2
done

# Run migrations
diesel migration run --database-url "$DATABASE_URL"

# Run the application
exec "$@"
